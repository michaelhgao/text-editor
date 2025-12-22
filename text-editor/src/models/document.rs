use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
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
    original_path: Option<PathBuf>,
    temp_path: PathBuf,
    dirty: bool,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![GapBuffer::new()],
            original_path: None,
            temp_path: PathBuf::new(),
            dirty: false,
        }
    }

    pub fn open(path: &str) -> io::Result<Self> {
        let original = PathBuf::from(path);
        let mut temp = original.clone();
        temp.set_extension("tmp");

        fs::copy(&original, &temp)?;

        let file = File::open(&temp)?;
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
            original_path: Some(original),
            temp_path: temp,
            dirty: false,
        })
    }

    pub fn save(&mut self, new_name: Option<&str>) -> io::Result<()> {
        if let Some(name) = new_name {
            self.original_path = Some(PathBuf::from(name));
        }

        let original = match &self.original_path {
            Some(path) => path.clone(),
            None => return Err(io::Error::new(io::ErrorKind::Other, "No file name")),
        };

        let mut temp = File::create(&self.temp_path)?;

        for (i, line) in self.lines.iter().enumerate() {
            temp.write_all(line.to_string().as_bytes())?;
            if i + 1 < self.lines.len() {
                temp.write_all(b"\n")?;
            }
        }

        temp.sync_all()?;

        fs::rename(&self.temp_path, &original)?;

        fs::copy(&original, &self.temp_path)?;

        self.dirty = false;

        Ok(())
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

        self.dirty = true;

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

        self.dirty = true;

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

        self.dirty = true;

        Ok(())
    }

    pub fn lines(&self) -> &Vec<GapBuffer> {
        &self.lines
    }

    pub fn file_name(&self) -> &str {
        self.original_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]")
    }

    pub fn full_path(&self) -> Option<&Path> {
        self.original_path.as_deref()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.temp_path);
    }
}
