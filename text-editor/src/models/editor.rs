use crate::models::document::Document;
use crossterm::event::{KeyCode, KeyEvent};
use tui::layout::Rect;

pub struct Editor {
    doc: Document,
    cursor: (usize, usize), // (row, col)
    pref_col: usize,
    should_quit: bool,
}

impl Editor {
    pub fn new(doc: Document) -> Self {
        Self {
            doc,
            cursor: (0, 0),
            pref_col: 0,
            should_quit: false,
        }
    }

    fn wraps(len: usize, width: usize) -> usize {
        if len == 0 {
            1
        } else {
            (len + width - 1) / width
        }
    }

    fn move_cursor(&mut self, dx: i32, dy: i32, rect: &Rect) {
        let width = rect.width as usize;
        let mut row = self.cursor.0;
        let mut col = self.cursor.1;

        if dx != 0 {
            let dx = dx.signum();
            if dx < 0 {
                if col > 0 {
                    col -= 1;
                } else if row > 0 {
                    row -= 1;
                    col = self.doc.lines()[row].len();
                }
            } else {
                let len = self.doc.lines()[row].len();
                if col < len {
                    col += 1;
                } else if row + 1 < self.doc.lines().len() {
                    row += 1;
                    col = 0;
                }
            }
            self.pref_col = col;
        }

        if dy != 0 {
            let dy = dy.signum();
            if dy < 0 {
                if col >= width {
                    col -= width;
                } else if row > 0 {
                    row -= 1;
                    let prev_len = self.doc.lines()[row].len();
                    let line_wraps = Self::wraps(prev_len, width);
                    let last_wrap = line_wraps - 1;
                    col = (last_wrap * width + self.pref_col.min(width - 1)).min(prev_len);
                } else {
                    col = 0;
                }
            } else {
                let len = self.doc.lines()[row].len();
                if col + width < len {
                    col += width;
                } else if row + 1 < self.doc.lines().len() {
                    row += 1;
                    col = self.pref_col.min(self.doc.lines()[row].len());
                } else {
                    col = len;
                }
            }
        }

        self.cursor = (row, col);
    }

    pub fn doc_to_screen(&self, rect: &Rect) -> (u16, u16) {
        let width = rect.width as usize;
        let mut screen_row = 0;
        for i in 0..self.cursor.0 {
            let len = self.doc.lines()[i].len();
            screen_row += (len / width).max(1);
        }
        let wrapped_row = self.cursor.1 / width;
        let wrapped_col = self.cursor.1 % width;

        ((screen_row + wrapped_row) as u16, wrapped_col as u16)
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, rect: &Rect) {
        match key_event.code {
            KeyCode::Char(c) => {
                self.doc.insert_char(self.cursor.0, self.cursor.1, c);
                self.move_cursor(1, 0, rect);
            }
            KeyCode::Backspace => {
                if self.cursor.1 > 0 {
                    self.move_cursor(-1, 0, rect);
                    self.doc.delete(self.cursor.0, self.cursor.1);
                } else if self.cursor.0 > 0 {
                    let prev_len = self.doc.lines()[self.cursor.0 - 1].len();
                    self.doc.delete(self.cursor.0, 0);
                    self.cursor.0 -= 1;
                    self.cursor.1 = prev_len;
                    self.pref_col = self.cursor.1;
                }
            }
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Up => {
                self.move_cursor(0, -1, rect);
            }
            KeyCode::Down => {
                self.move_cursor(0, 1, rect);
            }
            KeyCode::Left => {
                self.move_cursor(-1, 0, rect);
            }
            KeyCode::Right => {
                self.move_cursor(1, 0, rect);
            }
            KeyCode::Enter => {
                self.doc.insert_newline(self.cursor.0, self.cursor.1);
                self.cursor.0 += 1;
                self.cursor.1 = 0;
                self.pref_col = 0;
            }
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }
}
