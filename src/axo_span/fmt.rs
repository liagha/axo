use core::fmt;
use crate::axo_span::Span;

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Span {
            start: (start_line, start_column),
            end: (end_line, end_column), ..
        } = self;

        if start_line == end_line && start_column == end_column {
            write!(f, "{}:{}", start_line, start_column)
        } else {
            write!(f, "{}:{}-{}:{}", start_line, start_column, end_line, end_column)
        }
    }
}
