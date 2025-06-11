use {
    crate::{
        format::{Display, Debug, Formatter, Result},
        axo_scanner::{
            Token, TokenKind,
            PunctuationKind,
        }
    }
};

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "{}", b),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Punctuation(c) => {
                if c == &PunctuationKind::Newline {
                    return write!(f, "\n")
                }
                write!(f, "{}", c)
            },
            TokenKind::Operator(c) => write!(f, "{}", c),
            TokenKind::String(str) => write!(f, "{}", str),
            TokenKind::Identifier(str) => write!(f, "{}", str),
            TokenKind::Character(char) => write!(f, "'{}'", char),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
        }
    }
}

impl Debug for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "Boolean({})", b),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Integer(n) => write!(f, "Integer({})", n),
            TokenKind::Operator(op) => write!(f, "Operator({:?})", op),
            TokenKind::Punctuation(pun) => write!(f, "Punctuation({:?})", pun),
            TokenKind::Identifier(var) => write!(f, "Identifier({})", var),
            TokenKind::String(str) => write!(f, "String({})", str),
            TokenKind::Character(char) => write!(f, "Char('{}')", char),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.kind)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.kind)
    }
}

