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
use crate::{environment, Str};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Location<'a> {
    File(Str<'a>),
    Flag,
}

impl<'a> Location<'a> {
    pub fn get_value(&self) -> Str<'a> {
        match self {
            Location::File(file) => read_to_string(file.as_str().unwrap()).unwrap_or("".to_string()).into(),
            Location::Flag => environment::args().skip(1).collect::<Vec<String>>().join(" ").into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position<'position> {
    pub line: usize,
    pub column: usize,
    pub location: Location<'position>,
}

impl<'a> Position<'a> {
    #[inline]
    pub fn new(location: Location<'a>) -> Self {
        Self {
            line: 1,
            column: 1,
            location,
        }
    }

    #[inline]
    pub fn default(location: Location<'a>) -> Self {
        Self {
            line: 1,
            column: 1,
            location,
        }
    }

    #[inline]
    pub fn path(line: usize, column: usize, path: Str<'a>) -> Self {
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
    pub fn set_path(&mut self, path: Str<'a>) {
        self.location = Location::File(path);
    }

    #[inline]
    pub fn set_location(&mut self, location: Location<'a>) {
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
    pub fn swap_path(&self, path: Str<'a>) -> Self {
        Self {
            location: Location::File(path),
            ..*self
        }
    }

    #[inline]
    pub fn swap_location(&self, location: Location<'a>) -> Self {
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

impl<'a> PartialOrd for Position<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Position<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}