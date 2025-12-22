use crate::models::document::Document;
use crossterm::event::{KeyCode, KeyEvent};
use tui::layout::Rect;

const TAB_WIDTH: usize = 4;

pub struct Editor {
    doc: Document,
    cursor: (usize, usize), // (row, col)
    pref_col: usize,
    mode: Mode,
    cmd_buf: String,
    should_quit: bool,
}

impl Editor {
    pub fn new(doc: Document) -> Self {
        Self {
            doc,
            cursor: (0, 0),
            pref_col: 0,
            mode: Mode::Normal,
            cmd_buf: String::new(),
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, rect: &Rect) {
        match self.mode {
            Mode::Normal => {
                self.handle_normal_mode(key, rect);
            }
            Mode::Insert => {
                self.handle_insert_mode(key, rect);
            }
            Mode::Command => {
                self.handle_command_mode(key, rect);
            }
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent, rect: &Rect) {
        match key.code {
            KeyCode::Char('w') => {
                self.move_cursor(0, -1, rect);
            }
            KeyCode::Char('a') => {
                self.move_cursor(-1, 0, rect);
            }
            KeyCode::Char('s') => {
                self.move_cursor(0, 1, rect);
            }
            KeyCode::Char('d') => {
                self.move_cursor(1, 0, rect);
            }
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.cmd_buf.clear();
            }
            _ => {}
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent, rect: &Rect) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
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
            KeyCode::Enter => {
                self.doc.insert_newline(self.cursor.0, self.cursor.1);
                self.cursor.0 += 1;
                self.cursor.1 = 0;
                self.pref_col = 0;
            }
            KeyCode::Tab => {
                for _ in 0..TAB_WIDTH {
                    self.doc.insert_char(self.cursor.0, self.cursor.1, ' ');
                    self.move_cursor(1, 0, rect)
                }
            }
            _ => {}
        }
    }

    fn handle_command_mode(&mut self, key: KeyEvent, rect: &Rect) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.cmd_buf.clear();
            }
            KeyCode::Enter => {
                self.execute_command();
                self.mode = Mode::Normal;
                self.cmd_buf.clear();
            }
            KeyCode::Backspace => {
                self.cmd_buf.pop();
            }

            KeyCode::Char(c) => {
                self.cmd_buf.push(c);
            }
            _ => {}
        }
    }

    fn execute_command(&mut self) {
        match self.cmd_buf.as_str() {
            "q" => {
                // quit
                if self.doc.dirty() {
                } else {
                    self.should_quit = true;
                }
            }
            "s" => {
                // save
                self.doc.save(None);
            }
            cmd if cmd.starts_with("s ") => {
                // save as
                let name = cmd[2..].trim();
                self.doc.save(Some(name));
            }
            "sq" => {
                // save quit
                if self.doc.save(None).is_ok() {
                    self.should_quit = true;
                }
            }
            _ => {}
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

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn command_buffer(&self) -> &String {
        &self.cmd_buf
    }
}

pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
}
