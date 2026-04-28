use crate::{
    scanner::{Character, Token},
    tracker::{Span, Spanned},
};

impl<'token> PartialEq for Token<'token> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'token> Eq for Token<'token> {}

impl<'character> Spanned<'character> for Character {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'token> Spanned<'token> for Token<'token> {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}
