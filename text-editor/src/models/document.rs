pub struct Document {
    lines: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn insert(&mut self, line: usize, text: &str) {
        if line >= self.lines.len() {
            self.lines.push(text.to_string());
        }
        else {
            self.lines[line].push_str(text);
        }
    }

    pub fn delete(&mut self, line: usize, index: usize) {
        if line < self.lines.len() && index < self.lines[line].len() {
            self.lines[line].remove(index);
        }
    }

    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }

}