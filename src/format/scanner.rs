use {
    crate::{
        data::Str,
        format::{Show, Verbosity},
        scanner::{Character, Token, TokenKind},
    }
};

impl<'character> Show<'character> for Character<'character> {
    fn format(&self, verbosity: Verbosity) -> Str<'character> {
        match verbosity {
            Verbosity::Minimal => {
                format!("Character({})", self.value)
            }

            Verbosity::Detailed => {
                format!("Character({}, {:?})", self.value, self.span)
            }

            _ => {
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}

impl<'token> Show<'token> for Token<'token> {
    fn format(&self, verbosity: Verbosity) -> Str<'token> {
        match verbosity {
            Verbosity::Minimal => {
                format!("{}", self.kind.format(verbosity))
            }

            _ => {
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}

impl<'token> Show<'token> for TokenKind<'token> {
    fn format(&self, verbosity: Verbosity) -> Str<'token> {
        match verbosity {
            Verbosity::Minimal => {
                match self {
                    TokenKind::Boolean(boolean) => format!("{}", boolean),
                    TokenKind::Float(number) => format!("{}", number),
                    TokenKind::Integer(number) => format!("{}", number),
                    TokenKind::Operator(operator) => format!("{:?}", operator),
                    TokenKind::Punctuation(punctuation) => format!("{:?}", punctuation),
                    TokenKind::Identifier(identifier) => format!("{}", identifier),
                    TokenKind::String(string) => format!("\"{}\"", string),
                    TokenKind::Character(character) => format!("'{}'", character),
                    TokenKind::Comment(comment) => format!("//{}", comment),
                }
            }

            Verbosity::Detailed => {
                match self {
                    TokenKind::Boolean(boolean) => format!("Boolean({})", boolean),
                    TokenKind::Float(number) => format!("Float({})", number),
                    TokenKind::Integer(number) => format!("Integer({})", number),
                    TokenKind::Operator(operator) => format!("Operator({:?})", operator),
                    TokenKind::Punctuation(punctuation) => format!("Punctuation({:?})", punctuation),
                    TokenKind::Identifier(identifier) => format!("Identifier({})", identifier),
                    TokenKind::String(string) => format!("String({})", string),
                    TokenKind::Character(character) => format!("Character('{}')", character),
                    TokenKind::Comment(comment) => format!("Comment({})", comment),
                }
            }

            _ => {
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}
