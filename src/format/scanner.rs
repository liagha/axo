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
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!("{}", self.value).into(),
            Verbosity::Detailed => format!("Character({})", self.value).into(),
            Verbosity::Debug => format!(
                "Character {{\n    value: {},\n    span: {:?}\n}}",
                self.value, self.span
            ).into(),
        }
    }
}

impl<'token> Show<'token> for Token<'token> {
    fn format(&self, verbosity: Verbosity) -> Str<'token> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => self.kind.format(verbosity).into(),
            Verbosity::Detailed => format!("Token({})", self.kind.format(verbosity)).into(),
            Verbosity::Debug => format!(
                "Token {{\n{}\n}}",
                format!("kind: {}", self.kind.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'token> Show<'token> for TokenKind<'token> {
    fn format(&self, verbosity: Verbosity) -> Str<'token> {
        if verbosity == Verbosity::Off {
            return "".into();
        }

        match self {
            TokenKind::Boolean(boolean) => match verbosity {
                Verbosity::Minimal => format!("{}", boolean).into(),
                Verbosity::Detailed => format!("Boolean({})", boolean).into(),
                Verbosity::Debug => format!("Boolean {{\n    value: {}\n}}", boolean).into(),
                _ => "".into(),
            },
            TokenKind::Float(number) => match verbosity {
                Verbosity::Minimal => format!("{}", number).into(),
                Verbosity::Detailed => format!("Float({})", number).into(),
                Verbosity::Debug => format!("Float {{\n    value: {}\n}}", number).into(),
                _ => "".into(),
            },
            TokenKind::Integer(number) => match verbosity {
                Verbosity::Minimal => format!("{}", number).into(),
                Verbosity::Detailed => format!("Integer({})", number).into(),
                Verbosity::Debug => format!("Integer {{\n    value: {}\n}}", number).into(),
                _ => "".into(),
            },
            TokenKind::Operator(operator) => match verbosity {
                Verbosity::Minimal => format!("{:?}", operator).into(),
                Verbosity::Detailed => format!("Operator({:?})", operator).into(),
                Verbosity::Debug => format!("Operator {{\n    value: {:?}\n}}", operator).into(),
                _ => "".into(),
            },
            TokenKind::Punctuation(punctuation) => match verbosity {
                Verbosity::Minimal => format!("{:?}", punctuation).into(),
                Verbosity::Detailed => format!("Punctuation({:?})", punctuation).into(),
                Verbosity::Debug => format!("Punctuation {{\n    value: {:?}\n}}", punctuation).into(),
                _ => "".into(),
            },
            TokenKind::Identifier(identifier) => match verbosity {
                Verbosity::Minimal => format!("{}", identifier).into(),
                Verbosity::Detailed => format!("Identifier({})", identifier).into(),
                Verbosity::Debug => format!("Identifier {{\n    value: {}\n}}", identifier).into(),
                _ => "".into(),
            },
            TokenKind::String(string) => match verbosity {
                Verbosity::Minimal => format!("\"{}\"", string).into(),
                Verbosity::Detailed => format!("String(\"{}\")", string).into(),
                Verbosity::Debug => format!("String {{\n    value: \"{}\"\n}}", string).into(),
                _ => "".into(),
            },
            TokenKind::Character(character) => match verbosity {
                Verbosity::Minimal => format!("'{}'", character).into(),
                Verbosity::Detailed => format!("Character('{}')", character).into(),
                Verbosity::Debug => format!("Character {{\n    value: '{}'\n}}", character).into(),
                _ => "".into(),
            },
            TokenKind::Comment(comment) => match verbosity {
                Verbosity::Minimal => format!("//{}", comment).into(),
                Verbosity::Detailed => format!("Comment({})", comment).into(),
                Verbosity::Debug => format!("Comment {{\n    value: \"{}\"\n}}", comment).into(),
                _ => "".into(),
            },
        }
    }
}
