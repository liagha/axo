use {
    super::{Character, Token},
    crate::{
        tracker::{Span, Spanned},
    },
};

impl<'token> PartialEq for Token<'token> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'token> Eq for Token<'token> {}

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
