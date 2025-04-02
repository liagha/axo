#[derive(Clone, Debug, PartialEq)]
pub enum KeywordKind {
    Const,
    Extern,
    Macro,
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
    Unknown,
}

pub trait KeywordLexer {
    fn to_keyword(&self) -> KeywordKind;
}

impl<T> KeywordLexer for T where T: AsRef<str> {
    fn to_keyword(&self) -> KeywordKind {
        match self.as_ref() {
            "const" => KeywordKind::Const,
            "extern" => KeywordKind::Extern,
            "macro" => KeywordKind::Macro,
            "break" => KeywordKind::Break,
            "continue" => KeywordKind::Continue,
            "else" => KeywordKind::Else,
            "enum" => KeywordKind::Enum,
            "fn" => KeywordKind::Fn,
            "for" => KeywordKind::For,
            "if" => KeywordKind::If,
            "impl" => KeywordKind::Impl,
            "let" => KeywordKind::Let,
            "match" => KeywordKind::Match,
            "return" => KeywordKind::Return,
            "struct" => KeywordKind::Struct,
            "trait" => KeywordKind::Trait,
            "while" => KeywordKind::While,
            _ => KeywordKind::Unknown,
        }
    }
}

impl core::fmt::Display for KeywordKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let keyword_str = match self {
            KeywordKind::Const => "const",
            KeywordKind::Extern => "extern",
            KeywordKind::Macro => "macro",
            KeywordKind::Break => "break",
            KeywordKind::Continue => "continue",
            KeywordKind::Else => "else",
            KeywordKind::Enum => "enum",
            KeywordKind::Fn => "fn",
            KeywordKind::For => "for",
            KeywordKind::If => "if",
            KeywordKind::Impl => "impl",
            KeywordKind::Let => "let",
            KeywordKind::Match => "match",
            KeywordKind::Return => "return",
            KeywordKind::Struct => "struct",
            KeywordKind::Trait => "trait",
            KeywordKind::While => "while",
            KeywordKind::Unknown => "????",
        };
        write!(f, "{}", keyword_str)
    }
}
