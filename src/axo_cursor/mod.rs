#![allow(unused_imports)]

mod span;
mod format;
mod position;
mod peekable;

pub use span::*;
pub use position::*;
pub use peekable::*;

use crate::axo_parser::{Element, Item};

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Element {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Item {
    fn span(&self) -> Span {
        self.span.clone()
    }
}