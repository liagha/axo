mod format;
mod peekable;
mod position;
mod span;

use {
    crate::{
        format::Display,
        reporter::Error,
    },
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