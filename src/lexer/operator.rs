use crate::lexer::{Token, TokenKind, Span};

/// Enum representing various operator kinds in the language
#[derive(Clone, Debug, PartialEq)]
pub enum OperatorKind {
    In,                      // in (used in for-loops and iterators)
    Tilde,                   // ~ (bitwise NOT or unary negation)
    Equal,                   // = (assignment)
    Colon,                   // : (type annotation or key-value separator)
    Plus,                    // + (addition)
    Minus,                   // - (subtraction or negation)
    Star,                    // * (multiplication or pointer/reference)
    Slash,                   // / (division)
    Caret,                   // ^ (bitwise XOR)
    Pipe,                    // | (bitwise OR)
    Ampersand,               // & (bitwise AND or reference)
    Percent,                 // % (modulo)
    GreaterThan,             // > (greater than comparison)
    LessThan,                // < (less than comparison)
    Exclamation,             // ! (logical NOT)
    Dot,                     // . (member access)
    DotDot,                  // .. (range or spread)
    LessThanOrEqual,         // <= (less than or equal comparison)
    GreaterThanOrEqual,      // >= (greater than or equal comparison)
    EqualEqual,              // == (equality comparison)
    NotEqual,                // != (inequality comparison)
    LogicalAnd,              // && (logical AND)
    LogicalOr,               // || (logical OR)
    LeftShift,               // << (left bitwise shift)
    RightShift,              // >> (right bitwise shift)
    ColonEqual,              // := (alternate assignment)
    EqualColon,              // =: (reverse assignment)
    PlusEqual,               // += (addition assignment)
    MinusEqual,              // -= (subtraction assignment)
    StarEqual,               // *= (multiplication assignment)
    SlashEqual,              // /= (division assignment)
    PercentEqual,            // %= (modulo assignment)
    CaretEqual,              // ^= (bitwise XOR assignment)
    AmpersandEqual,          // &= (bitwise AND assignment)
    PipeEqual,               // |= (bitwise OR assignment)
    LogicalAndEqual,         // &&= (logical AND assignment)
    LogicalOrEqual,          // ||= (logical OR assignment)
    QuestionMarkEqual,       // ?= (optional assignment)
    DotDotEqual,             // ..= (inclusive range)
    Arrow,                   // -> (function return type or closure)
    FatArrow,                // => (match arm or closure)
    At,                      // @ (annotation or pattern binding)
    Hash,                    // # (attribute or preprocessor)
    QuestionMark,            // ? (optional or error handling)
    Dollar,                  // $ (template literal or macro)
    Backslash,               // \ (escape character)
    DoubleQuote,             // " (string delimiter)
    SingleQuote,             // ' (character literal delimiter)
    Backtick,                // ` (raw string or template literal)
    Underscore,              // _ (wildcard or ignored value)
    DoubleColon,             // :: (path separator or associated function)
    DoubleQuestionMark,      // ?? (null coalescing)
    DoubleExclamation,       // !! (double negation)
    DoubleStar,              // ** (exponentiation)
    DoubleSlash,             // // (integer division or comment)
    SlashStar,               // /* (block comment start)
    StarSlash,               // */ (block comment end)
    DoublePercent,           // %% (custom modulo)
    DoubleCaret,             // ^^ (custom exponentiation)
    DoubleTilde,             // ~~ (extended bitwise operation)
    DoubleAt,                // @@ (extended annotation)
    DoubleHash,              // ## (token concatenation)
    DoubleDollar,            // $$ (macro expansion)
    DoubleBackslash,         // \\ (escaped backslash)
    DoubleUnderscore,        // __ (special identifier)
    TripleEqual,             // === (strict equality)
    TripleSlash,             // /// (another comment type)
    SlashDoubleStar,         // /**
    DoubleSlashExclamation,  // //!
    NotTripleEqual,          // !== (strict inequality)
    DoubleStarEqual,         // **= (exponentiation assignment)
    ModuloExponentiation,    // %%= (custom modulo assignment)
    Unknown,                 // for when the operator is not detected
}

impl OperatorKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "in" => OperatorKind::In,
            "~" => OperatorKind::Tilde,
            "=" => OperatorKind::Equal,
            ":" => OperatorKind::Colon,
            "+" => OperatorKind::Plus,
            "-" => OperatorKind::Minus,
            "*" => OperatorKind::Star,
            "/" => OperatorKind::Slash,
            "^" => OperatorKind::Caret,
            "|" => OperatorKind::Pipe,
            "&" => OperatorKind::Ampersand,
            "%" => OperatorKind::Percent,
            ">" => OperatorKind::GreaterThan,
            "<" => OperatorKind::LessThan,
            "!" => OperatorKind::Exclamation,
            "." => OperatorKind::Dot,
            ".." => OperatorKind::DotDot,
            "<=" => OperatorKind::LessThanOrEqual,
            ">=" => OperatorKind::GreaterThanOrEqual,
            "==" => OperatorKind::EqualEqual,
            "!=" => OperatorKind::NotEqual,
            "&&" => OperatorKind::LogicalAnd,
            "||" => OperatorKind::LogicalOr,
            "<<" => OperatorKind::LeftShift,
            ">>" => OperatorKind::RightShift,
            ":=" => OperatorKind::ColonEqual,
            "=:" => OperatorKind::EqualColon,
            "+=" => OperatorKind::PlusEqual,
            "-=" => OperatorKind::MinusEqual,
            "*=" => OperatorKind::StarEqual,
            "/=" => OperatorKind::SlashEqual,
            "%=" => OperatorKind::PercentEqual,
            "^=" => OperatorKind::CaretEqual,
            "&=" => OperatorKind::AmpersandEqual,
            "|=" => OperatorKind::PipeEqual,
            "&&=" => OperatorKind::LogicalAndEqual,
            "||=" => OperatorKind::LogicalOrEqual,
            "?=" => OperatorKind::QuestionMarkEqual,
            "..=" => OperatorKind::DotDotEqual,
            "->" => OperatorKind::Arrow,
            "=>" => OperatorKind::FatArrow,
            "@" => OperatorKind::At,
            "#" => OperatorKind::Hash,
            "?" => OperatorKind::QuestionMark,
            "$" => OperatorKind::Dollar,
            "\\" => OperatorKind::Backslash,
            "\"" => OperatorKind::DoubleQuote,
            "'" => OperatorKind::SingleQuote,
            "`" => OperatorKind::Backtick,
            "_" => OperatorKind::Underscore,
            "::" => OperatorKind::DoubleColon,
            "??" => OperatorKind::DoubleQuestionMark,
            "!!" => OperatorKind::DoubleExclamation,
            "**" => OperatorKind::DoubleStar,
            "//" => OperatorKind::DoubleSlash,
            "/*" => OperatorKind::SlashStar,
            "*/" => OperatorKind::StarSlash,
            "%%" => OperatorKind::DoublePercent,
            "^^" => OperatorKind::DoubleCaret,
            "~~" => OperatorKind::DoubleTilde,
            "@@" => OperatorKind::DoubleAt,
            "##" => OperatorKind::DoubleHash,
            "$$" => OperatorKind::DoubleDollar,
            "\\\\" => OperatorKind::DoubleBackslash,
            "__" => OperatorKind::DoubleUnderscore,
            "===" => OperatorKind::TripleEqual,
            "///" => OperatorKind::TripleSlash,
            "!==" => OperatorKind::NotTripleEqual,
            "**=" => OperatorKind::DoubleStarEqual,
            "//!" => OperatorKind::DoubleSlashExclamation,
            "%%=" => OperatorKind::ModuloExponentiation,
            "" => OperatorKind::SlashDoubleStar,
            _ => OperatorKind::Unknown,
        }
    }
}

impl OperatorKind {
    /// Returns true if the operator is a compound assignment operator
    pub fn is_compound(&self) -> bool {
        matches!(
            self,
            OperatorKind::LogicalAndEqual
                | OperatorKind::LogicalOrEqual
                | OperatorKind::QuestionMarkEqual
                | OperatorKind::DotDotEqual
                | OperatorKind::AmpersandEqual
                | OperatorKind::PipeEqual
                | OperatorKind::StarEqual
                | OperatorKind::SlashEqual
                | OperatorKind::PercentEqual
                | OperatorKind::CaretEqual
                | OperatorKind::PlusEqual
                | OperatorKind::MinusEqual
        )
    }

    /// Checks if a token is a compound operator
    pub fn is_compound_token_op(input: Option<&TokenKind>) -> bool {
        matches!(input, Some(TokenKind::Operator(operator)) if operator.is_compound())
    }

    /// Checks if a specific token is a compound operator
    pub fn is_compound_token(input: &TokenKind) -> bool {
        matches!(input, TokenKind::Operator(operator) if operator.is_compound())
    }

    /// Checks if a character is a potential operator
    pub fn is_operator(char: char) -> bool {
        matches!(
            char,
            '~' | '=' | ':' | '+' | '-' |
            '*' | '/' | '^' | '|' | '&' |
            '%' | '>' | '<' | '!' | '.' |
            '@' | '\'' | '?' | '#' | '$' |
            '\\' | '`' | '_'
        )
    }

    /// Checks if the operator is a factor operator
    pub fn is_factor(&self) -> bool {
        matches!(
            self,
            OperatorKind::Star
                | OperatorKind::Slash
                | OperatorKind::Percent
                | OperatorKind::DotDot
                | OperatorKind::LogicalAnd
                | OperatorKind::DoubleStar
                | OperatorKind::In
        )
    }

    /// Checks if the operator is a term operator
    pub fn is_term(&self) -> bool {
        matches!(
            self,
            OperatorKind::Plus
                | OperatorKind::Minus
                | OperatorKind::LogicalOr
                | OperatorKind::Pipe
                | OperatorKind::Caret
        )
    }

    /// Checks if the operator is an expression operator
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            OperatorKind::EqualEqual
                | OperatorKind::NotEqual
                | OperatorKind::GreaterThan
                | OperatorKind::LessThan
                | OperatorKind::GreaterThanOrEqual
                | OperatorKind::LessThanOrEqual
        )
    }

    /// Checks if the operator is a unary operator
    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            OperatorKind::Exclamation
            | OperatorKind::Plus
            | OperatorKind::Minus
            | OperatorKind::Tilde
            | OperatorKind::Ampersand
        )
    }

    /// Decompounds a compound assignment operator to its base operator
    pub fn decompound(&self) -> OperatorKind {
        match self {
            OperatorKind::LogicalAndEqual => OperatorKind::LogicalAnd,
            OperatorKind::LogicalOrEqual => OperatorKind::LogicalOr,
            OperatorKind::QuestionMarkEqual => OperatorKind::QuestionMark,
            OperatorKind::DotDotEqual => OperatorKind::DotDot,
            OperatorKind::AmpersandEqual => OperatorKind::Ampersand,
            OperatorKind::PipeEqual => OperatorKind::Pipe,
            OperatorKind::StarEqual => OperatorKind::Star,
            OperatorKind::SlashEqual => OperatorKind::Slash,
            OperatorKind::PercentEqual => OperatorKind::Percent,
            OperatorKind::CaretEqual => OperatorKind::Caret,
            OperatorKind::PlusEqual => OperatorKind::Plus,
            OperatorKind::MinusEqual => OperatorKind::Minus,
            _ => unreachable!(),
        }
    }

    pub fn decompound_token(token: &Token) -> Token {
        let Span { start: (sl, sc), end: (el, ec), file_name, file_path } = token.span.clone();

        let new_span = Span { start: (sl, sc), end: (el, ec - 1), file_name, file_path };

        let (operator, span) = if let TokenKind::Operator(op) = &token.kind { 
            match op {
                OperatorKind::LogicalAndEqual => (OperatorKind::LogicalAnd, new_span),
                OperatorKind::LogicalOrEqual => (OperatorKind::LogicalOr, new_span),
                OperatorKind::QuestionMarkEqual => (OperatorKind::QuestionMark, new_span),
                OperatorKind::DotDotEqual => (OperatorKind::DotDot, new_span),
                OperatorKind::AmpersandEqual => (OperatorKind::Ampersand, new_span),
                OperatorKind::PipeEqual => (OperatorKind::Pipe, new_span),
                OperatorKind::StarEqual => (OperatorKind::Star, new_span),
                OperatorKind::SlashEqual => (OperatorKind::Slash, new_span),
                OperatorKind::PercentEqual => (OperatorKind::Percent, new_span),
                OperatorKind::CaretEqual => (OperatorKind::Caret, new_span),
                OperatorKind::PlusEqual => (OperatorKind::Plus, new_span),
                OperatorKind::MinusEqual => (OperatorKind::Minus, new_span),
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        };

        Token { kind: TokenKind::Operator(operator), span }
    }
}

impl core::fmt::Display for OperatorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let op_str = match self {
            OperatorKind::In => "in",
            OperatorKind::Tilde => "~",
            OperatorKind::Equal => "=",
            OperatorKind::Colon => ":",
            OperatorKind::Plus => "+",
            OperatorKind::Minus => "-",
            OperatorKind::Star => "*",
            OperatorKind::Slash => "/",
            OperatorKind::Caret => "^",
            OperatorKind::Pipe => "|",
            OperatorKind::Ampersand => "&",
            OperatorKind::Percent => "%",
            OperatorKind::GreaterThan => ">",
            OperatorKind::LessThan => "<",
            OperatorKind::Exclamation => "!",
            OperatorKind::Dot => ".",
            OperatorKind::DotDot => "..",
            OperatorKind::LessThanOrEqual => "<=",
            OperatorKind::GreaterThanOrEqual => ">=",
            OperatorKind::EqualEqual => "==",
            OperatorKind::NotEqual => "!=",
            OperatorKind::LogicalAnd => "&&",
            OperatorKind::LogicalOr => "||",
            OperatorKind::LeftShift => "<<",
            OperatorKind::RightShift => ">>",
            OperatorKind::ColonEqual => ":=",
            OperatorKind::EqualColon => "=:",
            OperatorKind::PlusEqual => "+=",
            OperatorKind::MinusEqual => "-=",
            OperatorKind::StarEqual => "*=",
            OperatorKind::SlashEqual => "/=",
            OperatorKind::PercentEqual => "%=",
            OperatorKind::CaretEqual => "^=",
            OperatorKind::AmpersandEqual => "&=",
            OperatorKind::PipeEqual => "|=",
            OperatorKind::LogicalAndEqual => "&&=",
            OperatorKind::LogicalOrEqual => "||=",
            OperatorKind::QuestionMarkEqual => "?=",
            OperatorKind::DotDotEqual => "..=",
            OperatorKind::Arrow => "->",
            OperatorKind::FatArrow => "=>",
            OperatorKind::At => "@",
            OperatorKind::Hash => "#",
            OperatorKind::QuestionMark => "?",
            OperatorKind::Dollar => "$",
            OperatorKind::Backslash => "\\",
            OperatorKind::DoubleQuote => "\"",
            OperatorKind::SingleQuote => "'",
            OperatorKind::Backtick => "`",
            OperatorKind::Underscore => "_",
            OperatorKind::DoubleColon => "::",
            OperatorKind::DoubleQuestionMark => "??",
            OperatorKind::DoubleExclamation => "!!",
            OperatorKind::DoubleStar => "**",
            OperatorKind::DoubleSlash => "//",
            OperatorKind::SlashStar => "/*",
            OperatorKind::StarSlash => "*/",
            OperatorKind::DoublePercent => "%%",
            OperatorKind::DoubleCaret => "^^",
            OperatorKind::DoubleTilde => "~~",
            OperatorKind::DoubleAt => "@@",
            OperatorKind::DoubleHash => "##",
            OperatorKind::DoubleDollar => "$$",
            OperatorKind::DoubleBackslash => "\\\\",
            OperatorKind::DoubleUnderscore => "__",
            OperatorKind::TripleEqual => "===",
            OperatorKind::TripleSlash => "///",
            OperatorKind::NotTripleEqual => "!==",
            OperatorKind::DoubleStarEqual => "**=",
            OperatorKind::ModuloExponentiation => "%%=",
            OperatorKind::SlashDoubleStar => "/**",
            OperatorKind::DoubleSlashExclamation => "//!",
            OperatorKind::Unknown => "????",
        };
        write!(f, "{}", op_str)
    }
}
