use {
    crate::{
        axo_scanner::{Token, TokenKind},
        format::{Debug, Formatter, Result},
    }
};

impl Debug for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "Boolean({})", b),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Integer(n) => write!(f, "Integer({})", n),
            TokenKind::Operator(op) => write!(f, "Operator({:?})", op),
            TokenKind::Punctuation(pun) => write!(f, "Punctuation({:?})", pun),
            TokenKind::Identifier(var) => write!(f, "Identifier({})", var),
            TokenKind::String(string) => write!(f, "String({})", string),
            TokenKind::Character(char) => write!(f, "Char('{}')", char),
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
