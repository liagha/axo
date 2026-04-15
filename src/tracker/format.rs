// src/tracker/format.rs
use crate::{
    format::{Debug, Display, Formatter, Result},
    tracker::{Position, Span},
};

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.identity, self.offset)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.start == self.end {
            write!(f, "{}:{}", self.identity, self.start)
        } else {
            write!(f, "{}:{}-{}", self.identity, self.start, self.end)
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.start == self.end {
            write!(f, "{}:{}", self.identity, self.start)
        } else {
            write!(f, "{}:{}-{}", self.identity, self.start, self.end)
        }
    }
}
