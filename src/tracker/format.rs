use crate::{
    format::{Debug, Display, Formatter, Result},
    tracker::{Location, Position, Span},
};

impl Debug for Location<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Location::Entry(file) => write!(f, "File({})", file),
            Location::Void => write!(f, "Void"),
            Location::Flag => write!(f, "Flag"),
        }
    }
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Location::Entry(file) => write!(f, "{}", file),
            Location::Void => write!(f, "Void"),
            Location::Flag => write!(f, "Flag"),
        }
    }
}

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
