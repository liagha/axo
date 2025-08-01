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
            Element, Symbolic
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
    #[track_caller]
    fn span(&self) -> Span;
}

impl Spanned for Character {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Token {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Element {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl<E: Display> Spanned for Error<E> {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for Span {
    #[track_caller]
    fn span(&self) -> Span {
        *self
    }
}

impl<T: Spanned> Spanned for &T {
    #[track_caller]
    fn span(&self) -> Span {
        (*self).span()
    }
}

impl<T: Spanned> Spanned for &mut T {
    #[track_caller]
    fn span(&self) -> Span {
        (**self).span()
    }
}

impl<T: Spanned> Spanned for Box<T> {
    #[track_caller]
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}

impl<T: Spanned> Spanned for Vec<T> {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_slice())
    }
}

impl<T: Spanned> Spanned for &[T] {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self)
    }
}

impl<T: Spanned> Spanned for Box<[T]> {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_ref())
    }
}

impl<T: Spanned, const N: usize> Spanned for [T; N] {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_slice())
    }
}