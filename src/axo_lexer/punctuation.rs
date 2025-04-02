#[derive(Clone, Debug, PartialEq, Copy)]
pub enum PunctuationKind {
    Space,
    Tab,
    Newline,
    CarriageReturn,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
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
            "(" => PunctuationKind::LeftParen,
            ")" => PunctuationKind::RightParen,
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
            '\r' => PunctuationKind::CarriageReturn,
            '(' => PunctuationKind::LeftParen,
            ')' => PunctuationKind::RightParen,
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

impl core::fmt::Display for PunctuationKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let punct_str = match self {
            PunctuationKind::Space => " ",
            PunctuationKind::Tab => "\t",
            PunctuationKind::Newline => "\n",
            PunctuationKind::CarriageReturn => "\r",
            PunctuationKind::LeftParen => "(",
            PunctuationKind::RightParen => ")",
            PunctuationKind::LeftBracket => "[",
            PunctuationKind::RightBracket => "]",
            PunctuationKind::LeftBrace => "{",
            PunctuationKind::RightBrace => "}",
            PunctuationKind::Comma => ",",
            PunctuationKind::Semicolon => ";",
        };
        write!(f, "{}", punct_str)
    }
}
