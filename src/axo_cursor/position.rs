use {
    crate::{
        file::{read_to_string},
        format,
        compare::{Ordering},
    }
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub path: &'static str,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            path: "",
        }
    }
}

impl Position {
    pub fn new(file: &'static str) -> Self {
        Self {
            line: 1,
            column: 1,
            path: file,
        }
    }

    pub fn correct(&self) -> Self {
        let mut corrected = self.clone();

        let content = match read_to_string(&self.path) {
            Ok(content) => content,
            Err(_) => return Self::new(self.path),
        };

        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Self::new(self.path);
        }

        corrected.line = corrected.line.max(1).min(lines.len());

        let line_index = corrected.line.saturating_sub(1);
        let line_length = lines[line_index].len();

        corrected.column = corrected.column.min(line_length);

        corrected
    }

    pub fn advance(&mut self, c: char) {
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    pub fn at(file: &'static str, line: usize, column: usize) -> Self {
        let mut pos = Self::new(file);
        pos.line = line;
        pos.column = column;
        pos.correct()
    }

    pub fn get_line_content(&self) -> Option<String> {
        match read_to_string(&self.path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if self.line > 0 && self.line <= lines.len() {
                    Some(lines[self.line - 1].to_string())
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }

    pub fn cmp(&self, other: &Self) -> Ordering {
        if self.path != other.path {
            return self.path.cmp(&other.path);
        }

        match self.line.cmp(&other.line) {
            Ordering::Equal => self.column.cmp(&other.column),
            other => other,
        }
    }
}


impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}