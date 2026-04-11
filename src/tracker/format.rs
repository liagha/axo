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

impl Display for Position<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.location, self.line, self.column)
    }
}

impl Debug for Span<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            if self.start_line == self.end_line && self.start_column == self.end_column {
                write!(f, "{}:{}", self.start_line, self.start_column)
            } else if self.start_line == self.end_line {
                write!(
                    f,
                    "{}:{}-{}",
                    self.start_line, self.start_column, self.end_column
                )
            } else {
                write!(
                    f,
                    "{}:{}-{}:{}",
                    self.start_line, self.start_column, self.end_line, self.end_column
                )
            }
        } else {
            if self.start_line == self.end_line && self.start_column == self.end_column {
                write!(
                    f,
                    "{}:{}:{}",
                    self.location, self.start_line, self.start_column
                )
            } else if self.start_line == self.end_line {
                write!(
                    f,
                    "{}:{}:{}-{}",
                    self.location, self.start_line, self.start_column, self.end_column
                )
            } else {
                write!(
                    f,
                    "{}:{}:{}-{}:{}",
                    self.location,
                    self.start_line,
                    self.start_column,
                    self.end_line,
                    self.end_column
                )
            }
        }
    }
}

impl Display for Span<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.start_line == self.end_line && self.start_column == self.end_column {
            write!(
                f,
                "{}:{}:{}",
                self.location, self.start_line, self.start_column
            )
        } else if self.start_line == self.end_line {
            write!(
                f,
                "{}:{}:{}-{}",
                self.location, self.start_line, self.start_column, self.end_column
            )
        } else {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.location,
                self.start_line,
                self.start_column,
                self.end_line,
                self.end_column
            )
        }
    }
}
