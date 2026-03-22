use {
    crate::{
        scanner::Character,
        format::{self, Debug, Display, Formatter}
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

use crate::internal::cache::{Encode, Decode};

impl Encode for PunctuationKind {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            PunctuationKind::Space => buffer.push(0),
            PunctuationKind::Indentation(size) => {
                buffer.push(1);
                size.encode(buffer);
            }
            PunctuationKind::Tab => buffer.push(2),
            PunctuationKind::Newline => buffer.push(3),
            PunctuationKind::Return => buffer.push(4),
            PunctuationKind::Comma => buffer.push(5),
            PunctuationKind::Semicolon => buffer.push(6),
            PunctuationKind::LeftParenthesis => buffer.push(7),
            PunctuationKind::RightParenthesis => buffer.push(8),
            PunctuationKind::LeftBracket => buffer.push(9),
            PunctuationKind::RightBracket => buffer.push(10),
            PunctuationKind::LeftBrace => buffer.push(11),
            PunctuationKind::RightBrace => buffer.push(12),
        }
    }
}

impl<'a> Decode<'a> for PunctuationKind {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => PunctuationKind::Space,
            1 => PunctuationKind::Indentation(usize::decode(buffer, cursor)),
            2 => PunctuationKind::Tab,
            3 => PunctuationKind::Newline,
            4 => PunctuationKind::Return,
            5 => PunctuationKind::Comma,
            6 => PunctuationKind::Semicolon,
            7 => PunctuationKind::LeftParenthesis,
            8 => PunctuationKind::RightParenthesis,
            9 => PunctuationKind::LeftBracket,
            10 => PunctuationKind::RightBracket,
            11 => PunctuationKind::LeftBrace,
            12 => PunctuationKind::RightBrace,
            _ => panic!(),
        }
    }
}