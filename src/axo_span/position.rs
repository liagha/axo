use crate::Path;
use crate::fs;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub file: Path,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            file: Path::default(),
        }
    }
}

impl Position {
    pub fn new(file: Path) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            file,
        }
    }

    pub fn correct(&self) -> Self {
        let mut corrected = self.clone();

        let content = match fs::read_to_string(&self.file) {
            Ok(content) => content,
            Err(_) => return Self::new(self.file.clone()),
        };

        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Self::new(self.file.clone());
        }

        corrected.line = corrected.line.max(1).min(lines.len());

        let line_index = corrected.line.saturating_sub(1);
        let line_length = lines[line_index].len();

        corrected.column = corrected.column.min(line_length);

        let mut index = 0;

        for (i, line) in lines.iter().enumerate() {
            if i + 1 == corrected.line {
                index += corrected.column;
                break;
            }
            index += line.len() + 1;
        }
        corrected.index = index.min(content.len());

        corrected
    }

    pub fn advance(&mut self, c: char) {
        self.index += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}