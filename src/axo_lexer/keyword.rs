#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeywordKind {
    Use,
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
    Loop,
    While,
    Fn,
    Return,
    Break,
    Let,
    Continue,
}

pub trait KeywordLexer {
    fn to_keyword(&self) -> Option<KeywordKind>;
}

impl<T> KeywordLexer for T where T: AsRef<str> {
    fn to_keyword(&self) -> Option<KeywordKind> {
        match self.as_ref() {
            "use" => Some(KeywordKind::Use),
            "const" => Some(KeywordKind::Const),
            "extern" => Some(KeywordKind::Extern),
            "macro" => Some(KeywordKind::Macro),
            "break" => Some(KeywordKind::Break),
            "continue" => Some(KeywordKind::Continue),
            "else" => Some(KeywordKind::Else),
            "enum" => Some(KeywordKind::Enum),
            "fn" => Some(KeywordKind::Fn),
            "for" => Some(KeywordKind::For),
            "if" => Some(KeywordKind::If),
            "loop" => Some(KeywordKind::Loop),
            "impl" => Some(KeywordKind::Impl),
            "let" => Some(KeywordKind::Let),
            "match" => Some(KeywordKind::Match),
            "return" => Some(KeywordKind::Return),
            "struct" => Some(KeywordKind::Struct),
            "trait" => Some(KeywordKind::Trait),
            "while" => Some(KeywordKind::While),
            _ => None,
        }
    }
}

impl core::fmt::Display for KeywordKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let keyword_str = match self {
            KeywordKind::Use => "use",
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
            KeywordKind::Loop => "loop",
            KeywordKind::Impl => "impl",
            KeywordKind::Let => "let",
            KeywordKind::Match => "match",
            KeywordKind::Return => "return",
            KeywordKind::Struct => "struct",
            KeywordKind::Trait => "trait",
            KeywordKind::While => "while",
        };
        write!(f, "{}", keyword_str)
    }
}
