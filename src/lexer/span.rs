use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: (usize, usize),  // (line, column)
    pub end: (usize, usize),    // (line, column)
    pub file_name: String, // The file name (e.g., "source.rs")
    pub file_path: PathBuf,     // Full path to the file
}

impl Span {
    pub fn new(start: (usize, usize), end: (usize, usize), file_path: PathBuf) -> Self {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        Span {
            start,
            end,
            file_name,
            file_path,
        }
    }

    pub fn with_owned_file_name(start: (usize, usize), end: (usize, usize), file_path: PathBuf) -> Self {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        Span {
            start,
            end,
            file_name,
            file_path,
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