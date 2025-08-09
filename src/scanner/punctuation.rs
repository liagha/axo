use {
    super::Character,
    crate::{
        format::{
            self,
            Display, Debug, 
            Formatter
        },
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

pub trait Punctuation {
    fn is_punctuation(&self) -> bool;
    fn to_punctuation(&self) -> PunctuationKind;
}

impl Punctuation for str {
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

impl<'character> Punctuation for Character<'character> {
    fn is_punctuation(&self) -> bool {
        matches!(
            self.value,
            ' ' | '\t' | '\n' | '\r' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
        )
    }

    fn to_punctuation(&self) -> PunctuationKind {
        match self.value {
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

impl Punctuation for char {
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
    fn fmt(&self, f: &mut Formatter) -> format::Result {
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
