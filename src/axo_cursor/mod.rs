#![allow(unused_imports)]

mod span;
mod format;
mod position;
mod peekable;

use std::ops::Deref;
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

pub trait Spanned<'spanned> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'spanned>;

    #[track_caller]
    fn span(self) -> Span<'spanned>;
}

impl<'character> Spanned<'character> for Character<'character> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'character> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'character> {
        self.span
    }
}

impl<'token> Spanned<'token> for Token<'token> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'token> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'token> {
        self.span
    }
}

impl<'element> Spanned<'element> for Element<'element> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'element> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'element> {
        self.span
    }
}

impl<'error, E: Display> Spanned<'error> for Error<'error, E> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'error> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'error> {
        self.span
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for Vec<T> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'item> {
        Span::from_slice(self.as_slice())
    }

    #[track_caller]
    fn span(self) -> Span<'item> {
        Span::from_slice(self.as_slice())
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for &[T] {
    #[track_caller]
    fn borrow_span(&self) -> Span<'item> {
        Span::from_slice(self)
    }

    #[track_caller]
    fn span(self) -> Span<'item> {
        Span::from_slice(self)
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for Box<[T]> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'item> {
        Span::from_slice(self.as_ref())
    }

    #[track_caller]
    fn span(self) -> Span<'item> {
        self.as_ref().span()
    }
}

impl<'item, T: Spanned<'item>, const N: usize> Spanned<'item> for [T; N] {
    #[track_caller]
    fn borrow_span(&self) -> Span<'item> {
        Span::from_slice(self.as_slice())
    }

    #[track_caller]
    fn span(self) -> Span<'item> {
        Span::from_slice(self.as_slice())
    }
}