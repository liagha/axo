#![allow(dead_code)]

use core::fmt;

#[derive(Clone, PartialEq)]
pub enum Token {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    Str(String),
    Operator(Operator),
    Identifier(String),
    Char(char),
    Punctuation(Punctuation),
    Keyword(Keyword),
    Invalid(String),
    EOF,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Tilde,              // ~
    Equal,              // =
    Colon,              // :
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    Caret,              // ^
    Pipe,               // |
    Ampersand,          // &
    Percent,            // %
    GreaterThan,        // >
    LessThan,           // <
    Exclamation,        // !
    Comma,              // ,
    Dot,                // .
    DotDot,             // ..
    LessThanOrEqual,    // <=
    GreaterThanOrEqual, // >=
    EqualEqual,         // ==
    NotEqual,           // !=
    LogicalAnd,         // &&
    LogicalOr,          // ||
    LeftShift,          // <<
    RightShift,         // >>
    PlusEqual,          // +=
    MinusEqual,         // -=
    StarEqual,          // *=
    SlashEqual,         // /=
    PercentEqual,       // %=
    CaretEqual,         // ^=
    AmpersandEqual,     // &=
    PipeEqual,          // |=
    LogicalAndEqual,    // &&=
    LogicalOrEqual,     // ||=
    QuestionMarkEqual,  // ?=
    DotDotEqual,        // ..=
    Arrow,              // ->
    FatArrow,           // =>
    At,                 // @
    Hash,               // #
    QuestionMark,       // ?
    Dollar,             // $
    Backslash,          // \
    DoubleQuote,        // "
    SingleQuote,        // '
    Backtick,           // `
    Underscore,         // _
    DoubleColon,        // ::
    DoubleQuestionMark, // ??
    DoubleExclamation,  // !!
    DoubleStar,         // **
    DoubleSlash,        // //
    SlashStar,          // /*
    StarSlash,          // */
    DoublePercent,      // %%
    DoubleCaret,        // ^^
    DoubleTilde,        // ~~
    DoubleAt,           // @@
    DoubleHash,         // ##
    DoubleDollar,       // $$
    DoubleBackslash,    // \\
    DoubleUnderscore,   // __
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Punctuation {
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

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
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

impl Operator {
    pub fn from_str(s: &str) -> Self {
        match s {
            "~" => Operator::Tilde,
            "=" => Operator::Equal,
            ":" => Operator::Colon,
            "+" => Operator::Plus,
            "-" => Operator::Minus,
            "*" => Operator::Star,
            "/" => Operator::Slash,
            "^" => Operator::Caret,
            "|" => Operator::Pipe,
            "&" => Operator::Ampersand,
            "%" => Operator::Percent,
            ">" => Operator::GreaterThan,
            "<" => Operator::LessThan,
            "!" => Operator::Exclamation,
            "," => Operator::Comma,
            "." => Operator::Dot,
            ".." => Operator::DotDot,
            "<=" => Operator::LessThanOrEqual,
            ">=" => Operator::GreaterThanOrEqual,
            "==" => Operator::EqualEqual,
            "!=" => Operator::NotEqual,
            "&&" => Operator::LogicalAnd,
            "||" => Operator::LogicalOr,
            "<<" => Operator::LeftShift,
            ">>" => Operator::RightShift,
            "+=" => Operator::PlusEqual,
            "-=" => Operator::MinusEqual,
            "*=" => Operator::StarEqual,
            "/=" => Operator::SlashEqual,
            "%=" => Operator::PercentEqual,
            "?" => Operator::QuestionMark,
            "..=" => Operator::DotDotEqual,
            "->" => Operator::Arrow,
            "=>" => Operator::FatArrow,
            "@" => Operator::At,
            "#" => Operator::Hash,
            "$" => Operator::Dollar,
            "\\" => Operator::Backslash,
            "\"" => Operator::DoubleQuote,
            "'" => Operator::SingleQuote,
            "`" => Operator::Backtick,
            "_" => Operator::Underscore,
            "::" => Operator::DoubleColon,
            "??" => Operator::DoubleQuestionMark,
            "!!" => Operator::DoubleExclamation,
            "**" => Operator::DoubleStar,
            "//" => Operator::DoubleSlash,
            "/*" => Operator::SlashStar,
            "*/" => Operator::StarSlash,
            "%%" => Operator::DoublePercent,
            "^^" => Operator::DoubleCaret,
            "~~" => Operator::DoubleTilde,
            "@@" => Operator::DoubleAt,
            "##" => Operator::DoubleHash,
            "$$" => Operator::DoubleDollar,
            "\\\\" => Operator::DoubleBackslash,
            "__" => Operator::DoubleUnderscore,
            "^=" => Operator::CaretEqual,
            "&=" => Operator::AmpersandEqual,
            "|=" => Operator::PipeEqual,
            "&&=" => Operator::LogicalAndEqual,
            "||=" => Operator::LogicalOrEqual,
            "?=" => Operator::QuestionMarkEqual,
            _ => unreachable!(),
        }
    }

    pub fn is_compound(&self) -> bool {
        matches!(
            self,
            Operator::LogicalAndEqual
                | Operator::LogicalOrEqual
                | Operator::QuestionMarkEqual
                | Operator::DotDotEqual
                | Operator::AmpersandEqual
                | Operator::PipeEqual
                | Operator::StarEqual
                | Operator::SlashEqual
                | Operator::PercentEqual
                | Operator::CaretEqual
                | Operator::PlusEqual
                | Operator::MinusEqual
        )
    }

    pub fn is_compound_token(input: Option<&Token>) -> bool {
        if let Some(Token::Operator(operator)) = input {
            operator.is_compound()
        } else {
            false
        }
    }

    pub fn is_operator(char: char) -> bool {
        matches!(
            char,
            '~' | '=' | ':' | '+' | '-' |
            '*' | '/' | '^' | '|' | '&' |
            '%' | '>' | '<' | '!' | ',' |
            '.' | '@' | '\'' | '?' | '#' |
            '$' | '\\' | '`' | '_'
        )
    }

    pub fn is_factor(&self) -> bool {
        matches!(
            self,
            Operator::Star
                | Operator::Slash
                | Operator::Percent
                | Operator::DotDot
                | Operator::LogicalAnd
                | Operator::DoubleStar
        )
    }

    pub fn is_term(&self) -> bool {
        matches!(
            self,
            Operator::Plus
                | Operator::Minus
                | Operator::LogicalOr
                | Operator::Pipe
                | Operator::Caret
        )
    }

    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Operator::EqualEqual
                | Operator::NotEqual
                | Operator::GreaterThan
                | Operator::LessThan
                | Operator::GreaterThanOrEqual
                | Operator::LessThanOrEqual
        )
    }

    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            Operator::Exclamation | Operator::Minus | Operator::Tilde
        )
    }
}

impl Punctuation {
    pub fn from_str(s: &str) -> Self {
        match s {
            " " => Punctuation::Space,
            "\t" => Punctuation::Tab,
            "\n" => Punctuation::Newline,
            "\r" => Punctuation::CarriageReturn,
            "(" => Punctuation::LeftParen,
            ")" => Punctuation::RightParen,
            "[" => Punctuation::LeftBracket,
            "]" => Punctuation::RightBracket,
            "{" => Punctuation::LeftBrace,
            "}" => Punctuation::RightBrace,
            ";" => Punctuation::Semicolon,
            _ => unreachable!(),
        }
    }

    pub fn is_punctuation(char: char) -> bool {
        matches!(
            char,
            ' ' | '\t' | '\n' | '\r' | '(' | ')' | '[' | ']' | '{' | '}' | ';'
        )
    }
}

impl Token {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "true" => Some(Token::Boolean(true)),
            "false" => Some(Token::Boolean(false)),
            "struct" => Some(Token::Keyword(Keyword::Struct)),
            "enum" => Some(Token::Keyword(Keyword::Enum)),
            "impl" => Some(Token::Keyword(Keyword::Impl)),
            "trait" => Some(Token::Keyword(Keyword::Trait)),
            "match" => Some(Token::Keyword(Keyword::Match)),
            "if" => Some(Token::Keyword(Keyword::If)),
            "else" => Some(Token::Keyword(Keyword::Else)),
            "for" => Some(Token::Keyword(Keyword::For)),
            "while" => Some(Token::Keyword(Keyword::While)),
            "fn" => Some(Token::Keyword(Keyword::Fn)),
            "in" => Some(Token::Keyword(Keyword::In)),
            "return" => Some(Token::Keyword(Keyword::Return)),
            "let" => Some(Token::Keyword(Keyword::Let)),
            "continue" => Some(Token::Keyword(Keyword::Continue)),
            "break" => Some(Token::Keyword(Keyword::Break)),
            _ => None,
        }
    }

    pub fn get_operator(input: Option<&Token>) -> Option<Operator> {
        if let Some(Token::Operator(operator)) = input {
            Some(operator.clone())
        } else {
            None
        }
    }
}

impl Keyword {
    pub fn from_str(s: &str) -> Self {
        match s {
            "struct" => Keyword::Struct,
            "enum" => Keyword::Enum,
            "trait" => Keyword::Trait,
            "impl" => Keyword::Impl,
            "match" => Keyword::Match,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "for" => Keyword::For,
            "while" => Keyword::While,
            "fn" => Keyword::Fn,
            "return" => Keyword::Return,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Punctuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let punct_str = match self {
            Punctuation::Space => " ",
            Punctuation::Tab => "\t",
            Punctuation::Newline => "\n",
            Punctuation::CarriageReturn => "\r",
            Punctuation::LeftParen => "(",
            Punctuation::RightParen => ")",
            Punctuation::LeftBracket => "[",
            Punctuation::RightBracket => "]",
            Punctuation::LeftBrace => "{",
            Punctuation::RightBrace => "}",
            Punctuation::Semicolon => ";",
        };
        write!(f, "{}", punct_str)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op_str = match self {
            Operator::Tilde => "~",
            Operator::Equal => "=",
            Operator::Colon => ":",
            Operator::Plus => "+",
            Operator::Minus => "-",
            Operator::Star => "*",
            Operator::Slash => "/",
            Operator::Caret => "^",
            Operator::Pipe => "|",
            Operator::Ampersand => "&",
            Operator::Percent => "%",
            Operator::GreaterThan => ">",
            Operator::LessThan => "<",
            Operator::Exclamation => "!",
            Operator::Comma => ",",
            Operator::Dot => ".",
            Operator::DotDot => "..",
            Operator::LessThanOrEqual => "<=",
            Operator::GreaterThanOrEqual => ">=",
            Operator::EqualEqual => "==",
            Operator::NotEqual => "!=",
            Operator::LogicalAnd => "&&",
            Operator::LogicalOr => "||",
            Operator::LeftShift => "<<",
            Operator::RightShift => ">>",
            Operator::PlusEqual => "+=",
            Operator::MinusEqual => "-=",
            Operator::StarEqual => "*=",
            Operator::SlashEqual => "/=",
            Operator::PercentEqual => "%=",
            Operator::QuestionMark => "?",
            Operator::DotDotEqual => "..=",
            Operator::Arrow => "->",
            Operator::FatArrow => "=>",
            Operator::At => "@",
            Operator::Hash => "#",
            Operator::Dollar => "$",
            Operator::Backslash => "\\",
            Operator::DoubleQuote => "\"",
            Operator::SingleQuote => "'",
            Operator::Backtick => "`",
            Operator::Underscore => "_",
            Operator::DoubleColon => "::",
            Operator::DoubleQuestionMark => "??",
            Operator::DoubleExclamation => "!!",
            Operator::DoubleStar => "**",
            Operator::DoubleSlash => "//",
            Operator::SlashStar => "/*",
            Operator::StarSlash => "*/",
            Operator::DoublePercent => "%%",
            Operator::DoubleCaret => "^^",
            Operator::DoubleTilde => "~~",
            Operator::DoubleAt => "@@",
            Operator::DoubleHash => "##",
            Operator::DoubleDollar => "$$",
            Operator::DoubleBackslash => "\\\\",
            Operator::DoubleUnderscore => "__",
            Operator::CaretEqual => "^=",
            Operator::AmpersandEqual => "&=",
            Operator::PipeEqual => "|=",
            Operator::LogicalAndEqual => "&&=",
            Operator::LogicalOrEqual => "||=",
            Operator::QuestionMarkEqual => "?=",
        };
        write!(f, "{}", op_str)
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keyword_str = match self {
            Keyword::Struct => "struct",
            Keyword::Enum => "enum",
            Keyword::Trait => "trait",
            Keyword::Impl => "impl",
            Keyword::Match => "match",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::For => "for",
            Keyword::While => "while",
            Keyword::Fn => "fn",
            Keyword::In => "in",
            Keyword::Return => "return",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Let => "let",
        };
        write!(f, "{}", keyword_str)
    }
}
