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

    /// The part of the value that fits in `width` columns, scrolled so the cursor stays
    /// visible, and the cursor offset within it.
    pub fn view(&self, width: usize) -> (String, usize) {
        let width = width.max(1);
        // keep one column for the cursor when it sits past the last char
        let scroll = self.cursor.saturating_sub(width - 1);
        let visible = self.value.chars().skip(scroll).take(width).collect();
        (visible, self.cursor - scroll)
    }

    /// The value wrapped on word boundaries into lines of at most `width` columns
    /// (words longer than that are split), and the cursor position as `(row, column)`.
    pub fn wrap(&self, width: usize) -> (Vec<String>, (usize, usize)) {
        let width = width.max(1);
        let chars: Vec<char> = self.value.chars().collect();

        // start of each line, in chars
        let mut starts = vec![0];
        let mut start = 0;
        while chars.len() - start > width {
            let end = (start + 1..=start + width)
                .rev()
                .find(|&i| chars[i - 1].is_whitespace())
                .unwrap_or(start + width);
            starts.push(end);
            start = end;
        }
        // a full last line leaves no room for the cursor after it
        if self.cursor == chars.len() && chars.len() - start == width {
            starts.push(chars.len());
        }

        let lines = starts
            .iter()
            .enumerate()
            .map(|(row, &start)| {
                let end = starts.get(row + 1).copied().unwrap_or(chars.len());
                chars[start..end].iter().collect()
            })
            .collect();
        let row = starts.iter().rposition(|&s| s <= self.cursor).unwrap_or(0);
        (lines, (row, self.cursor - starts[row]))
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
        assert_eq!(input.cursor, 14);
        press(&mut input, KeyCode::Left, KeyModifiers::CONTROL);
        assert_eq!(input.cursor, 8);

        press(&mut input, KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!(input.value(), "fix löгin bug");
        assert_eq!(input.cursor, 4);

        press(&mut input, KeyCode::Right, KeyModifiers::CONTROL);
        assert_eq!(input.cursor, 9);
        press(&mut input, KeyCode::Backspace, KeyModifiers::CONTROL);
        assert_eq!(input.value(), "fix  bug");

        press(&mut input, KeyCode::Char('h'), KeyModifiers::CONTROL);
        assert_eq!(input.value(), " bug");
        assert_eq!(input.cursor, 0);
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

    #[test]
    fn wrap_breaks_on_words_and_tracks_cursor() {
        let mut input = TextInput::new("fix the login bug");
        assert_eq!(
            input.wrap(8),
            (
                vec!["fix the ".into(), "login ".into(), "bug".into()],
                (2, 3)
            )
        );
        press(&mut input, KeyCode::Left, KeyModifiers::CONTROL);
        assert_eq!(input.wrap(8).1, (2, 0));

        let input = TextInput::new("abcdefgh");
        assert_eq!(
            input.wrap(4),
            (vec!["abcd".into(), "efgh".into(), "".into()], (2, 0))
        );
    }

    #[test]
    fn view_scrolls_to_keep_cursor_visible() {
        let mut input = TextInput::new("abcdefgh");
        assert_eq!(input.view(5), ("efgh".to_string(), 4));
        assert_eq!(input.view(20), ("abcdefgh".to_string(), 8));
        press(&mut input, KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(input.view(5), ("abcde".to_string(), 0));
    }
}
