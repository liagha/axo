#![allow(unused_imports)]

mod span;
mod format;
mod position;
mod peekable;

use {
    crate::{
        hash::Hash,
        format::{
            Debug, Display,
        },
        axo_error::Error,
        axo_form::{
            form::Form
        },
        axo_parser::{
            Element, Symbol
        },
        axo_scanner::{
            Character, Token
        },
    }
};

pub use {
    span::*,
    position::*,
    peekable::*,
};

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Character {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Element {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Symbol {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<E: Display> Spanned for Error<E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Span {
    fn span(&self) -> Span {
        self.clone()
    }
}

impl<Item: Spanned> Spanned for Vec<Item> {
    fn span(&self) -> Span {
        if self.len() >= 2 {
            let start = self.first().unwrap().span();
            let end = self.last().unwrap().span();

            Span::mix(&start, &end)
        } else if self.len() == 1 {
            self[0].span()
        } else {
            Span::default()
        }
    }
}
