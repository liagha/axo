pub use crate::format::{Display, Debug, Formatter, Write};

#[derive(Clone, Debug, PartialEq, Copy, Eq, Hash)]
pub enum PunctuationKind {
    Space,
    Tab,
    Newline,
    CarriageReturn,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    SemiColon,
}

pub trait PunctuationLexer {
    fn is_punctuation(&self) -> bool;
    fn to_punctuation(&self) -> PunctuationKind;
}

impl PunctuationLexer for str {
    fn is_punctuation(&self) -> bool {
        matches!(
            self,
            " " | "\t" | "\n" | "\r" | "(" | ")" | "[" | "]" | "{" | "}" | "," | ";"
        )
    }

    fn to_punctuation(&self) -> PunctuationKind {
        match self {
            " " => PunctuationKind::Space,
            "\t" => PunctuationKind::Tab,
            "\n" => PunctuationKind::Newline,
            "\r" => PunctuationKind::CarriageReturn,
            "(" => PunctuationKind::LeftParenthesis,
            ")" => PunctuationKind::RightParenthesis,
            "[" => PunctuationKind::LeftBracket,
            "]" => PunctuationKind::RightBracket,
            "{" => PunctuationKind::LeftBrace,
            "}" => PunctuationKind::RightBrace,
            "," => PunctuationKind::Comma,
            ";" => PunctuationKind::SemiColon,
            _ => unreachable!(),
        }
    }
}

impl PunctuationLexer for char {
    fn is_punctuation(&self) -> bool {
        matches!(
            self,
            ' ' | '\t' | '\n' | '\r' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
        )
    }

    fn to_punctuation(&self) -> PunctuationKind {
        match self {
            ' ' => PunctuationKind::Space,
            '\t' => PunctuationKind::Tab,
            '\n' => PunctuationKind::Newline,
            '\r' => PunctuationKind::CarriageReturn,
            '(' => PunctuationKind::LeftParenthesis,
            ')' => PunctuationKind::RightParenthesis,
            '[' => PunctuationKind::LeftBracket,
            ']' => PunctuationKind::RightBracket,
            '{' => PunctuationKind::LeftBrace,
            '}' => PunctuationKind::RightBrace,
            ',' => PunctuationKind::Comma,
            ';' => PunctuationKind::SemiColon,
            _ => unreachable!(),
        }
    }
}

impl Display for PunctuationKind {
    fn fmt(&self, f: &mut Formatter) -> crate::format::Result {
        let punct_str = match self {
            PunctuationKind::Space => " ",
            PunctuationKind::Tab => "\t",
            PunctuationKind::Newline => "\n",
            PunctuationKind::CarriageReturn => "\r",
            PunctuationKind::LeftParenthesis => "(",
            PunctuationKind::RightParenthesis => ")",
            PunctuationKind::LeftBracket => "[",
            PunctuationKind::RightBracket => "]",
            PunctuationKind::LeftBrace => "{",
            PunctuationKind::RightBrace => "}",
            PunctuationKind::Comma => ",",
            PunctuationKind::SemiColon => ";",
        };
        write!(f, "{}", punct_str)
    }
}
