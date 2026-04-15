pub mod error;
mod peekable;
mod position;
mod span;
mod format;

use crate::{data::Scale, format::Display, reporter::Error};

pub use {error::*, peekable::*, position::*, span::*};

pub trait Spanned<'spanned> {
    #[track_caller]
    fn span(&self) -> Span;
}

impl<'error, E> Spanned<'error> for Error<'error, E>
where
    E: Clone + Display,
{
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for Vec<T> {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_slice())
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for &[T] {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self)
    }
}

impl<'item, T: Spanned<'item>> Spanned<'item> for Box<[T]> {
    #[track_caller]
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}

impl<'item, T: Spanned<'item>, const N: Scale> Spanned<'item> for [T; N] {
    #[track_caller]
    fn span(&self) -> Span {
        Span::from_slice(self.as_slice())
    }
}

pub type TrackError<'error> = Error<'error, ErrorKind<'error>>;
