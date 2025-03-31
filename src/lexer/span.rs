use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: (usize, usize),  // (line, column)
    pub end: (usize, usize),    // (line, column)
    pub file: PathBuf,     // Full path to the file
}

impl Span {
    pub fn new(start: (usize, usize), end: (usize, usize), file: PathBuf) -> Self {
        Span {
            start,
            end,
            file,
        }
    }

    pub fn with_owned_file_name(start: (usize, usize), end: (usize, usize), file: PathBuf) -> Self {
        Span {
            start,
            end,
            file,
        }
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