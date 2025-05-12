use core::fmt;
use crate::axo_span::Span;

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Span {
            start,
            end
        } = self;

        if start == end {
            write!(f, "{}:{}", start.line, start.column)
        } else {
            write!(f, "{}:{}-{}:{}", start.line, start.column, end.line, end.column)
        }
    }
}
