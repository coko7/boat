use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Single-line text input with a cursor and readline-style word editing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    value: String,
    /// Cursor position, in chars
    cursor: usize,
}

impl TextInput {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self { value, cursor }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Cursor position, in chars from the start of the value.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Applies an editing key. Returns `false` when the key is not an editing key,
    /// so the caller can handle it (enter, esc, tab...).
    ///
    /// Ctrl-backspace is often sent as ctrl-h, and alt-backspace as backspace with alt,
    /// so all three delete the previous word like ctrl-w.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let word = ctrl || key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Left if word => self.cursor = self.prev_word(),
            KeyCode::Right if word => self.cursor = self.next_word(),
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.len()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.len(),
            KeyCode::Char('a') if ctrl => self.cursor = 0,
            KeyCode::Char('e') if ctrl => self.cursor = self.len(),
            KeyCode::Backspace if word => self.delete_to(self.prev_word()),
            KeyCode::Char('w' | 'h') if ctrl => self.delete_to(self.prev_word()),
            KeyCode::Char('u') if ctrl => self.delete_to(0),
            KeyCode::Char('k') if ctrl => self.delete_to(self.len()),
            KeyCode::Delete if word => self.delete_to(self.next_word()),
            KeyCode::Backspace => self.delete_to(self.cursor.saturating_sub(1)),
            KeyCode::Delete => self.delete_to((self.cursor + 1).min(self.len())),
            KeyCode::Char(c) if !ctrl => {
                let at = self.byte_index(self.cursor);
                self.value.insert(at, c);
                self.cursor += 1;
            }
            _ => return false,
        }
        true
    }

    fn len(&self) -> usize {
        self.value.chars().count()
    }

    fn byte_index(&self, cursor: usize) -> usize {
        self.value
            .char_indices()
            .nth(cursor)
            .map_or(self.value.len(), |(i, _)| i)
    }

    /// Removes the chars between the cursor and `to` (either side), leaving the cursor at the start.
    fn delete_to(&mut self, to: usize) {
        let (start, end) = (self.cursor.min(to), self.cursor.max(to));
        let range = self.byte_index(start)..self.byte_index(end);
        self.value.replace_range(range, "");
        self.cursor = start;
    }

    /// Start of the word before the cursor, skipping whitespace first.
    fn prev_word(&self) -> usize {
        let chars: Vec<char> = self.value.chars().collect();
        let mut i = self.cursor;
        while i > 0 && chars[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        i
    }

    /// End of the word after the cursor, skipping whitespace first.
    fn next_word(&self) -> usize {
        let chars: Vec<char> = self.value.chars().collect();
        let mut i = self.cursor;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(input: &mut TextInput, code: KeyCode, modifiers: KeyModifiers) {
        assert!(input.handle_key(KeyEvent::new(code, modifiers)));
    }

    #[test]
    fn moves_and_deletes_by_word() {
        let mut input = TextInput::new("fix the löгin bug");
        press(&mut input, KeyCode::Left, KeyModifiers::CONTROL);
        assert_eq!(input.cursor(), 14);
        press(&mut input, KeyCode::Left, KeyModifiers::CONTROL);
        assert_eq!(input.cursor(), 8);

        press(&mut input, KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!(input.value(), "fix löгin bug");
        assert_eq!(input.cursor(), 4);

        press(&mut input, KeyCode::Right, KeyModifiers::CONTROL);
        assert_eq!(input.cursor(), 9);
        press(&mut input, KeyCode::Backspace, KeyModifiers::CONTROL);
        assert_eq!(input.value(), "fix  bug");

        press(&mut input, KeyCode::Char('h'), KeyModifiers::CONTROL);
        assert_eq!(input.value(), " bug");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn inserts_at_cursor() {
        let mut input = TextInput::new("ac");
        press(&mut input, KeyCode::Left, KeyModifiers::NONE);
        press(&mut input, KeyCode::Char('b'), KeyModifiers::NONE);
        assert_eq!(input.value(), "abc");
        press(&mut input, KeyCode::Home, KeyModifiers::NONE);
        press(&mut input, KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(input.value(), "bc");
        assert!(!input.handle_key(KeyEvent::from(KeyCode::Enter)));
    }
}
