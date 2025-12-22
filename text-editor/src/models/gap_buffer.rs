pub struct GapBuffer {
    data: Vec<char>,
    gap_start: usize,
    gap_size: usize,
}

impl GapBuffer {
    pub fn new() -> Self {
        let cap = 10;
        Self {
            data: vec!['\0'; cap],
            gap_start: 0,
            gap_size: cap,
        }
    }

    pub fn insert_char(&mut self, col: usize, c: char) {
        self.move_gap(col);

        if self.gap_size == 0 {
            self.grow();
        }

        self.data[self.gap_start] = c;
        self.gap_start += 1;
        self.gap_size -= 1;
    }

    pub fn delete(&mut self, col: usize) {
        self.move_gap(col);

        if self.gap_start > 0 {
            self.gap_start -= 1;
            self.gap_size += 1;
        }
    }

    pub fn merge(&mut self, other: GapBuffer) {
        self.move_gap(self.len());

        for c in other.chars() {
            self.insert_char(self.len(), c);
        }
    }

    pub fn split(&mut self, col: usize) -> Self {
        self.move_gap(col);

        let suffix_start = self.gap_start + self.gap_size;
        let suffix_chars: Vec<char> = self.data[suffix_start..].to_vec();

        self.data.truncate(self.gap_start + self.gap_size);

        let mut new_buf = GapBuffer::new();
        for c in suffix_chars {
            new_buf.insert_char(new_buf.len(), c);
        }

        new_buf
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.gap_size
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.data[..self.gap_start]
            .iter()
            .chain(self.data[self.gap_start + self.gap_size..].iter())
            .copied()
    }

    pub fn insert_str(&mut self, col: usize, s: &str) {
        self.move_gap(col);
        for c in s.chars() {
            if self.gap_size == 0 {
                self.grow();
            }
            self.data[self.gap_start] = c;
            self.gap_start += 1;
            self.gap_size -= 1;
        }
    }

    fn move_gap(&mut self, idx: usize) {
        if idx == self.gap_start {
            return;
        }

        if idx < self.gap_start {
            let distance = self.gap_start - idx;
            for i in 0..distance {
                self.data[self.gap_start + self.gap_size - 1 - i] =
                    self.data[self.gap_start - 1 - i];
            }
        } else {
            let distance = idx - self.gap_start;
            for i in 0..distance {
                self.data[self.gap_start + i] = self.data[self.gap_start + self.gap_size + i];
            }
        }
        self.gap_start = idx;
    }

    fn grow(&mut self) {
        let old_cap = self.data.len();
        let grow_by = old_cap.max(10);
        let new_cap = old_cap + grow_by;

        self.data.resize(new_cap, '\0');

        let suffix_len = old_cap - (self.gap_start + self.gap_size);

        if suffix_len > 0 {
            let old_suffix_start = self.gap_start + self.gap_size;
            let new_suffix_start = old_suffix_start + grow_by;

            self.data
                .copy_within(old_suffix_start..old_cap, new_suffix_start);
        }

        self.gap_size += grow_by;
    }
}

impl ToString for GapBuffer {
    fn to_string(&self) -> String {
        let mut s = String::with_capacity(self.len());
        s.extend(&self.data[..self.gap_start]);
        s.extend(&self.data[self.gap_start + self.gap_size..]);
        s
    }
}
