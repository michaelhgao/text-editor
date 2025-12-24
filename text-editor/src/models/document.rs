use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::SystemTime,
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
    swap_path: Option<PathBuf>,
    dirty: bool,
    last_swap: SystemTime,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![GapBuffer::new()],
            original_path: None,
            temp_path: PathBuf::new(),
            swap_path: None,
            dirty: false,
            last_swap: SystemTime::now(),
        }
    }

    pub fn open(path: &str) -> io::Result<Self> {
        let original = PathBuf::from(path);

        let swap = Self::swap_path_for(&original);

        if swap.exists() {
            return Err(io::Error::new(io::ErrorKind::Other, "Swap file exists"));
        }

        File::create(&swap)?;

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
            swap_path: Some(swap),
            last_swap: SystemTime::now(),
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

    fn swap_path_for(path: &PathBuf) -> PathBuf {
        let mut swap = path.clone();
        let name = swap.file_name().unwrap().to_string_lossy().to_string();
        swap.set_file_name(format!(".{}.swp", name));
        swap
    }

    pub fn write_swap(&mut self, cursor: (usize, usize)) -> io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let swap_path = match &self.swap_path {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut swap_file = File::create(swap_path)?;

        writeln!(swap_file, "# SWAP")?;
        if let Some(ref orig) = self.original_path {
            writeln!(swap_file, "# path={}", orig.display())?;
        }
        writeln!(swap_file, "# cursor={},{}", cursor.0, cursor.1)?;
        for line in &self.lines {
            writeln!(swap_file, "{}", line.to_string())?;
        }
        swap_file.sync_all()?;
        self.last_swap = SystemTime::now();
        Ok(())
    }

    pub fn recover_from_swap(path: &PathBuf) -> io::Result<Document> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        let mut cursor = (0, 0);
        let mut original_path: Option<PathBuf> = None;

        for line in reader.lines() {
            let line = line?;
            if let Some(rest) = line.strip_prefix("# path=") {
                original_path = Some(PathBuf::from(rest));
            } else if let Some(rest) = line.strip_prefix("# cursor=") {
                let mut parts = rest.split(',');
                let row = parts.next().unwrap().parse().unwrap();
                let col = parts.next().unwrap().parse().unwrap();
                cursor = (row, col);
            } else if !line.starts_with('#') {
                let mut gb = GapBuffer::new();
                gb.insert_str(0, &line);
                lines.push(gb);
            }
        }

        if lines.is_empty() {
            lines.push(GapBuffer::new());
        }

        let original = original_path.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Swap missing original path")
        })?;

        let mut temp = original.clone();
        temp.set_extension("tmp");

        let mut temp_file = File::create(&temp)?;
        for (i, line) in lines.iter().enumerate() {
            temp_file.write_all(line.to_string().as_bytes())?;
            if i + 1 < lines.len() {
                temp_file.write_all(b"\n")?;
            }
        }

        Ok(Document {
            lines,
            dirty: true,
            original_path: Some(original),
            temp_path: temp,
            swap_path: Some(path.clone()),
            last_swap: SystemTime::now(),
        })
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.temp_path);
        if let Some(ref swap) = self.swap_path {
            let _ = std::fs::remove_file(swap);
        }
    }
}
