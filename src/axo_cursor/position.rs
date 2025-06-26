use {
    crate::{
        file::{read_to_string},
        format,
        compare::{
            Ordering
        },
        operations::{
            Add, Sub, Mul, Div
        },
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
    #[inline]
    pub fn new(file: &'static str) -> Self {
        Self {
            line: 1,
            column: 1,
            path: file,
        }
    }
    
    #[inline]
    pub fn set_line(&mut self, line: usize) {
        self.line = line;
    }

    #[inline]
    pub fn set_column(&mut self, column: usize) {
        self.column = column;
    }

    #[inline]
    pub fn set_path(&mut self, path: &'static str) {
        self.path = path;
    }

    #[inline]
    pub fn swap_line(&self, line: usize) -> Self {
        Self {
            line,
            column: self.column,
            path: self.path
        }
    }

    #[inline]
    pub fn swap_column(&self, column: usize) -> Self {
        Self {
            line: self.line,
            column,
            path: self.path
        }
    }

    #[inline]
    pub fn swap_path(&self, path: &'static str) -> Self {
        Self {
            line: self.line,
            column: self.column,
            path,
        }
    }

    #[inline]
    pub fn join_line(&self, amount: usize) -> Self {
        Self {
            line: self.line + amount,
            column: self.column,
            path: self.path,
        }
    }

    #[inline]
    pub fn join_column(&self, amount: usize) -> Self {
        Self {
            line: self.line,
            column: self.column + amount,
            path: self.path,
        }
    }

    #[inline]
    pub fn add_line(&mut self, amount: usize) {
        self.line += amount;
    }

    #[inline]
    pub fn add_column(&mut self, amount: usize) {
        self.column += amount;
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