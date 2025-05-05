use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub file: PathBuf,
}

impl Position {
    pub fn new(file: PathBuf) -> Self {
        Self {
            line: 1,
            column: 0,
            index: 0,
            file,
        }
    }
}