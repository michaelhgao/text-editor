use crate::models::document::Document;
use crossterm::event::{KeyCode, KeyEvent};

pub struct Editor {
    doc: Document,
    cursor_pos: (usize, usize), // (row, col)
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            doc: Document::new(),
            cursor_pos: (0, 0),
            should_quit: false,
        }
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        let (mut col, mut row) = self.cursor_pos;

        if dy < 0 {
            row = row.saturating_sub(dy.unsigned_abs() as usize);
        } else {
            row = row.saturating_add(dy as usize);
        }

        if dx < 0 {
            col = col.saturating_sub(dx.unsigned_abs() as usize);
        } else {
            col = col.saturating_add(dx as usize);
        }

        let max_rows = self.doc.lines().len().saturating_sub(1);
        row = row.min(max_rows);

        if let Some(line) = self.doc.lines().get(row) {
            let line_len = line.chars().count();
            col = col.min(line_len);
        } else {
            col = 0;
        }

        self.cursor_pos = (row, col);
    }

    pub fn process_keypress(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                self.doc.insert(self.cursor_pos.0, self.cursor_pos.1, c);
                self.move_cursor(1, 0);
            }
            KeyCode::Backspace => {
                if self.cursor_pos.1 > 0 {
                    self.move_cursor(-1, 0);
                    self.doc.delete(self.cursor_pos.0, self.cursor_pos.1);
                }
            }
            KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }
}
