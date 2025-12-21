/// A `Document` represents a text document in the text editor.
///
///

#[derive(Debug)]
pub enum DocumentError {
    RowOutOfBounds,
    ColOutOfBounds,
}

pub struct Document {
    lines: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) -> Result<(), DocumentError> {
        let line = self
            .lines
            .get_mut(row)
            .ok_or(DocumentError::RowOutOfBounds)?;

        if col > line.len() {
            return Err(DocumentError::ColOutOfBounds);
        }

        let r = line.split_off(col);
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

        line.insert(col, c);
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

            previous_line.push_str(&current_line);
        } else {
            let line = self
                .lines
                .get_mut(row)
                .ok_or(DocumentError::RowOutOfBounds)?;

            if col > line.len() {
                return Err(DocumentError::ColOutOfBounds);
            }
            line.remove(col - 1);
        }
        Ok(())
    }

    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }
}
