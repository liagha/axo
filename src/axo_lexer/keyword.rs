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
    Return,
    Break,
    Let,
    Continue,
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
            KeywordKind::Return => "return",
            KeywordKind::Break => "break",
            KeywordKind::Continue => "continue",
            KeywordKind::Let => "let",
        };
        write!(f, "{}", keyword_str)
    }
}
