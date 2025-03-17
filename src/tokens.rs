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
    Tilde,
    Equal,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Pipe,
    Ampersand,
    Percent,
    GreaterThan,
    LessThan,
    Exclamation,
    Comma,
    Dot,
    DotDot,
    LessThanOrEqual,
    GreaterThanOrEqual,
    EqualEqual,
    NotEqual,
    LogicalAnd,
    LogicalOr,
    LeftShift,
    RightShift,
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
    If,
    Else,
    For,
    While,
    Fn,
    Return,
}

use core::str::FromStr;
use core::fmt;


impl FromStr for Operator {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "~" => Ok(Operator::Tilde),
            "=" => Ok(Operator::Equal),
            ":" => Ok(Operator::Colon),
            "+" => Ok(Operator::Plus),
            "-" => Ok(Operator::Minus),
            "*" => Ok(Operator::Star),
            "/" => Ok(Operator::Slash),
            "^" => Ok(Operator::Caret),
            "|" => Ok(Operator::Pipe),
            "&" => Ok(Operator::Ampersand),
            "%" => Ok(Operator::Percent),
            ">" => Ok(Operator::GreaterThan),
            "<" => Ok(Operator::LessThan),
            "!" => Ok(Operator::Exclamation),
            "," => Ok(Operator::Comma),
            "." => Ok(Operator::Dot),
            ".." => Ok(Operator::DotDot),
            "<=" => Ok(Operator::LessThanOrEqual),
            ">=" => Ok(Operator::GreaterThanOrEqual),
            "==" => Ok(Operator::EqualEqual),
            "!=" => Ok(Operator::NotEqual),
            "&&" => Ok(Operator::LogicalAnd),
            "||" => Ok(Operator::LogicalOr),
            "<<" => Ok(Operator::LeftShift),
            ">>" => Ok(Operator::RightShift),
            _ => Err(()),
        }
    }
}

impl FromStr for Punctuation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            " " => Ok(Punctuation::Space),
            "\t" => Ok(Punctuation::Tab),
            "\n" => Ok(Punctuation::Newline),
            "\r" => Ok(Punctuation::CarriageReturn),
            "(" => Ok(Punctuation::LeftParen),
            ")" => Ok(Punctuation::RightParen),
            "[" => Ok(Punctuation::LeftBracket),
            "]" => Ok(Punctuation::RightBracket),
            "{" => Ok(Punctuation::LeftBrace),
            "}" => Ok(Punctuation::RightBrace),
            ";" => Ok(Punctuation::Semicolon),
            _ => Err(()),
        }
    }
}

impl FromStr for Token {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "if" => Ok(Token::Keyword(Keyword::If)),
            "else" => Ok(Token::Keyword(Keyword::Else)),
            "for" => Ok(Token::Keyword(Keyword::For)),
            "while" => Ok(Token::Keyword(Keyword::While)),
            "fn" => Ok(Token::Keyword(Keyword::Fn)),
            "return" => Ok(Token::Keyword(Keyword::Return)),
            _ => Err(()),
        }
    }
}

impl FromStr for Keyword {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "for" => Ok(Keyword::For),
            "while" => Ok(Keyword::While),
            "fn" => Ok(Keyword::Fn),
            "return" => Ok(Keyword::Return),
            _ => Err(()),
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
        };
        write!(f, "{}", op_str)
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keyword_str = match self {
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::For => "for",
            Keyword::While => "while",
            Keyword::Fn => "fn",
            Keyword::Return => "return",
        };
        write!(f, "{}", keyword_str)
    }
}
