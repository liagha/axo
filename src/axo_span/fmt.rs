use core::fmt;
use crate::axo_span::position::Position;
use crate::axo_span::Span;

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            if self.start.file != self.end.file {
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
            if self.start.file != self.end.file {
                write!(f, "{}:{}:{} - {}:{}:{}",
                       self.start.file.display(), self.start.line, self.start.column,
                       self.end.file.display(), self.end.line, self.end.column)
            } else if self.start.line == self.end.line && self.start.column == self.end.column {
                write!(f, "{}:{}:{}", self.start.file.display(), self.start.line, self.start.column)
            } else if self.start.line == self.end.line {
                write!(f, "{}:{}:{}-{}",
                       self.start.file.display(), self.start.line,
                       self.start.column, self.end.column)
            } else {
                write!(f, "{}:{}:{}-{}:{}",
                       self.start.file.display(), self.start.line, self.start.column,
                       self.end.line, self.end.column)
            }
        }
    }
}


impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.file != self.end.file {
            write!(f, "{}:{}:{} - {}:{}:{}",
                   self.start.file.display(), self.start.line, self.start.column,
                   self.end.file.display(), self.end.line, self.end.column)
        } else if self.start.line == self.end.line && self.start.column == self.end.column {
            write!(f, "{}:{}:{}", self.start.file.display(), self.start.line, self.start.column)
        } else if self.start.line == self.end.line {
            write!(f, "{}:{}:{}-{}",
                   self.start.file.display(), self.start.line,
                   self.start.column, self.end.column)
        } else {
            write!(f, "{}:{}:{}-{}:{}",
                   self.start.file.display(), self.start.line, self.start.column,
                   self.end.line, self.end.column)
        }
    }
}
