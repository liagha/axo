use {
    crate::{
        format::{
            Display, Debug, 
            Formatter
        },
    },
};

#[derive(Clone, Debug, PartialEq, Copy, Eq, Hash)]
pub enum PunctuationKind {
    Space,
    Indentation(usize),
    Tab,
    Newline,
    Return,
    Comma,
    Semicolon,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
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
            "\r" => PunctuationKind::Return,
            "(" => PunctuationKind::LeftParenthesis,
            ")" => PunctuationKind::RightParenthesis,
            "[" => PunctuationKind::LeftBracket,
            "]" => PunctuationKind::RightBracket,
            "{" => PunctuationKind::LeftBrace,
            "}" => PunctuationKind::RightBrace,
            "," => PunctuationKind::Comma,
            ";" => PunctuationKind::Semicolon,
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
            '\r' => PunctuationKind::Return,
            '(' => PunctuationKind::LeftParenthesis,
            ')' => PunctuationKind::RightParenthesis,
            '[' => PunctuationKind::LeftBracket,
            ']' => PunctuationKind::RightBracket,
            '{' => PunctuationKind::LeftBrace,
            '}' => PunctuationKind::RightBrace,
            ',' => PunctuationKind::Comma,
            ';' => PunctuationKind::Semicolon,
            _ => unreachable!(),
        }
    }
}

impl Display for PunctuationKind {
    fn fmt(&self, f: &mut Formatter) -> crate::format::Result {
        let punctuation = match self {
            PunctuationKind::Space => " ",
            PunctuationKind::Indentation(size) => &*" ".repeat(size.clone()),
            PunctuationKind::Tab => "\t",
            PunctuationKind::Newline => "\n",
            PunctuationKind::Return => "\r",
            PunctuationKind::LeftParenthesis => "(",
            PunctuationKind::RightParenthesis => ")",
            PunctuationKind::LeftBracket => "[",
            PunctuationKind::RightBracket => "]",
            PunctuationKind::LeftBrace => "{",
            PunctuationKind::RightBrace => "}",
            PunctuationKind::Comma => ",",
            PunctuationKind::Semicolon => ";",
        };

        write!(f, "{}", punctuation)
    }
}
