// src/models/editor.rs
use crate::models::document::Document;

pub struct Editor {
    document: Document,
    cursor_position: (usize, usize), // (line, column)
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            cursor_position: (0, 0),
            should_quit: false,
        }
    }

    pub fn process_keypress(&mut self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            crossterm::event::KeyCode::Char(c) => {
                self.document.insert(self.cursor_position.0, &c.to_string());
                self.cursor_position.1 += 1;
            }
            crossterm::event::KeyCode::Backspace => {
                if self.cursor_position.1 > 0 {
                    self.document.delete(self.cursor_position.0, self.cursor_position.1 - 1);
                    self.cursor_position.1 -= 1;
                }
            }
            crossterm::event::KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn document(&self) -> &Document {
        &self.document
    }
}