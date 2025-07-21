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
        self.span
    }
}

impl<E: Display> Spanned for Error<E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl<T: Spanned> Spanned for &T {
    fn span(&self) -> Span {
        (*self).span()
    }
}

impl<T: Spanned> Spanned for &mut T {
    fn span(&self) -> Span {
        (**self).span()
    }
}

impl<T: Spanned> Spanned for Box<T> {
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}

fn span_from_slice<T: Spanned>(items: &[T]) -> Span {
    match items.len() {
        0 => Span::default(),
        1 => items[0].span(),
        _ => {
            let start = items.first().unwrap().span();
            let end = items.last().unwrap().span();
            start.merge(&end)
        }
    }
}

impl<T: Spanned> Spanned for Vec<T> {
    fn span(&self) -> Span {
        span_from_slice(self.as_slice())
    }
}

impl<T: Spanned> Spanned for &[T] {
    fn span(&self) -> Span {
        span_from_slice(self)
    }
}

impl<T: Spanned> Spanned for Box<[T]> {
    fn span(&self) -> Span {
        span_from_slice(self.as_ref())
    }
}

impl<T: Spanned, const N: usize> Spanned for [T; N] {
    fn span(&self) -> Span {
        span_from_slice(self.as_slice())
    }
}