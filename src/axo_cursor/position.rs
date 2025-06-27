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
pub enum Location {
    File(&'static str),
    Void,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub location: Location,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            location: Location::Void,
        }
    }
}

impl Position {
    #[inline]
    pub fn new(file: &'static str) -> Self {
        Self {
            line: 1,
            column: 1,
            location: Location::File(file),
        }
    }
    
    #[inline]
    pub fn path(line: usize, column: usize, path: &'static str) -> Self {
        Self {
            line,
            column,
            location: Location::File(path),
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
        self.location = Location::File(path);
    }
    
    #[inline]
    pub fn set_location(&mut self, location: Location) {
        self.location = location;
    }

    #[inline]
    pub fn swap_line(&self, line: usize) -> Self {
        Self {
            line,
            column: self.column,
            location: self.location
        }
    }

    #[inline]
    pub fn swap_column(&self, column: usize) -> Self {
        Self {
            line: self.line,
            column,
            location: self.location
        }
    }

    #[inline]
    pub fn swap_path(&self, path: &'static str) -> Self {
        Self {
            line: self.line,
            column: self.column,
            location: Location::File(path),
        }
    }

    #[inline]
    pub fn swap_location(&self, location: Location) -> Self {
        Self {
            line: self.line,
            column: self.column,
            location,
        }
    }

    #[inline]
    pub fn join_line(&self, amount: usize) -> Self {
        Self {
            line: self.line + amount,
            column: self.column,
            location: self.location,
        }
    }

    #[inline]
    pub fn join_column(&self, amount: usize) -> Self {
        Self {
            line: self.line,
            column: self.column + amount,
            location: self.location,
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
        if self.location != other.location {
            return Ordering::Less;
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