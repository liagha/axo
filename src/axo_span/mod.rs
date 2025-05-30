#![allow(unused_imports)]

pub mod span;
mod format;
pub mod position;

pub use span::*;
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