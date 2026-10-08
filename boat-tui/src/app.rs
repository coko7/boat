use anyhow::{Context, Result};
use boat_lib::{
    models::activity::{Activity, NewActivity},
    repository::{Id, activities_repository},
};
use chrono::{DateTime, Utc};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ListState,
};
use rusqlite::Connection;
use std::{
    cmp::Reverse,
    fs,
    process::Command,
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    form::{FormEvent, NewActivityForm},
    ui,
};

const TICK_RATE: Duration = Duration::from_millis(250);
const REFRESH_RATE: Duration = Duration::from_secs(5);
const STATUS_TTL: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    Cancel,
    New,
    Meeting,
    Note,
    Jira,
    Filter,
    Edit,
    Config,
    Help,
    Quit,
}

impl Action {
    pub const ALL: [Action; 12] = [
        Action::Start,
        Action::Stop,
        Action::Cancel,
        Action::New,
        Action::Meeting,
        Action::Note,
        Action::Jira,
        Action::Filter,
        Action::Edit,
        Action::Config,
        Action::Help,
        Action::Quit,
    ];

    pub fn from_key(code: KeyCode) -> Option<Self> {
        let action = match code {
            KeyCode::Enter | KeyCode::Char('r') => Action::Start,
            KeyCode::Char('s') => Action::Stop,
            KeyCode::Char('c') => Action::Cancel,
            KeyCode::Char('n') => Action::New,
            KeyCode::Char('m') => Action::Meeting,
            KeyCode::Char('o') => Action::Note,
            KeyCode::Char('J') => Action::Jira,
            KeyCode::Char('/') => Action::Filter,
            KeyCode::Char('e') => Action::Edit,
            KeyCode::Char('C') => Action::Config,
            KeyCode::Char('?') => Action::Help,
            KeyCode::Char('q') => Action::Quit,
            _ => return None,
        };
        Some(action)
    }

    pub fn key_hint(self) -> &'static str {
        match self {
            Action::Start => "⏎/r",
            Action::Stop => "s",
            Action::Cancel => "c",
            Action::New => "n",
            Action::Meeting => "m",
            Action::Note => "o",
            Action::Jira => "J",
            Action::Filter => "/",
            Action::Edit => "e",
            Action::Config => "C",
            Action::Help => "?",
            Action::Quit => "q",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Action::Start => "start",
            Action::Stop => "stop",
            Action::Cancel => "cancel",
            Action::New => "new",
            Action::Meeting => "meeting",
            Action::Note => "note",
            Action::Jira => "jira",
            Action::Filter => "filter",
            Action::Edit => "edit",
            Action::Config => "config",
            Action::Help => "help",
            Action::Quit => "quit",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Action::Start => "start/resume the selected activity",
            Action::Stop => "pause/stop the current activity",
            Action::Cancel => "cancel the current activity",
            Action::New => "create a new activity",
            Action::Meeting => "start a meeting from a preset",
            Action::Note => "view/take notes for the selected activity",
            Action::Jira => "open the selected activity's jira issue",
            Action::Filter => "filter activities by name, tag or id",
            Action::Edit => "modify today's activity logs",
            Action::Config => "tweak the boat configuration",
            Action::Help => "show all keybinds",
            Action::Quit => "quit",
        }
    }

    /// Shown in the footer; the rest are listed in the help popup.
    pub fn is_important(self) -> bool {
        matches!(
            self,
            Action::Start
                | Action::Stop
                | Action::New
                | Action::Filter
                | Action::Help
                | Action::Quit
        )
    }
}

/// What a text prompt is collecting.
#[derive(Debug, Clone)]
pub enum InputKind {
    MeetingName { tag: String },
}

#[derive(Debug, Clone)]
pub struct PickerItem {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Picker {
    pub title: String,
    pub items: Vec<PickerItem>,
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Browse,
    Filter,
    Input {
        kind: InputKind,
        prompt: String,
        placeholder: String,
        value: String,
    },
    NewForm(NewActivityForm),
    MeetingPicker(Picker),
    ConfirmCancel {
        message: String,
    },
    Help,
}

pub struct Status {
    pub message: String,
    pub is_error: bool,
    pub at: Instant,
}

pub struct App {
    pub config: Config,
    pub conn: Connection,
    pub mode: Mode,
    /// All activities, most recently active first
    pub activities: Vec<Activity>,
    pub filter: String,
    pub list_state: ListState,
    pub status: Option<Status>,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: Config, conn: Connection) -> Result<Self> {
        let mut app = Self {
            config,
            conn,
            mode: Mode::Browse,
            activities: vec![],
            filter: String::new(),
            list_state: ListState::default().with_selected(Some(0)),
            status: None,
            should_quit: false,
        };
        app.refresh()?;
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut last_refresh = Instant::now();
        while !self.should_quit {
            terminal.draw(|frame| ui::draw(frame, self))?;

            if event::poll(TICK_RATE)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key, terminal);
            }

            // Pick up changes made by boat-cli in another terminal
            if last_refresh.elapsed() >= REFRESH_RATE {
                self.refresh_or_report();
                last_refresh = Instant::now();
            }

            if self
                .status
                .as_ref()
                .is_some_and(|s| s.at.elapsed() > STATUS_TTL)
            {
                self.status = None;
            }
        }
        Ok(())
    }

    /// Activities matching the current filter, in display order.
    pub fn visible(&self) -> Vec<&Activity> {
        let needles: Vec<String> = self
            .filter
            .split_whitespace()
            .map(str::to_lowercase)
            .collect();
        self.activities
            .iter()
            .filter(|a| {
                let mut hay = format!("{} {}", a.id, a.name.to_lowercase());
                for tag in &a.tags {
                    hay.push(' ');
                    hay.push_str(&tag.to_lowercase());
                }
                needles.iter().all(|n| hay.contains(n))
            })
            .collect()
    }

    pub fn selected(&self) -> Option<&Activity> {
        self.visible().get(self.list_state.selected()?).copied()
    }

    pub fn current(&self) -> Option<&Activity> {
        self.activities.iter().find(|a| is_ongoing(a))
    }

    fn refresh(&mut self) -> Result<()> {
        let selected_id = self.selected().map(|a| a.id);

        let mut activities = activities_repository::get_all(&self.conn)?;
        let now = Utc::now();
        activities.sort_by_key(|a| (Reverse(last_active(a, now)), Reverse(a.id)));
        self.activities = activities;

        self.restore_selection(selected_id);
        Ok(())
    }

    /// Keeps the cursor on the same activity after the list is re-sorted or filtered.
    fn restore_selection(&mut self, id: Option<Id>) {
        let visible = self.visible();
        let index = id
            .and_then(|id| visible.iter().position(|a| a.id == id))
            .unwrap_or_else(|| {
                self.list_state
                    .selected()
                    .unwrap_or(0)
                    .min(visible.len().saturating_sub(1))
            });
        self.list_state.select(Some(index));
    }

    fn select_id(&mut self, id: Id) {
        if let Some(index) = self.visible().iter().position(|a| a.id == id) {
            self.list_state.select(Some(index));
        }
    }

    fn refresh_or_report(&mut self) {
        if let Err(e) = self.refresh() {
            self.error(format!("refresh failed: {e}"));
        }
    }

    fn info(&mut self, message: impl Into<String>) {
        self.status = Some(Status {
            message: message.into(),
            is_error: false,
            at: Instant::now(),
        });
    }

    fn error(&mut self, message: impl Into<String>) {
        self.status = Some(Status {
            message: message.into(),
            is_error: true,
            at: Instant::now(),
        });
    }

    fn handle_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        let result = match self.mode {
            Mode::Browse => self.handle_browse_key(key, terminal),
            Mode::Filter => {
                self.handle_filter_key(key);
                Ok(())
            }
            Mode::Input { .. } => self.handle_input_key(key),
            Mode::NewForm(_) => self.handle_form_key(key),
            Mode::MeetingPicker(_) => self.handle_picker_key(key),
            Mode::ConfirmCancel { .. } => self.handle_confirm_key(key),
            Mode::Help => {
                self.mode = Mode::Browse;
                Ok(())
            }
        };

        if let Err(e) = result {
            self.mode = Mode::Browse;
            self.error(e.to_string());
        }
        self.refresh_or_report();
    }

    fn handle_browse_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
            KeyCode::PageDown => self.list_state.scroll_down_by(10),
            KeyCode::PageUp => self.list_state.scroll_up_by(10),
            KeyCode::Home | KeyCode::Char('g') => self.list_state.select_first(),
            KeyCode::End | KeyCode::Char('G') => self.list_state.select_last(),
            KeyCode::Esc if !self.filter.is_empty() => self.set_filter(String::new()),
            code => {
                if let Some(action) = Action::from_key(code) {
                    self.trigger(action, terminal)?;
                }
            }
        }
        Ok(())
    }

    fn trigger(&mut self, action: Action, terminal: &mut DefaultTerminal) -> Result<()> {
        match action {
            Action::Start => {
                let activity = self.selected().context("no activity selected")?;
                let (id, name) = (activity.id, activity.name.clone());
                activities_repository::start(&mut self.conn, id)?;
                self.info(format!("Started activity: {name} ({id})"));
            }
            Action::Stop => {
                let name = self.current_label().context("No current activity")?;
                activities_repository::stop_current(&self.conn)?;
                self.info(format!("Stopped activity: {name}"));
            }
            Action::Cancel => {
                let name = self.current_label().context("No current activity")?;
                self.mode = Mode::ConfirmCancel {
                    message: format!("Cancel {name}?"),
                };
            }
            Action::New => self.mode = Mode::NewForm(NewActivityForm::default()),
            Action::Meeting => self.open_meeting_picker(),
            Action::Note => self.open_note(terminal)?,
            Action::Jira => self.open_jira()?,
            Action::Filter => self.mode = Mode::Filter,
            Action::Edit => {
                run_suspended(
                    terminal,
                    Command::new("boat").args(["edit", "--period=today"]),
                )?;
            }
            Action::Config => {
                let path = self.config.config_file.clone();
                run_suspended(terminal, Command::new(editor()).arg(path))?;
            }
            Action::Help => self.mode = Mode::Help,
            Action::Quit => self.should_quit = true,
        }
        Ok(())
    }

    fn set_filter(&mut self, filter: String) {
        let selected_id = self.selected().map(|a| a.id);
        self.filter = filter;
        self.restore_selection(selected_id);
    }

    fn handle_filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.set_filter(String::new());
                self.mode = Mode::Browse;
            }
            KeyCode::Enter => self.mode = Mode::Browse,
            KeyCode::Down => self.list_state.select_next(),
            KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Backspace => {
                let mut filter = self.filter.clone();
                filter.pop();
                self.set_filter(filter);
            }
            KeyCode::Char(c) => {
                let filter = format!("{}{c}", self.filter);
                self.set_filter(filter);
            }
            _ => {}
        }
    }

    fn prompt(&mut self, kind: InputKind, prompt: &str, placeholder: &str) {
        self.mode = Mode::Input {
            kind,
            prompt: prompt.to_string(),
            placeholder: placeholder.to_string(),
            value: String::new(),
        };
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Result<()> {
        let Mode::Input { kind, value, .. } = &mut self.mode else {
            return Ok(());
        };

        match key.code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Backspace => {
                value.pop();
            }
            KeyCode::Char(c) => value.push(c),
            KeyCode::Enter => {
                let kind = kind.clone();
                let value = value.trim().to_string();
                self.mode = Mode::Browse;
                self.submit_input(kind, value)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn submit_input(&mut self, kind: InputKind, value: String) -> Result<()> {
        match kind {
            InputKind::MeetingName { tag } => {
                let name = if value.is_empty() {
                    format!(
                        "unnamed meeting (at {})",
                        chrono::Local::now().format("%Y-%m-%d %H:%M")
                    )
                } else {
                    value
                };
                self.create_and_start(name, vec![tag])?;
            }
        }
        Ok(())
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> Result<()> {
        let Mode::NewForm(form) = &mut self.mode else {
            return Ok(());
        };

        match form.handle_key(key) {
            FormEvent::None => {}
            FormEvent::Cancel => self.mode = Mode::Browse,
            FormEvent::Submit(input) => {
                self.mode = Mode::Browse;
                self.create_activity(
                    NewActivity {
                        name: input.name,
                        description: input.description,
                        tags: input.tags,
                    },
                    input.start_now,
                )?;
            }
        }
        Ok(())
    }

    fn create_and_start(&mut self, name: String, tags: Vec<String>) -> Result<()> {
        let new_activity = NewActivity {
            name,
            description: None,
            tags,
        };
        self.create_activity(new_activity, true)
    }

    fn create_activity(&mut self, new_activity: NewActivity, start: bool) -> Result<()> {
        let activity = activities_repository::create(&mut self.conn, new_activity)?;
        if start {
            activities_repository::start(&mut self.conn, activity.id)?;
            self.info(format!(
                "Started new activity: {} ({})",
                activity.name, activity.id
            ));
        } else {
            self.info(format!(
                "Created activity: {} ({})",
                activity.name, activity.id
            ));
        }
        self.follow(activity.id)
    }

    /// Clears the filter and moves the cursor to the given activity once the list is reloaded.
    fn follow(&mut self, id: Id) -> Result<()> {
        self.filter.clear();
        self.refresh()?;
        self.select_id(id);
        Ok(())
    }

    fn open_meeting_picker(&mut self) {
        let items = self
            .config
            .tui
            .meetings
            .iter()
            .map(|m| PickerItem {
                label: m.label.clone(),
                value: m.tag.clone(),
            })
            .collect();
        self.mode = Mode::MeetingPicker(Picker {
            title: " Meetings ".to_string(),
            items,
            selected: 0,
        });
    }

    fn handle_picker_key(&mut self, key: KeyEvent) -> Result<()> {
        let Mode::MeetingPicker(picker) = &mut self.mode else {
            return Ok(());
        };
        let count = picker.items.len();

        match key.code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Down | KeyCode::Char('j') => {
                picker.selected = (picker.selected + 1).min(count.saturating_sub(1));
            }
            KeyCode::Up | KeyCode::Char('k') => picker.selected = picker.selected.saturating_sub(1),
            KeyCode::Enter => {
                let Some(item) = picker.items.get(picker.selected).cloned() else {
                    return Ok(());
                };
                self.mode = Mode::Browse;
                self.start_meeting(item)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn start_meeting(&mut self, item: PickerItem) -> Result<()> {
        let preset = self
            .config
            .tui
            .meetings
            .iter()
            .find(|m| m.tag == item.value)
            .cloned()
            .context("unknown meeting preset")?;

        if preset.create {
            self.prompt(
                InputKind::MeetingName { tag: preset.tag },
                "Meeting name",
                "unnamed meeting",
            );
            return Ok(());
        }

        let latest = self
            .activities
            .iter()
            .filter(|a| a.tags.contains(&preset.tag))
            .max_by_key(|a| a.id)
            .map(|a| (a.id, a.name.clone()));
        match latest {
            Some((id, name)) => {
                activities_repository::start(&mut self.conn, id)?;
                self.info(format!("Started meeting: {name} ({id})"));
                self.follow(id)
            }
            None => self.create_and_start(preset.label, vec![preset.tag]),
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.mode = Mode::Browse;
                let name = self.current_label().unwrap_or_default();
                activities_repository::cancel_current(&self.conn)?;
                self.info(format!("Canceled activity: {name}"));
            }
            KeyCode::Char('n') | KeyCode::Esc => self.mode = Mode::Browse,
            _ => {}
        }
        Ok(())
    }

    fn open_note(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let activity = self.selected().context("no activity selected")?;
        let note_dir = self.config.tui.notes_dir.join(activity.id.to_string());
        let note_path = note_dir.join("note.md");
        if !note_path.exists() {
            fs::create_dir_all(&note_dir)?;
            fs::write(&note_path, format!("# {} - Notes\n", activity.name))?;
        }
        run_suspended(terminal, Command::new(editor()).arg(note_path))
    }

    fn open_jira(&mut self) -> Result<()> {
        let issue = self
            .selected()
            .and_then(|a| tag_value(a, "jira"))
            .context("selected activity has no jira tag")?;
        let base = self
            .config
            .tui
            .jira_base_url
            .as_deref()
            .context("jira_base_url is not set in tui.toml")?;
        let url = format!("{}/browse/{issue}", base.trim_end_matches('/'));
        open_url(&url)?;
        self.info(format!("Opened {url}"));
        Ok(())
    }

    fn current_label(&self) -> Option<String> {
        self.current().map(|a| format!("\"{}\" ({})", a.name, a.id))
    }
}

pub fn is_ongoing(activity: &Activity) -> bool {
    activity.logs.iter().any(|l| l.ends_at.is_none())
}

/// When the activity was last worked on: now if ongoing, else the end of its latest log.
pub fn last_active(activity: &Activity, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    activity.logs.iter().map(|l| l.ends_at.unwrap_or(now)).max()
}

/// Returns the value of the first `<prefix>:<value>` tag on the activity.
pub fn tag_value(activity: &Activity, prefix: &str) -> Option<String> {
    let mut tags: Vec<_> = activity.tags.iter().collect();
    tags.sort();
    tags.into_iter()
        .find_map(|t| t.strip_prefix(prefix)?.strip_prefix(':'))
        .map(str::to_string)
}

fn editor() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string())
}

/// Leaves the alternate screen, runs an interactive command, then restores the TUI.
fn run_suspended(terminal: &mut DefaultTerminal, command: &mut Command) -> Result<()> {
    ratatui::restore();
    let status = command.status();
    *terminal = ratatui::init();
    let status = status.with_context(|| format!("failed to run {command:?}"))?;
    anyhow::ensure!(status.success(), "{command:?} exited with {status}");
    Ok(())
}

/// Opens a URL with `$BROWSER` when set, falling back to the platform opener.
fn open_url(url: &str) -> Result<()> {
    let browser = std::env::var("BROWSER").unwrap_or_default();
    let mut candidates = browser_commands(&browser, url);
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    candidates.push(vec![opener.to_string(), url.to_string()]);

    for argv in &candidates {
        let spawned = Command::new(&argv[0])
            .args(&argv[1..])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        if spawned.is_ok() {
            return Ok(());
        }
    }
    anyhow::bail!("failed to open {url}: set $BROWSER or install {opener}")
}

/// Builds one command per `$BROWSER` entry. Follows the common convention:
/// entries are separated by `:`, and `%s` is replaced by the URL (otherwise it is appended).
fn browser_commands(browser: &str, url: &str) -> Vec<Vec<String>> {
    browser
        .split(':')
        .filter_map(|entry| {
            let mut argv: Vec<String> = entry.split_whitespace().map(str::to_string).collect();
            if argv.is_empty() {
                return None;
            }
            if argv.iter().any(|arg| arg.contains("%s")) {
                argv.iter_mut()
                    .for_each(|arg| *arg = arg.replace("%s", url));
            } else {
                argv.push(url.to_string());
            }
            Some(argv)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://acme.atlassian.net/browse/PROJ-1";

    #[test]
    fn browser_commands_appends_url() {
        assert_eq!(browser_commands("firefox", URL), [vec!["firefox", URL]]);
    }

    #[test]
    fn browser_commands_handles_args_placeholder_and_fallbacks() {
        assert_eq!(
            browser_commands("firefox --new-tab %s:chromium", URL),
            [vec!["firefox", "--new-tab", URL], vec!["chromium", URL]]
        );
    }

    #[test]
    fn browser_commands_empty_when_unset() {
        assert!(browser_commands("", URL).is_empty());
    }
}
