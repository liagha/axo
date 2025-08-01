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
use crate::environment;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Location {
    File(&'static str),
    Flag,
}

impl Location {
    pub fn get_value(&self) -> String {
        match self {
            Location::File(file) => read_to_string(file).unwrap_or("".to_string()),
            Location::Flag => environment::args().skip(1).collect::<Vec<String>>().join(" "),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub location: Location,
}

impl Position {
    #[inline]
    pub fn new(location: Location) -> Self {
        Self {
            line: 1,
            column: 1,
            location,
        }
    }
    
    #[inline]
    pub fn default(location: Location) -> Self {
        Self {
            line: 1,
            column: 1,
            location,
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
            ..*self
        }
    }

    #[inline]
    pub fn swap_column(&self, column: usize) -> Self {
        Self {
            column,
            ..*self
        }
    }

    #[inline]
    pub fn swap_path(&self, path: &'static str) -> Self {
        Self {
            location: Location::File(path),
            ..*self
        }
    }

    #[inline]
    pub fn swap_location(&self, location: Location) -> Self {
        Self {
            location,
            ..*self
        }
    }

    #[inline]
    pub fn advance_line(&self, amount: usize) -> Self {
        Self {
            line: self.line + amount,
            ..*self
        }
    }

    #[inline]
    pub fn join_column(&self, amount: usize) -> Self {
        Self {
            column: self.column + amount,
            ..*self
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