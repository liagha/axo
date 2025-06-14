use {
    super::{
        Span, Position,
    },
    
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },
    }
};

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.path.display(), self.line, self.column)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            if self.start.path != self.end.path {
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
            if self.start.path != self.end.path {
                write!(f, "{}:{}:{} - {}:{}:{}",
                       self.start.path.display(), self.start.line, self.start.column,
                       self.end.path.display(), self.end.line, self.end.column)
            } else if self.start.line == self.end.line && self.start.column == self.end.column {
                write!(f, "{}:{}:{}", self.start.path.display(), self.start.line, self.start.column)
            } else if self.start.line == self.end.line {
                write!(f, "{}:{}:{}-{}",
                       self.start.path.display(), self.start.line,
                       self.start.column, self.end.column)
            } else {
                write!(f, "{}:{}:{}-{}:{}",
                       self.start.path.display(), self.start.line, self.start.column,
                       self.end.line, self.end.column)
            }
        }
    }
}


impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.start.path != self.end.path {
            write!(f, "{}:{}:{} - {}:{}:{}",
                   self.start.path.display(), self.start.line, self.start.column,
                   self.end.path.display(), self.end.line, self.end.column)
        } else if self.start.line == self.end.line && self.start.column == self.end.column {
            write!(f, "{}:{}:{}", self.start.path.display(), self.start.line, self.start.column)
        } else if self.start.line == self.end.line {
            write!(f, "{}:{}:{}-{}",
                   self.start.path.display(), self.start.line,
                   self.start.column, self.end.column)
        } else {
            write!(f, "{}:{}:{}-{}:{}",
                   self.start.path.display(), self.start.line, self.start.column,
                   self.end.line, self.end.column)
        }
    }
}
