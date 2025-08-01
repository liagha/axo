use {
    super::{
        Span, Position,
    },
    
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },
        axo_cursor::Location,
    }
};

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Location::File(file) => write!(f, "File({})", file),
            Location::Flag => write!(f, "Flag"),
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.location, self.line, self.column)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            if self.start.location != self.end.location {
                write!(f, "{}:{} - {}:{}",
                       self.start.line, self.start.column,
                       self.end.line, self.end.column)
            } else if self.start.line == self.end.line && self.start.column == self.end.column {
                write!(f, "{}:{}", self.start.line, self.start.column)
            } else if self.start.line == self.end.line {
                write!(f, "{}:{}-{}",
                       self.start.line,
                       self.start.column, self.end.column
                )
            } else {
                write!(f, "{}:{}-{}:{}",
                       self.start.line, self.start.column,
                       self.end.line, self.end.column)
            }
        } else {
            if self.start.location != self.end.location {
                write!(f, "{}:{}:{} - {}:{}:{}",
                       self.start.location, self.start.line, self.start.column,
                       self.end.location, self.end.line, self.end.column)
            } else if self.start.line == self.end.line && self.start.column == self.end.column {
                write!(f, "{}:{}:{}", self.start.location, self.start.line, self.start.column)
            } else if self.start.line == self.end.line {
                write!(f, "{}:{}:{}-{}",
                       self.start.location, self.start.line,
                       self.start.column, self.end.column)
            } else {
                write!(f, "{}:{}:{}-{}:{}",
                       self.start.location, self.start.line, self.start.column,
                       self.end.line, self.end.column)
            }
        }
    }
}


impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.start.location != self.end.location {
            write!(f, "{}:{}:{} - {}:{}:{}",
                   self.start.location, self.start.line, self.start.column,
                   self.end.location, self.end.line, self.end.column)
        } else if self.start.line == self.end.line && self.start.column == self.end.column {
            write!(f, "{}:{}:{}", self.start.location, self.start.line, self.start.column)
        } else if self.start.line == self.end.line {
            write!(f, "{}:{}:{}-{}",
                   self.start.location, self.start.line,
                   self.start.column, self.end.column)
        } else {
            write!(f, "{}:{}:{}-{}:{}",
                   self.start.location, self.start.line, self.start.column,
                   self.end.line, self.end.column)
        }
    }
}
