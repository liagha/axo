#![allow(dead_code)]

use crate::Path;
use crate::axo_span::Spanned;
use crate::axo_span::position::Position;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}
impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span {
            start,
            end,
        }
    }

    pub fn extend(&mut self, size: (usize, usize, usize)) {
        self.end.line += size.0;
        self.end.column += size.1;
        self.end.index += size.2;
    }

    pub fn extend_to(&mut self, other: Box<dyn Spanned>) {
        self.end = other.span().end;
    }
}

impl Spanned for Span {
    fn span(&self) -> Span {
        self.clone()
    }
}