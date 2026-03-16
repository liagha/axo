use crate::{
    data::Str,
    format::{
        Show, Verbosity,
        Debug, Display,
        Formatter, Result
    },
    scanner::TokenKind,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    ExpectedName,
    ExpectedHead,
    ExpectedBody,
    ExpectedAnnotation,
    MissingSeparator(TokenKind<'error>),
    UnclosedDelimiter(TokenKind<'error>),
    UnexpectedPunctuation,
    UnexpectedToken(TokenKind<'error>),
}

impl<'error> Show<'error> for ErrorKind<'error> {
    fn format(&self, verbosity: Verbosity) -> Str<'error> {
        match self {
            ErrorKind::ExpectedName => "expected name.".to_string(),
            ErrorKind::ExpectedHead => "expected head.".to_string(),
            ErrorKind::ExpectedBody => "expected body.".to_string(),
            ErrorKind::ExpectedAnnotation => "expected annotation.".to_string(),
            ErrorKind::UnexpectedPunctuation => "unexpected punctuation.".to_string(),
            ErrorKind::MissingSeparator(kind) => {
                format!("expected separator `{}`.", kind.format(verbosity))
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                format!("unclosed delimiter `{}`.", delimiter.format(verbosity))
            }
            ErrorKind::UnexpectedToken(token) => {
                format!("unexpected token `{}`.", token.format(verbosity))
            }
        }.into()
    }
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(Verbosity::Minimal))
    }
}

impl<'error> Debug for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(Verbosity::Minimal))
    }
}
