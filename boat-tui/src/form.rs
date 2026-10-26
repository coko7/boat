use boat_lib::{models::activity::Activity, repository::Id};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::input::TextInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Name,
    Description,
    Customer,
    Jira,
    Tags,
    StartNow,
}

impl Field {
    pub const ALL: [Field; 6] = [
        Field::Name,
        Field::Description,
        Field::Customer,
        Field::Jira,
        Field::Tags,
        Field::StartNow,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Field::Name => "Name",
            Field::Description => "Description",
            Field::Customer => "Customer",
            Field::Jira => "Jira issue",
            Field::Tags => "Extra tags",
            Field::StartNow => "Start now",
        }
    }

    pub fn placeholder(self) -> &'static str {
        match self {
            Field::Name => "work on boat (required)",
            Field::Description => "optional",
            Field::Customer => "e.g. Acme Corp",
            Field::Jira => "e.g. PROJ-123",
            Field::Tags => "space separated, e.g. task:misc",
            Field::StartNow => "",
        }
    }
}

/// What the form produced once submitted.
pub struct ActivityInput {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub start_now: bool,
}

pub enum FormEvent {
    None,
    Cancel,
    Submit(ActivityInput),
}

#[derive(Debug, Clone)]
pub struct ActivityForm {
    pub name: TextInput,
    pub description: TextInput,
    pub customer: TextInput,
    pub jira: TextInput,
    pub tags: TextInput,
    pub start_now: bool,
    pub focus: Field,
    pub error: Option<String>,
    /// Activity being edited; `None` when creating a new one
    pub editing: Option<Id>,
}

impl Default for ActivityForm {
    fn default() -> Self {
        Self {
            name: TextInput::default(),
            description: TextInput::default(),
            customer: TextInput::default(),
            jira: TextInput::default(),
            tags: TextInput::default(),
            start_now: true,
            focus: Field::Name,
            error: None,
            editing: None,
        }
    }
}

impl ActivityForm {
    /// Pre-fills the form from an existing activity. The first `customer:` and `jira:`
    /// tags go to their dedicated fields, every other tag to the extra tags field.
    pub fn edit(activity: &Activity) -> Self {
        let mut tags: Vec<&String> = activity.tags.iter().collect();
        tags.sort();

        let mut take = |prefix: &str| {
            let index = tags.iter().position(|t| t.starts_with(prefix))?;
            Some(tags.remove(index)[prefix.len()..].to_string())
        };
        let customer = take("customer:").unwrap_or_default();
        let jira = take("jira:").unwrap_or_default();

        Self {
            name: TextInput::new(activity.name.clone()),
            description: TextInput::new(activity.description.clone().unwrap_or_default()),
            customer: TextInput::new(customer),
            jira: TextInput::new(jira),
            tags: TextInput::new(
                tags.into_iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            start_now: false,
            editing: Some(activity.id),
            ..Self::default()
        }
    }

    /// Fields shown in the form; starting only makes sense for a new activity.
    pub fn fields(&self) -> Vec<Field> {
        Field::ALL
            .into_iter()
            .filter(|f| self.editing.is_none() || *f != Field::StartNow)
            .collect()
    }

    /// Text input backing the field; `None` for the start now checkbox.
    pub fn input(&self, field: Field) -> Option<&TextInput> {
        match field {
            Field::Name => Some(&self.name),
            Field::Description => Some(&self.description),
            Field::Customer => Some(&self.customer),
            Field::Jira => Some(&self.jira),
            Field::Tags => Some(&self.tags),
            Field::StartNow => None,
        }
    }

    fn input_mut(&mut self, field: Field) -> Option<&mut TextInput> {
        match field {
            Field::Name => Some(&mut self.name),
            Field::Description => Some(&mut self.description),
            Field::Customer => Some(&mut self.customer),
            Field::Jira => Some(&mut self.jira),
            Field::Tags => Some(&mut self.tags),
            Field::StartNow => None,
        }
    }

    fn move_focus(&mut self, delta: isize) {
        let fields = self.fields();
        let len = fields.len() as isize;
        let index = fields.iter().position(|f| *f == self.focus).unwrap_or(0) as isize;
        self.focus = fields[(index + delta).rem_euclid(len) as usize];
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> FormEvent {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        self.error = None;
        match key.code {
            KeyCode::Esc => return FormEvent::Cancel,
            KeyCode::Enter => return self.submit(),
            KeyCode::Char('s') if ctrl => return self.submit(),
            KeyCode::Tab | KeyCode::Down => self.move_focus(1),
            KeyCode::BackTab | KeyCode::Up => self.move_focus(-1),
            KeyCode::Char(' ') if self.focus == Field::StartNow => {
                self.start_now = !self.start_now;
            }
            _ => {
                if let Some(input) = self.input_mut(self.focus) {
                    input.handle_key(key);
                }
            }
        }
        FormEvent::None
    }

    fn submit(&mut self) -> FormEvent {
        let name = self.name.value().trim();
        if name.is_empty() {
            self.error = Some("Name is required".to_string());
            self.focus = Field::Name;
            return FormEvent::None;
        }

        let description =
            Some(self.description.value().trim().to_string()).filter(|d| !d.is_empty());

        let mut tags = vec![];
        let customer = self.customer.value().trim();
        if !customer.is_empty() {
            tags.push(format!("customer:{}", slugify(customer)));
        }
        let jira = self.jira.value().trim();
        if !jira.is_empty() {
            tags.push(format!("jira:{jira}"));
        }
        for tag in self.tags.value().split_whitespace() {
            if !tags.iter().any(|t| t == tag) {
                tags.push(tag.to_string());
            }
        }

        FormEvent::Submit(ActivityInput {
            name: name.to_string(),
            description,
            tags,
            start_now: self.start_now,
        })
    }
}

/// `Acme Corp` -> `acme-corp`, matching boat-fleet's customer tags.
fn slugify(s: &str) -> String {
    s.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_str(form: &mut ActivityForm, s: &str) {
        for c in s.chars() {
            form.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
    }

    #[test]
    fn submit_requires_name() {
        let mut form = ActivityForm {
            focus: Field::Jira,
            ..Default::default()
        };
        assert!(matches!(
            form.handle_key(KeyEvent::from(KeyCode::Enter)),
            FormEvent::None
        ));
        assert!(form.error.is_some());
        assert_eq!(form.focus, Field::Name);
    }

    #[test]
    fn submit_builds_tags_from_fields() {
        let mut form = ActivityForm::default();
        type_str(&mut form, "fix login");
        form.handle_key(KeyEvent::from(KeyCode::Tab));
        form.handle_key(KeyEvent::from(KeyCode::Tab));
        type_str(&mut form, "Acme  Corp");
        form.handle_key(KeyEvent::from(KeyCode::Tab));
        type_str(&mut form, "PROJ-1");
        form.handle_key(KeyEvent::from(KeyCode::Tab));
        type_str(&mut form, "task:bug jira:PROJ-1");

        let FormEvent::Submit(input) = form.handle_key(KeyEvent::from(KeyCode::Enter)) else {
            panic!("expected submit");
        };
        assert_eq!(input.name, "fix login");
        assert_eq!(input.description, None);
        assert_eq!(
            input.tags,
            ["customer:acme-corp", "jira:PROJ-1", "task:bug"]
        );
        assert!(input.start_now);
    }

    #[test]
    fn space_toggles_start_now_only_on_that_field() {
        let mut form = ActivityForm::default();
        form.handle_key(KeyEvent::from(KeyCode::BackTab));
        assert_eq!(form.focus, Field::StartNow);
        form.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert!(!form.start_now);
    }

    #[test]
    fn edit_prefills_fields_and_round_trips_tags() {
        let activity = Activity {
            id: 7,
            name: "fix login".to_string(),
            description: Some("users locked out".to_string()),
            tags: ["customer:acme-corp", "jira:PROJ-1", "task:bug", "urgent"]
                .map(String::from)
                .into(),
            logs: vec![],
        };

        let mut form = ActivityForm::edit(&activity);
        assert_eq!(form.editing, Some(7));
        assert_eq!(form.customer.value(), "acme-corp");
        assert_eq!(form.jira.value(), "PROJ-1");
        assert_eq!(form.tags.value(), "task:bug urgent");
        assert!(!form.fields().contains(&Field::StartNow));

        let FormEvent::Submit(input) = form.handle_key(KeyEvent::from(KeyCode::Enter)) else {
            panic!("expected submit");
        };
        assert_eq!(input.description.as_deref(), Some("users locked out"));
        assert_eq!(
            input.tags,
            ["customer:acme-corp", "jira:PROJ-1", "task:bug", "urgent"]
        );
    }
}
