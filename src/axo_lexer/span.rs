#![allow(dead_code)]

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: PathBuf,     // Full path to the file
    pub start: (usize, usize),  // (line, column)
    pub end: (usize, usize),    // (line, column)
}

impl Span {
    pub fn new(start: (usize, usize), end: (usize, usize), file: PathBuf) -> Self {
        Span {
            start,
            end,
            file,
        }
    }

    fn extract_span(&self) -> String {
        let (start_line, start_column) = self.start;
        let (end_line, end_column) = self.end;
        
        let source = std::fs::read_to_string(self.file.clone()).unwrap();
        
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