use {
    super::{
        Character, Token, TokenKind,
    },
    crate::{
        tracker::{Span, Spanned},
        format::{Display, Debug, Formatter, Result},
    }
};

impl<'token> Display for TokenKind<'token> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TokenKind::Boolean(boolean) => write!(f, "{}", boolean),
            TokenKind::Float(number) => write!(f, "{}", number),
            TokenKind::Integer(number) => write!(f, "{}", number),
            TokenKind::Operator(operator) => write!(f, "{:?}", operator),
            TokenKind::Punctuation(punctuation) => write!(f, "{:?}", punctuation),
            TokenKind::Identifier(identifier) => write!(f, "{}", identifier),
            TokenKind::String(string) => write!(f, "\"{}\"", string),
            TokenKind::Character(character) => write!(f, "'{}'", character),
            TokenKind::Comment(comment) => write!(f, "//{}", comment),
        }
    }
}

impl<'token> Display for Token<'token> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.kind)
    }
}

impl<'token> Debug for TokenKind<'token> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TokenKind::Boolean(boolean) => write!(f, "Boolean({})", boolean),
            TokenKind::Float(number) => write!(f, "Float({})", number),
            TokenKind::Integer(number) => write!(f, "Integer({})", number),
            TokenKind::Operator(operator) => write!(f, "Operator({:?})", operator),
            TokenKind::Punctuation(punctuation) => write!(f, "Punctuation({:?})", punctuation),
            TokenKind::Identifier(identifier) => write!(f, "Identifier({})", identifier),
            TokenKind::String(string) => write!(f, "String({})", string),
            TokenKind::Character(character) => write!(f, "Character('{}')", character),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
        }
    }
}

impl<'token> Debug for Token<'token> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{:?} | {:?}", self.kind, self.span)
        }
    }
}

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


