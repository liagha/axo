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
    Semicolon,
}

impl PunctuationKind {
    pub fn from_str(s: &str) -> Self {
        match s {
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
            ";" => PunctuationKind::Semicolon,
            _ => unreachable!(),
        }
    }

    pub fn from_char(c: &char) -> Self {
        match c.clone() {
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
            ';' => PunctuationKind::Semicolon,
            _ => unreachable!(),
        }
    }

    pub fn is_punctuation_str(str: &str) -> bool {
        matches!(
            str,
            " " | "\t" | "\n" | "\r" | "(" | ")" | "[" | "]" | "{" | "}" | ";"
        )
    }

    pub fn is_punctuation(char: char) -> bool {
        if char == '\n' { return true; }

        matches!(
            char,
            ' ' | '\t' | '\n' | '\r' | '(' | ')' | '[' | ']' | '{' | '}' | ';'
        )
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
            PunctuationKind::Semicolon => ";",
        };
        write!(f, "{}", punct_str)
    }
}
