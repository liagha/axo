#![allow(dead_code)]

use crate::Path;
use crate::axo_span::Spanned;
use crate::axo_span::position::Position;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: Path,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Span {
    pub fn new(start: (usize, usize), end: (usize, usize), file: Path) -> Self {
        Span {
            start,
            end,
            file,
        }
    }

    pub fn zero() -> Self {
        Self {
            file: Path::new(),
            start: (0, 0),
            end: (0, 0),
        }
    }

    pub fn extend(&mut self, size: (usize, usize)) {
        self.end.0 += size.0;
        self.end.1 += size.1;
    }

    pub fn extend_to(&mut self, other: Box<dyn Spanned>) {
        self.end.0 = other.span().end.0;
        self.end.1 = other.span().end.1;
    }

    pub fn correct(&self) -> Self {
        let content = match crate::fs::read_to_string(&self.file) {
            Ok(content) => content,
            Err(_) => return Self::zero(),
        };

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Self::zero();
        }

        let correct_position = |line: usize, column: usize| -> (usize, usize) {
            let line = line.max(1).min(lines.len());
            let line_idx = line.saturating_sub(1);
            let column = column.min(lines[line_idx].len());
            (line, column)
        };

        let (start_line, start_column) = correct_position(self.start.0, self.start.1);
        let (end_line, end_column) = correct_position(self.end.0, self.end.1);

        let (final_start, final_end) = if start_line > end_line || (start_line == end_line && start_column > end_column) {
            ((end_line, end_column), (start_line, start_column))
        } else {
            ((start_line, start_column), (end_line, end_column))
        };

        Span {
            file: self.file.clone(),
            start: final_start,
            end: final_end,
        }
    }

    pub fn merge(&self, other: &Span) -> Self {
        let start_line = self.start.0.min(other.start.0);
        let start_col = if self.start.0 == other.start.0 {
            self.start.1.min(other.start.1)
        } else {
            if self.start.0 < other.start.0 { self.start.1 } else { other.start.1 }
        };

        let end_line = self.end.0.max(other.end.0);
        let end_col = if self.end.0 == other.end.0 {
            self.end.1.max(other.end.1)
        } else {
            if self.end.0 > other.end.0 { self.end.1 } else { other.end.1 }
        };

        Span {
            file: self.file.clone(),
            start: (start_line, start_col),
            end: (end_line, end_col),
        }
    }

    pub fn contains(&self, position: (usize, usize)) -> bool {
        let (line, col) = position;

        if line < self.start.0 || line > self.end.0 {
            return false;
        }

        if line == self.start.0 && col < self.start.1 {
            return false;
        }

        if line == self.end.0 && col > self.end.1 {
            return false;
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn shift(&self, line_offset: usize, col_offset: usize) -> Self {
        Span {
            file: self.file.clone(),
            start: (self.start.0 + line_offset, self.start.1 + col_offset),
            end: (self.end.0 + line_offset, self.end.1 + col_offset),
        }
    }

    fn extract_span(&self) -> String {
        let (start_line, start_column) = self.start;
        let (end_line, end_column) = self.end;

        let source = crate::fs::read_to_string(self.file.clone()).unwrap_or_default();

        let lines: Vec<&str> = source.lines().collect();

        let start_line_idx = start_line.saturating_sub(1);
        let end_line_idx = end_line.saturating_sub(1);

        if start_line_idx >= lines.len() || end_line_idx >= lines.len() {
            return String::new();
        }

        if start_line_idx == end_line_idx {
            let line = lines[start_line_idx];
            let start = start_column.saturating_sub(1);
            let end = end_column.saturating_sub(1).min(line.len());
            if start <= end && end <= line.len() {
                return line[start..end].to_string();
            }
        } else {
            let mut result = String::new();

            let first_line = lines[start_line_idx];
            let first_line_start = start_column.saturating_sub(1);
            if first_line_start < first_line.len() {
                result.push_str(&first_line[first_line_start..]);
            }
            result.push('\n');

            for line_idx in (start_line_idx + 1)..end_line_idx {
                result.push_str(lines[line_idx]);
                result.push('\n');
            }

            let last_line = lines[end_line_idx];
            let last_line_end = end_column.saturating_sub(1).min(last_line.len());
            if last_line_end > 0 {
                result.push_str(&last_line[..last_line_end]);
            }

            return result;
        }

        String::new()
    }

    pub fn line_start(&self) -> usize {
        self.start.0
    }

    pub fn column_start(&self) -> usize {
        self.start.1
    }

    pub fn line_end(&self) -> usize {
        self.end.0
    }

    pub fn column_end(&self) -> usize {
        self.end.1
    }
}