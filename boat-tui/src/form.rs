use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
pub struct NewActivityInput {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub start_now: bool,
}

pub enum FormEvent {
    None,
    Cancel,
    Submit(NewActivityInput),
}

#[derive(Debug, Clone)]
pub struct NewActivityForm {
    pub name: String,
    pub description: String,
    pub customer: String,
    pub jira: String,
    pub tags: String,
    pub start_now: bool,
    pub focus: Field,
    pub error: Option<String>,
}

impl Default for NewActivityForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            customer: String::new(),
            jira: String::new(),
            tags: String::new(),
            start_now: true,
            focus: Field::Name,
            error: None,
        }
    }
}

impl NewActivityForm {
    pub fn value(&self, field: Field) -> &str {
        match field {
            Field::Name => &self.name,
            Field::Description => &self.description,
            Field::Customer => &self.customer,
            Field::Jira => &self.jira,
            Field::Tags => &self.tags,
            Field::StartNow => "",
        }
    }

    fn value_mut(&mut self, field: Field) -> Option<&mut String> {
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
        let len = Field::ALL.len() as isize;
        let index = Field::ALL
            .iter()
            .position(|f| *f == self.focus)
            .unwrap_or(0) as isize;
        self.focus = Field::ALL[(index + delta).rem_euclid(len) as usize];
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
            KeyCode::Char('u') if ctrl => {
                if let Some(value) = self.value_mut(self.focus) {
                    value.clear();
                }
            }
            KeyCode::Backspace => {
                if let Some(value) = self.value_mut(self.focus) {
                    value.pop();
                }
            }
            KeyCode::Char(c) if !ctrl => {
                if let Some(value) = self.value_mut(self.focus) {
                    value.push(c);
                }
            }
            _ => {}
        }
        FormEvent::None
    }

    fn submit(&mut self) -> FormEvent {
        let name = self.name.trim();
        if name.is_empty() {
            self.error = Some("Name is required".to_string());
            self.focus = Field::Name;
            return FormEvent::None;
        }

        let description = Some(self.description.trim().to_string()).filter(|d| !d.is_empty());

        let mut tags = vec![];
        let customer = self.customer.trim();
        if !customer.is_empty() {
            tags.push(format!("customer:{}", slugify(customer)));
        }
        let jira = self.jira.trim();
        if !jira.is_empty() {
            tags.push(format!("jira:{jira}"));
        }
        for tag in self.tags.split_whitespace() {
            if !tags.iter().any(|t| t == tag) {
                tags.push(tag.to_string());
            }
        }

        FormEvent::Submit(NewActivityInput {
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

    fn type_str(form: &mut NewActivityForm, s: &str) {
        for c in s.chars() {
            form.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
    }

    #[test]
    fn submit_requires_name() {
        let mut form = NewActivityForm {
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
        let mut form = NewActivityForm::default();
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
        let mut form = NewActivityForm::default();
        form.handle_key(KeyEvent::from(KeyCode::BackTab));
        assert_eq!(form.focus, Field::StartNow);
        form.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert!(!form.start_now);
    }
}
