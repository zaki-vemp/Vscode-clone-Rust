//! Line-oriented text buffer with a "last saved" snapshot.
//!
//! The buffer keeps the live (possibly unsaved) lines plus the exact lines
//! that were last written to disk. The diff view compares the two, so the
//! editor always knows what changed since the last save without any
//! external tooling.

/// Editable text plus its last-saved snapshot.
#[derive(Clone, Debug, Default)]
pub struct TextBuffer {
    lines: Vec<String>,
    saved: Vec<String>,
}

impl TextBuffer {
    /// Build a buffer whose saved state equals the given text (e.g. after loading a file).
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = split_lines(text);
        Self {
            saved: lines.clone(),
            lines,
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn saved_lines(&self) -> &[String] {
        &self.saved
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Full current text with `\n` separators (no trailing newline added).
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn is_dirty(&self) -> bool {
        self.lines != self.saved
    }

    /// Replace the whole live content (used when the UI editor commits a change).
    pub fn set_text(&mut self, text: &str) {
        self.lines = split_lines(text);
    }

    /// Mark the current content as saved (after a successful write to disk).
    pub fn mark_saved(&mut self) {
        self.saved = self.lines.clone();
    }

    /// Revert live content to the last saved snapshot.
    pub fn revert(&mut self) {
        self.lines = self.saved.clone();
    }

    /// Convert a char offset (as reported by egui cursors) to 0-based (line, col).
    pub fn line_col_from_offset(&self, mut offset: usize) -> (usize, usize) {
        for (i, line) in self.lines.iter().enumerate() {
            let chars = line.chars().count();
            if offset <= chars {
                return (i, offset);
            }
            offset = offset.saturating_sub(chars + 1); // +1 for the newline
        }
        let last = self.lines.len().saturating_sub(1);
        (
            last,
            self.lines.get(last).map(|l| l.chars().count()).unwrap_or(0),
        )
    }

    /// Convert a 0-based (line, col) to a char offset.
    pub fn offset_from_line_col(&self, line: usize, col: usize) -> usize {
        let mut offset = 0;
        for (i, l) in self.lines.iter().enumerate() {
            if i == line {
                return offset + col.min(l.chars().count());
            }
            offset += l.chars().count() + 1;
        }
        offset
    }
}

fn split_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    text.split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_buffer_is_clean() {
        let b = TextBuffer::from_text("fn main() {}\n");
        assert!(!b.is_dirty());
        assert_eq!(b.line_count(), 2);
    }

    #[test]
    fn edit_marks_dirty_and_revert_clears() {
        let mut b = TextBuffer::from_text("a\nb");
        b.set_text("a\nc");
        assert!(b.is_dirty());
        b.revert();
        assert!(!b.is_dirty());
        assert_eq!(b.text(), "a\nb");
    }

    #[test]
    fn mark_saved_clears_dirty() {
        let mut b = TextBuffer::from_text("a");
        b.set_text("b");
        b.mark_saved();
        assert!(!b.is_dirty());
    }

    #[test]
    fn offset_round_trip() {
        let b = TextBuffer::from_text("ab\ncde\nf");
        assert_eq!(b.line_col_from_offset(0), (0, 0));
        assert_eq!(b.line_col_from_offset(2), (0, 2));
        assert_eq!(b.line_col_from_offset(3), (1, 0));
        assert_eq!(b.offset_from_line_col(1, 2), 5);
    }
}
