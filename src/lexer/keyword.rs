#[derive(Clone, Debug, PartialEq)]
pub enum KeywordKind {
    Struct,
    Enum,
    Impl,
    Trait,
    Match,
    If,
    Else,
    For,
    While,
    Fn,
    In,
    Return,
    Break,
    Let,
    Continue,
}

impl KeywordKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "struct" => KeywordKind::Struct,
            "enum" => KeywordKind::Enum,
            "trait" => KeywordKind::Trait,
            "impl" => KeywordKind::Impl,
            "match" => KeywordKind::Match,
            "if" => KeywordKind::If,
            "else" => KeywordKind::Else,
            "for" => KeywordKind::For,
            "while" => KeywordKind::While,
            "fn" => KeywordKind::Fn,
            "return" => KeywordKind::Return,
            _ => unreachable!(),
        }
    }
}

impl core::fmt::Display for KeywordKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let keyword_str = match self {
            KeywordKind::Struct => "struct",
            KeywordKind::Enum => "enum",
            KeywordKind::Trait => "trait",
            KeywordKind::Impl => "impl",
            KeywordKind::Match => "match",
            KeywordKind::If => "if",
            KeywordKind::Else => "else",
            KeywordKind::For => "for",
            KeywordKind::While => "while",
            KeywordKind::Fn => "fn",
            KeywordKind::In => "in",
            KeywordKind::Return => "return",
            KeywordKind::Break => "break",
            KeywordKind::Continue => "continue",
            KeywordKind::Let => "let",
        };
        write!(f, "{}", keyword_str)
    }
}
