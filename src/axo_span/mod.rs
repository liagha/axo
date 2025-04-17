#![allow(unused_imports)]

pub mod span;
mod fmt;

pub use span::*;
use crate::axo_parser::{Expr, Item};

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Item {
    fn span(&self) -> Span {
        self.span.clone()
    }
}