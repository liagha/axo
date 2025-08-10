use {
    super::{Token, TokenKind},
    crate::{
        format::{Debug, Formatter, Result},
    }
};

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
