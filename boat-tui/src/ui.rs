use boat_lib::models::activity::Activity;
use chrono::{DateTime, Datelike, Local, TimeDelta, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Clear, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
        canvas::{Canvas, Points},
    },
};

use crate::{
    app::{Action, App, Mode, Picker, is_ongoing, last_active, tag_value},
    form::{Field, NewActivityForm},
    report::DayReport,
};

const ACCENT: Color = Color::Cyan;
const RUNNING: Color = Color::Green;
const LEFT_PANE_PERCENT: u16 = 50;
const TODAY_PANE_PERCENT: u16 = 45;
const SLICE_COLORS: [Color; 8] = [
    Color::Blue,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::Red,
    Color::Cyan,
    Color::LightBlue,
    Color::LightMagenta,
];
const OTHERS_COLOR: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let [list, side] = Layout::horizontal([
        Constraint::Percentage(LEFT_PANE_PERCENT),
        Constraint::Fill(1),
    ])
    .areas(body);
    let [preview, today] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Percentage(TODAY_PANE_PERCENT),
    ])
    .areas(side);

    draw_header(frame, header, app);
    draw_activities(frame, list, app);
    draw_preview(frame, preview, app);
    draw_today(frame, today, app);
    draw_footer(frame, footer, app);

    match &app.mode {
        Mode::Browse | Mode::Filter => {}
        Mode::Input {
            prompt,
            placeholder,
            value,
            ..
        } => draw_input(frame, prompt, placeholder, value),
        Mode::NewForm(form) => draw_new_form(frame, form),
        Mode::MeetingPicker(picker) => draw_picker(frame, picker),
        Mode::ConfirmCancel { message } => draw_confirm(frame, message),
        Mode::Help => draw_help(frame),
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let clock = Local::now().format("%a %d %b  %H:%M:%S").to_string();
    let [title, current, time] = Layout::horizontal([
        Constraint::Length(14),
        Constraint::Fill(1),
        Constraint::Length(clock.len() as u16 + 1),
    ])
    .areas(area);

    frame.render_widget(
        Line::from(vec![" ⛵ ".into(), "Boat Fleet".bold().fg(ACCENT)]),
        title,
    );

    let current_line = match app.current() {
        Some(activity) => {
            let elapsed = running_since(activity)
                .map(|s| format_duration(Utc::now() - s))
                .unwrap_or_default();
            Line::from(vec![
                Span::styled("● ", Style::new().fg(RUNNING)),
                Span::raw(activity.name.clone()).bold(),
                Span::styled(format!("  {elapsed}"), Style::new().fg(RUNNING)),
            ])
        }
        None => Line::from("○ idle").dim(),
    };
    frame.render_widget(current_line.centered(), current);
    frame.render_widget(Line::from(clock).dim(), time);
}

fn draw_activities(frame: &mut Frame, area: Rect, app: &mut App) {
    let now = Utc::now();
    let total = app.activities.len();
    let visible = app.visible();
    let count = if visible.len() == total {
        format!(" {total} ")
    } else {
        format!(" {}/{total} ", visible.len())
    };

    let block = titled_block(" Activities ").title_bottom(Line::from(count).right_aligned());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let show_filter = matches!(app.mode, Mode::Filter) || !app.filter.is_empty();
    let [filter_area, list_area] = Layout::vertical([
        Constraint::Length(if show_filter { 1 } else { 0 }),
        Constraint::Fill(1),
    ])
    .areas(inner);

    // marker (2) + id (5) + age (6) + highlight symbol (1)
    let name_width = (list_area.width as usize).saturating_sub(14);
    let items: Vec<ListItem> = visible
        .iter()
        .map(|activity| {
            let ongoing = is_ongoing(activity);
            let marker = if ongoing {
                Span::styled("● ", Style::new().fg(RUNNING))
            } else {
                Span::raw("  ")
            };
            let age = match last_active(activity, now) {
                _ if ongoing => "now".to_string(),
                Some(at) => format_age(now - at),
                None => "–".to_string(),
            };
            let line = Line::from(vec![
                marker,
                Span::raw(format!("{:>4} ", activity.id)).dim(),
                Span::raw(format!(
                    "{:<name_width$}",
                    truncate(&activity.name, name_width)
                )),
                Span::raw(format!("{age:>6}")).dim(),
            ]);
            let item = ListItem::new(line);
            if ongoing {
                item.fg(RUNNING).bold()
            } else {
                item
            }
        })
        .collect();
    drop(visible);

    if show_filter {
        let filter = Line::from(vec![
            Span::styled("/", Style::new().fg(ACCENT)),
            Span::raw(app.filter.clone()),
        ]);
        frame.render_widget(filter, filter_area);
        if matches!(app.mode, Mode::Filter) {
            let x = filter_area.x + 1 + app.filter.chars().count() as u16;
            frame.set_cursor_position((x, filter_area.y));
        }
    }

    if items.is_empty() {
        let hint = if app.filter.is_empty() {
            "no activities yet · press 'n' to create one"
        } else {
            "no match"
        };
        frame.render_widget(Paragraph::new(hint).dim().centered(), list_area);
        return;
    }

    let list = List::new(items)
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("›");
    frame.render_stateful_widget(list, list_area, &mut app.list_state);
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App) {
    let Some(activity) = app.selected() else {
        frame.render_widget(titled_block(" Preview "), area);
        return;
    };

    let block = titled_block(Line::from(vec![
        Span::raw(format!(" #{} ", activity.id)).dim(),
        Span::raw(format!("{} ", activity.name)).bold(),
    ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let now = Utc::now();
    let mut lines = vec![status_line(activity, now)];
    if let Some(desc) = activity.description.as_deref().filter(|d| !d.is_empty()) {
        lines.push(Line::from(desc.to_string()).italic());
    }
    if !activity.tags.is_empty() {
        let mut tags: Vec<_> = activity.tags.iter().collect();
        tags.sort();
        lines.push(Line::from(
            tags.into_iter()
                .flat_map(|t| [Span::styled(format!(" {t} "), tag_style(t)), Span::raw(" ")])
                .collect::<Vec<_>>(),
        ));
    }
    if tag_value(activity, "jira").is_some() {
        lines.push(Line::from("press J to open in jira").dim());
    }
    lines.push(Line::default());
    lines.extend(stats_lines(activity, now));
    lines.push(Line::default());

    let summary_height = lines.len() as u16;
    let [summary, sessions] =
        Layout::vertical([Constraint::Length(summary_height), Constraint::Fill(1)]).areas(inner);

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        summary.inner(Margin::new(1, 0)),
    );
    draw_sessions(frame, sessions, activity, now);
}

fn status_line(activity: &Activity, now: DateTime<Utc>) -> Line<'static> {
    match running_since(activity) {
        Some(start) => Line::from(vec![
            Span::styled("● running ", Style::new().fg(RUNNING).bold()),
            Span::styled(format_duration(now - start), Style::new().fg(RUNNING)),
            Span::raw(format!(
                "  since {}",
                start.with_timezone(&Local).format("%H:%M")
            ))
            .dim(),
        ]),
        None => {
            let last = match last_active(activity, now).map(|at| format_age(now - at)) {
                Some(age) if age == "now" => "last active just now".to_string(),
                Some(age) => format!("last active {age} ago"),
                None => "never started".to_string(),
            };
            Line::from(vec!["○ idle ".dim(), Span::raw(format!(" {last}")).dim()])
        }
    }
}

fn stats_lines(activity: &Activity, now: DateTime<Utc>) -> Vec<Line<'static>> {
    let today = Local::now().date_naive();
    let week_start = today - TimeDelta::days(today.weekday().num_days_from_monday() as i64);

    let (mut total, mut today_total, mut week_total) =
        (TimeDelta::zero(), TimeDelta::zero(), TimeDelta::zero());
    for log in &activity.logs {
        let duration = log.ends_at.unwrap_or(now) - log.starts_at;
        let day = log.starts_at.with_timezone(&Local).date_naive();
        total += duration;
        if day == today {
            today_total += duration;
        }
        if day >= week_start {
            week_total += duration;
        }
    }

    let first = activity
        .logs
        .iter()
        .map(|l| l.starts_at)
        .min()
        .map(|d| d.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "–".to_string());

    let stat = |label: &str, value: String| {
        Line::from(vec![
            Span::raw(format!("{label:<10}")).dim(),
            Span::styled(value, Style::new().fg(ACCENT)),
        ])
    };
    vec![
        stat("today", format_duration(today_total)),
        stat("this week", format_duration(week_total)),
        stat("total", format_duration(total)),
        stat("sessions", activity.logs.len().to_string()),
        stat("first", first),
    ]
}

fn draw_sessions(frame: &mut Frame, area: Rect, activity: &Activity, now: DateTime<Utc>) {
    let mut logs: Vec<_> = activity.logs.iter().collect();
    logs.sort_by_key(|l| std::cmp::Reverse(l.starts_at));

    let rows: Vec<Row> = logs
        .into_iter()
        .map(|log| {
            let start = log.starts_at.with_timezone(&Local);
            let end = log
                .ends_at
                .map(|e| e.with_timezone(&Local).format("%H:%M").to_string())
                .unwrap_or_else(|| "now".to_string());
            let row = Row::new(vec![
                Span::raw(start.format("%a %Y-%m-%d").to_string()).dim(),
                Span::raw(format!("{}–{end}", start.format("%H:%M"))),
                Span::styled(
                    format_duration(log.ends_at.unwrap_or(now) - log.starts_at),
                    Style::new().fg(ACCENT),
                ),
            ]);
            if log.ends_at.is_none() {
                row.fg(RUNNING)
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Length(11),
        ],
    )
    .header(Row::new(["date", "time", "duration"]).bold().underlined())
    .column_spacing(2);
    frame.render_widget(table, area.inner(Margin::new(1, 0)));
}

/// One row of the today report, possibly several small activities lumped together.
struct Slice {
    label: String,
    duration: TimeDelta,
    share: f64,
    color: Color,
    bold: bool,
    ongoing: bool,
}

fn draw_today(frame: &mut Frame, area: Rect, app: &App) {
    let report = DayReport::today(&app.activities, Utc::now());
    let block = titled_block(Line::from(vec![
        Span::raw(" Today ").bold(),
        Span::styled(
            format!("{} ", format_duration(report.total)),
            Style::new().fg(ACCENT),
        ),
    ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if report.entries.is_empty() {
        frame.render_widget(
            Paragraph::new("nothing tracked today").dim().centered(),
            inner.inner(Margin::new(0, inner.height.saturating_sub(1) / 2)),
        );
        return;
    }

    // Keep a header and a total row; lump whatever does not fit (or has no color left) into "others"
    let max_rows = (inner.height.saturating_sub(2) as usize).clamp(1, SLICE_COLORS.len());
    let selected_id = app.selected().map(|a| a.id);
    let lump = report.entries.len() > max_rows;
    let shown = if lump { max_rows - 1 } else { max_rows };

    let mut slices: Vec<Slice> = report
        .entries
        .iter()
        .take(shown)
        .enumerate()
        .map(|(i, entry)| Slice {
            label: entry.name.clone(),
            duration: entry.duration,
            share: report.share(entry),
            color: SLICE_COLORS[i],
            bold: Some(entry.activity_id) == selected_id,
            ongoing: entry.ongoing,
        })
        .collect();
    if lump {
        let rest = &report.entries[shown..];
        let duration = rest.iter().map(|e| e.duration).sum();
        slices.push(Slice {
            label: format!("{} others", rest.len()),
            duration,
            share: rest.iter().map(|e| report.share(e)).sum(),
            color: OTHERS_COLOR,
            bold: false,
            ongoing: rest.iter().any(|e| e.ongoing),
        });
    }

    // Terminal cells are about twice as tall as wide, so a round pie needs width = 2 * height
    let pie_width = (inner.height * 2).min(inner.width / 2);
    let [table_area, pie_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(pie_width)])
            .spacing(1)
            .areas(inner.inner(Margin::new(1, 0)));

    draw_today_table(frame, table_area, &slices, report.total);
    draw_pie(frame, pie_area, &slices);
}

fn draw_today_table(frame: &mut Frame, area: Rect, slices: &[Slice], total: TimeDelta) {
    let rows = slices.iter().map(|slice| {
        let style = if slice.bold {
            Style::new().bold().underlined()
        } else {
            Style::new()
        };
        let marker = if slice.ongoing { "● " } else { "" };
        Row::new(vec![
            Span::styled("██", Style::new().fg(slice.color)),
            Span::styled(format!("{marker}{}", slice.label), style),
            Span::styled(format_duration(slice.duration), Style::new().fg(ACCENT)),
            Span::raw(format!("{:>3.0}%", slice.share * 100.0)).dim(),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(4),
        ],
    )
    .header(Row::new(["", "activity", "time", "%"]).bold().underlined())
    .footer(Row::new(vec![
        Span::raw(""),
        Span::raw("total").bold(),
        Span::styled(format_duration(total), Style::new().fg(ACCENT).bold()),
        Span::raw("100%").dim(),
    ]))
    .column_spacing(1);
    frame.render_widget(table, area);
}

/// Draws a pie chart with half-block pixels, which are roughly square on most terminals.
fn draw_pie(frame: &mut Frame, area: Rect, slices: &[Slice]) {
    let (w, h) = (area.width as f64, area.height as f64);
    // One x unit per cell column, one y unit per half cell row
    let radius = (w / 2.0).min(h) - 0.5;

    let mut points: Vec<Vec<(f64, f64)>> = vec![vec![]; slices.len()];
    let mut y = -h + 0.5;
    while y < h {
        let mut x = -w / 2.0 + 0.5;
        while x < w / 2.0 {
            if x * x + y * y <= radius * radius {
                // Clockwise from 12 o'clock, in [0, 1)
                let turn = x.atan2(y).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU;
                let mut cumulative = 0.0;
                let index = slices
                    .iter()
                    .position(|s| {
                        cumulative += s.share;
                        turn < cumulative
                    })
                    .unwrap_or(slices.len() - 1);
                points[index].push((x, y));
            }
            x += 1.0;
        }
        y += 1.0;
    }

    let canvas = Canvas::default()
        .marker(Marker::HalfBlock)
        .x_bounds([-w / 2.0, w / 2.0])
        .y_bounds([-h, h])
        .paint(|ctx| {
            for (slice, coords) in slices.iter().zip(&points) {
                ctx.draw(&Points {
                    coords,
                    color: slice.color,
                });
            }
        });
    frame.render_widget(canvas, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let line = match (&app.status, &app.mode) {
        (Some(status), _) if status.is_error => Line::from(format!(" ⚠ {}", status.message)).red(),
        (Some(status), _) => Line::from(format!(" ℹ {}", status.message)).green(),
        (None, Mode::Browse) => {
            let mut spans = vec![key_span("↑↓"), Span::raw(" move  ").dim()];
            for action in Action::ALL.into_iter().filter(|a| a.is_important()) {
                spans.push(key_span(action.key_hint()));
                spans.push(Span::raw(format!(" {}  ", action.label())).dim());
            }
            Line::from(spans)
        }
        (None, Mode::Filter) => hints(&[("type", "filter"), ("⏎", "keep"), ("esc", "clear")]),
        (None, Mode::Input { .. }) => hints(&[("⏎", "submit"), ("esc", "cancel")]),
        (None, Mode::NewForm(_)) => hints(&[
            ("tab/↓", "next field"),
            ("shift-tab/↑", "previous"),
            ("space", "toggle"),
            ("ctrl-u", "clear"),
            ("⏎", "create"),
            ("esc", "cancel"),
        ]),
        (None, Mode::MeetingPicker(_)) => {
            hints(&[("↑↓", "move"), ("⏎", "start"), ("esc", "cancel")])
        }
        (None, Mode::ConfirmCancel { .. }) => hints(&[("y", "confirm"), ("n", "abort")]),
        (None, Mode::Help) => hints(&[("any key", "close")]),
    };
    frame.render_widget(line, area);
}

fn hints(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans = vec![];
    for (key, label) in pairs {
        spans.push(key_span(key));
        spans.push(Span::raw(format!(" {label}  ")).dim());
    }
    Line::from(spans)
}

fn key_span(key: &str) -> Span<'static> {
    Span::styled(format!(" {key}"), Style::new().fg(ACCENT).bold())
}

fn draw_input(frame: &mut Frame, prompt: &str, placeholder: &str, value: &str) {
    let area = popup(frame.area(), 60, 3);
    frame.render_widget(Clear, area);

    let text = if value.is_empty() {
        Line::from(placeholder.to_string()).dim()
    } else {
        Line::from(value.to_string())
    };
    let block = titled_block(format!(" {prompt} "));
    let inner = block.inner(area);
    frame.render_widget(Paragraph::new(text).block(block), area);
    frame.set_cursor_position((inner.x + value.chars().count() as u16, inner.y));
}

fn draw_new_form(frame: &mut Frame, form: &NewActivityForm) {
    const LABEL_WIDTH: usize = 13;

    let area = popup(frame.area(), 70, Field::ALL.len() as u16 + 6);
    frame.render_widget(Clear, area);

    let block = titled_block(" New Activity ");
    let inner = block.inner(area).inner(Margin::new(1, 1));
    frame.render_widget(block, area);

    let mut cursor = None;
    let mut lines: Vec<Line> = Field::ALL
        .into_iter()
        .enumerate()
        .map(|(row, field)| {
            let focused = form.focus == field;
            let required = if field == Field::Name { "*" } else { " " };
            let label_style = if focused {
                Style::new().fg(ACCENT).bold()
            } else {
                Style::new().dim()
            };
            let mut spans = vec![
                Span::styled(if focused { "› " } else { "  " }, label_style),
                Span::styled(
                    format!("{:<LABEL_WIDTH$}", format!("{}{required}", field.label())),
                    label_style,
                ),
            ];

            if field == Field::StartNow {
                let check = if form.start_now { "[x]" } else { "[ ]" };
                spans.push(Span::styled(check, label_style));
                return Line::from(spans);
            }

            let value = form.value(field);
            if focused {
                let x = inner.x + (2 + LABEL_WIDTH + value.chars().count()) as u16;
                cursor = Some((x, inner.y + row as u16));
            }
            if value.is_empty() {
                spans.push(Span::raw(field.placeholder()).dim().italic());
            } else {
                spans.push(Span::raw(value.to_string()));
            }
            Line::from(spans)
        })
        .collect();

    lines.push(Line::default());
    lines.push(match &form.error {
        Some(error) => Line::from(format!("⚠ {error}")).red(),
        None => Line::from("customer and jira become customer:<slug> and jira:<issue> tags").dim(),
    });

    frame.render_widget(Paragraph::new(lines), inner);
    if let Some(position) = cursor {
        frame.set_cursor_position(position);
    }
}

fn draw_picker(frame: &mut Frame, picker: &Picker) {
    let area = popup(frame.area(), 40, picker.items.len() as u16 + 2);
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = picker
        .items
        .iter()
        .map(|item| ListItem::new(item.label.clone()))
        .collect();
    let list = List::new(items)
        .block(titled_block(picker.title.clone()))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("› ");
    let mut state = ListState::default().with_selected(Some(picker.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_confirm(frame: &mut Frame, message: &str) {
    let area = popup(frame.area(), 50, 5);
    frame.render_widget(Clear, area);
    let text = vec![
        Line::from(message.to_string()).bold().centered(),
        Line::default(),
        Line::from("[y]es / [n]o").dim().centered(),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .block(titled_block(" Confirm ").border_style(Style::new().yellow())),
        area,
    );
}

fn draw_help(frame: &mut Frame) {
    let navigation = [
        ("↑↓ j/k", "move selection"),
        ("g/G", "first/last activity"),
        ("PgUp/PgDn", "scroll by 10"),
        ("esc", "clear filter"),
        ("ctrl-c", "quit"),
    ];

    let mut rows: Vec<Row> = Action::ALL
        .into_iter()
        .map(|a| {
            Row::new(vec![
                Span::styled(a.key_hint(), Style::new().fg(ACCENT).bold()),
                Span::raw(a.label()).bold(),
                Span::raw(a.description()).dim(),
            ])
        })
        .collect();
    rows.push(Row::new(["", "", ""]));
    rows.extend(navigation.into_iter().map(|(key, desc)| {
        Row::new(vec![
            Span::styled(key, Style::new().fg(ACCENT).bold()),
            Span::raw(""),
            Span::raw(desc).dim(),
        ])
    }));

    let area = popup(frame.area(), 70, rows.len() as u16 + 2);
    frame.render_widget(Clear, area);
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Fill(1),
        ],
    )
    .block(titled_block(" Keybinds "))
    .column_spacing(2);
    frame.render_widget(table, area);
}

fn titled_block<'a>(title: impl Into<Line<'a>>) -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(title.into().bold())
}

fn popup(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn running_since(activity: &Activity) -> Option<DateTime<Utc>> {
    activity
        .logs
        .iter()
        .find(|l| l.ends_at.is_none())
        .map(|l| l.starts_at)
}

fn tag_style(tag: &str) -> Style {
    let color = match tag.split(':').next().unwrap_or_default() {
        "jira" => Color::Blue,
        "customer" => Color::Magenta,
        "meeting" => Color::Yellow,
        "task" => Color::Green,
        _ => Color::Gray,
    };
    Style::new().fg(Color::Black).bg(color)
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn format_duration(duration: TimeDelta) -> String {
    let secs = duration.num_seconds().max(0);
    format!(
        "{}h {:02}m {:02}s",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Compact relative age, e.g. `5m`, `3h`, `12d`, `4mo`.
fn format_age(age: TimeDelta) -> String {
    let mins = age.num_minutes().max(0);
    match mins {
        0 => "now".to_string(),
        m if m < 60 => format!("{m}m"),
        m if m < 60 * 24 => format!("{}h", m / 60),
        m if m < 60 * 24 * 60 => format!("{}d", m / (60 * 24)),
        m => format!("{}mo", m / (60 * 24 * 30)),
    }
}
