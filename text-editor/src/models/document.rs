use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use crate::models::gap_buffer::GapBuffer;

#[derive(Debug)]
pub enum DocumentError {
    RowOutOfBounds,
    ColOutOfBounds,
}

/// A `Document` represents a text document in the text editor.
pub struct Document {
    lines: Vec<GapBuffer>,
    path: Option<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![GapBuffer::new()],
            path: None,
        }
    }

    pub fn open(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let mut gb = GapBuffer::new();
            gb.insert_str(gb.len(), &line);
            lines.push(gb);
        }
        if lines.is_empty() {
            lines.push(GapBuffer::new());
        }
        Ok(Self {
            lines,
            path: Some(path.to_string()),
        })
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) -> Result<(), DocumentError> {
        let line = self
            .lines
            .get_mut(row)
            .ok_or(DocumentError::RowOutOfBounds)?;

        if col > line.len() {
            return Err(DocumentError::ColOutOfBounds);
        }

        let r = line.split(col);
        self.lines.insert(row + 1, r);
        Ok(())
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) -> Result<(), DocumentError> {
        let line = self
            .lines
            .get_mut(row)
            .ok_or(DocumentError::RowOutOfBounds)?;

        if col > line.len() {
            return Err(DocumentError::ColOutOfBounds);
        }

        line.insert_char(col, c);
        Ok(())
    }

    pub fn delete(&mut self, row: usize, col: usize) -> Result<(), DocumentError> {
        if col == 0 {
            if row == 0 {
                return Ok(());
            }

            let current_line = self.lines.remove(row);

            let previous_line = self
                .lines
                .get_mut(row - 1)
                .ok_or(DocumentError::RowOutOfBounds)?;

            previous_line.merge(current_line);
        } else {
            let line = self
                .lines
                .get_mut(row)
                .ok_or(DocumentError::RowOutOfBounds)?;

            if col > line.len() {
                return Err(DocumentError::ColOutOfBounds);
            }
            line.delete(col - 1);
        }
        Ok(())
    }

    pub fn lines(&self) -> &Vec<GapBuffer> {
        &self.lines
    }
}
