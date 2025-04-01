use crate::lexer::{Token, TokenKind, Span};

/// Enum representing various operator kinds in the language
#[derive(Clone, Debug, PartialEq)]
pub enum OperatorKind {
    // Single character operators (sorted alphabetically)
    At,                      // @ (annotation or pattern binding)
    Ampersand,               // & (bitwise AND or reference)
    Backslash,               // \ (escape character)
    Caret,                   // ^ (bitwise XOR)
    Colon,                   // : (type annotation or key-value separator)
    Dollar,                  // $ (template literal or macro)
    Dot,                     // . (member access)
    DoubleQuote,             // " (string delimiter)
    Equal,                   // = (assignment)
    Exclamation,             // ! (logical NOT)
    GreaterThan,             // > (greater than comparison)
    Hash,                    // # (attribute or preprocessor)
    LessThan,                // < (less than comparison)
    Minus,                   // - (subtraction or negation)
    Percent,                 // % (modulo)
    Pipe,                    // | (bitwise OR)
    Plus,                    // + (addition)
    QuestionMark,            // ? (optional or error handling)
    SingleQuote,             // ' (character literal delimiter)
    Slash,                   // / (division)
    Star,                    // * (multiplication or pointer/reference)
    Tilde,                   // ~ (bitwise NOT or unary negation)
    Underscore,              // _ (wildcard or ignored value)
    Backtick,                // ` (raw string or template literal)

    // Word operators
    In,                      // in (used in for-loops and iterators)

    // Double character operators (sorted by category and function)
    // Increments and decrements
    PlusPlus,                // ++ (increment)
    MinusMinus,              // -- (decrement)

    // Basic arithmetic and logic double operators
    StarStar,                // ** (exponentiation)
    SlashSlash,              // // (integer division or comment)
    PercentPercent,          // %% (custom modulo)
    TildeTilde,              // ~~ (extended bitwise operation)
    CaretCaret,              // ^^ (custom exponentiation)

    // Range operators
    DotDot,                  // .. (range or spread)
    DotDotEqual,             // ..= (inclusive range)
    DotDotDot,               // ... (for showing unlimited sequence)

    // Comparison operators
    EqualEqual,              // == (equality comparison)
    ExclamationEqual,        // != (inequality comparison)
    LessThanEqual,           // <= (less than or equal comparison)
    GreaterThanEqual,        // >= (greater than or equal comparison)
    EqualEqualEqual,         // === (strict equality)
    ExclamationEqualEqual,   // !== (strict inequality)

    // Logical operators
    AmpersandAmpersand,      // && (logical AND)
    PipePipe,                // || (logical OR)
    QuestionMarkQuestionMark, // ?? (null coalescing)
    ExclamationExclamation,  // !! (double negation)

    // Bitwise shift operators
    LessThanLessThan,        // << (left bitwise shift)
    GreaterThanGreaterThan,  // >> (right bitwise shift)

    // Assignment operators
    ColonEqual,              // := (alternate assignment)
    EqualColon,              // =: (reverse assignment)

    // Compound assignment operators
    PlusEqual,               // += (addition assignment)
    MinusEqual,              // -= (subtraction assignment)
    StarEqual,               // *= (multiplication assignment)
    SlashEqual,              // /= (division assignment)
    PercentEqual,            // %= (modulo assignment)
    CaretEqual,              // ^= (bitwise XOR assignment)
    AmpersandEqual,          // &= (bitwise AND assignment)
    PipeEqual,               // |= (bitwise OR assignment)
    StarStarEqual,           // **= (exponentiation assignment)
    PercentPercentEqual,     // %%= (custom modulo assignment)
    AmpersandAmpersandEqual, // &&= (logical AND assignment)
    PipePipeEqual,           // ||= (logical OR assignment)
    QuestionMarkEqual,       // ?= (optional assignment)

    // Arrow operators
    MinusGreaterThan,        // -> (function return type or closure)
    EqualGreaterThan,        // => (match arm or closure)

    // Special path/namespace operators
    ColonColon,              // :: (path separator or associated function)

    // Comment operators
    SlashStar,               // /* (block comment start)
    StarSlash,               // */ (block comment end)
    SlashSlashSlash,         // /// (documentation comment)
    SlashStarStar,           // /** (doc comment start)
    SlashSlashExclamation,   // //! (module doc comment)

    // Special double character operators
    AtAt,                    // @@ (extended annotation)
    HashHash,                // ## (token concatenation)
    DollarDollar,            // $$ (macro expansion)
    BackslashBackslash,      // \\ (escaped backslash)
    UnderscoreUnderscore,    // __ (special identifier)

    // Unknown operator
    Unknown,                 // for when the operator is not detected
}

impl OperatorKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            // Single character operators
            "@" => OperatorKind::At,
            "&" => OperatorKind::Ampersand,
            "\\" => OperatorKind::Backslash,
            "^" => OperatorKind::Caret,
            ":" => OperatorKind::Colon,
            "$" => OperatorKind::Dollar,
            "." => OperatorKind::Dot,
            "\"" => OperatorKind::DoubleQuote,
            "=" => OperatorKind::Equal,
            "!" => OperatorKind::Exclamation,
            ">" => OperatorKind::GreaterThan,
            "#" => OperatorKind::Hash,
            "<" => OperatorKind::LessThan,
            "-" => OperatorKind::Minus,
            "%" => OperatorKind::Percent,
            "|" => OperatorKind::Pipe,
            "+" => OperatorKind::Plus,
            "?" => OperatorKind::QuestionMark,
            "'" => OperatorKind::SingleQuote,
            "/" => OperatorKind::Slash,
            "*" => OperatorKind::Star,
            "~" => OperatorKind::Tilde,
            "_" => OperatorKind::Underscore,
            "`" => OperatorKind::Backtick,

            // Word operators
            "in" => OperatorKind::In,

            // Double character operators
            "++" => OperatorKind::PlusPlus,
            "--" => OperatorKind::MinusMinus,
            "**" => OperatorKind::StarStar,
            "//" => OperatorKind::SlashSlash,
            "%%" => OperatorKind::PercentPercent,
            "~~" => OperatorKind::TildeTilde,
            "^^" => OperatorKind::CaretCaret,
            ".." => OperatorKind::DotDot,
            "..=" => OperatorKind::DotDotEqual,
            "..." => OperatorKind::DotDotDot,
            "==" => OperatorKind::EqualEqual,
            "!=" => OperatorKind::ExclamationEqual,
            "<=" => OperatorKind::LessThanEqual,
            ">=" => OperatorKind::GreaterThanEqual,
            "===" => OperatorKind::EqualEqualEqual,
            "!==" => OperatorKind::ExclamationEqualEqual,
            "&&" => OperatorKind::AmpersandAmpersand,
            "||" => OperatorKind::PipePipe,
            "??" => OperatorKind::QuestionMarkQuestionMark,
            "!!" => OperatorKind::ExclamationExclamation,
            "<<" => OperatorKind::LessThanLessThan,
            ">>" => OperatorKind::GreaterThanGreaterThan,
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
            "**=" => OperatorKind::StarStarEqual,
            "%%=" => OperatorKind::PercentPercentEqual,
            "&&=" => OperatorKind::AmpersandAmpersandEqual,
            "||=" => OperatorKind::PipePipeEqual,
            "?=" => OperatorKind::QuestionMarkEqual,
            "->" => OperatorKind::MinusGreaterThan,
            "=>" => OperatorKind::EqualGreaterThan,
            "::" => OperatorKind::ColonColon,
            "/*" => OperatorKind::SlashStar,
            "*/" => OperatorKind::StarSlash,
            "///" => OperatorKind::SlashSlashSlash,
            "/**" => OperatorKind::SlashStarStar,
            "//!" => OperatorKind::SlashSlashExclamation,
            "@@" => OperatorKind::AtAt,
            "##" => OperatorKind::HashHash,
            "$$" => OperatorKind::DollarDollar,
            "\\\\" => OperatorKind::BackslashBackslash,
            "__" => OperatorKind::UnderscoreUnderscore,
            _ => OperatorKind::Unknown,
        }
    }
}

impl OperatorKind {
    /// Returns true if the operator is a compound assignment operator
    pub fn is_compound(&self) -> bool {
        matches!(
            self,
            OperatorKind::AmpersandAmpersandEqual
                | OperatorKind::PipePipeEqual
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
                | OperatorKind::StarStarEqual
                | OperatorKind::PercentPercentEqual
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
                | OperatorKind::AmpersandAmpersand
                | OperatorKind::StarStar
                | OperatorKind::In
        )
    }

    /// Checks if the operator is a term operator
    pub fn is_term(&self) -> bool {
        matches!(
            self,
            OperatorKind::Plus
                | OperatorKind::Minus
                | OperatorKind::PipePipe
                | OperatorKind::Pipe
                | OperatorKind::Caret
        )
    }

    /// Checks if the operator is an expression operator
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            OperatorKind::EqualEqual
                | OperatorKind::ExclamationEqual
                | OperatorKind::GreaterThan
                | OperatorKind::LessThan
                | OperatorKind::GreaterThanEqual
                | OperatorKind::LessThanEqual
        )
    }

    /// Checks if the operator is a prefix operator
    pub fn is_prefix(&self) -> bool {
        matches!(
            self,
            OperatorKind::Exclamation
            | OperatorKind::Plus
            | OperatorKind::Minus
            | OperatorKind::Tilde
            | OperatorKind::Ampersand
            | OperatorKind::PlusPlus
            | OperatorKind::MinusMinus
        )
    }

    /// Checks if the operator is a postfix operator
    pub fn is_postfix(&self) -> bool {
        matches!(
            self,
            OperatorKind::DotDotDot
            | OperatorKind::PlusPlus
            | OperatorKind::MinusMinus
        )
    }

    /// Decompounds a compound assignment operator to its base operator
    pub fn decompound(&self) -> OperatorKind {
        match self {
            OperatorKind::AmpersandAmpersandEqual => OperatorKind::AmpersandAmpersand,
            OperatorKind::PipePipeEqual => OperatorKind::PipePipe,
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
            OperatorKind::StarStarEqual => OperatorKind::StarStar,
            OperatorKind::PercentPercentEqual => OperatorKind::PercentPercent,
            _ => unreachable!(),
        }
    }

    pub fn decompound_token(token: &Token) -> Token {
        let Span { start: (sl, sc), end: (el, ec), file } = token.span.clone();

        let new_span = Span { start: (sl, sc), end: (el, ec - 1), file };

        let (operator, span) = if let TokenKind::Operator(op) = &token.kind {
            match op {
                OperatorKind::AmpersandAmpersandEqual => (OperatorKind::AmpersandAmpersand, new_span),
                OperatorKind::PipePipeEqual => (OperatorKind::PipePipe, new_span),
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
                OperatorKind::StarStarEqual => (OperatorKind::StarStar, new_span),
                OperatorKind::PercentPercentEqual => (OperatorKind::PercentPercent, new_span),
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
            OperatorKind::At => "@",
            OperatorKind::Hash => "#",
            OperatorKind::QuestionMark => "?",
            OperatorKind::Dollar => "$",
            OperatorKind::Backslash => "\\",
            OperatorKind::DoubleQuote => "\"",
            OperatorKind::SingleQuote => "'",
            OperatorKind::Backtick => "`",
            OperatorKind::Underscore => "_",
            OperatorKind::PlusPlus => "++",
            OperatorKind::MinusMinus => "--",
            OperatorKind::StarStar => "**",
            OperatorKind::SlashSlash => "//",
            OperatorKind::PercentPercent => "%%",
            OperatorKind::TildeTilde => "~~",
            OperatorKind::CaretCaret => "^^",
            OperatorKind::DotDot => "..",
            OperatorKind::DotDotEqual => "..=",
            OperatorKind::DotDotDot => "...",
            OperatorKind::EqualEqual => "==",
            OperatorKind::ExclamationEqual => "!=",
            OperatorKind::LessThanEqual => "<=",
            OperatorKind::GreaterThanEqual => ">=",
            OperatorKind::EqualEqualEqual => "===",
            OperatorKind::ExclamationEqualEqual => "!==",
            OperatorKind::AmpersandAmpersand => "&&",
            OperatorKind::PipePipe => "||",
            OperatorKind::QuestionMarkQuestionMark => "??",
            OperatorKind::ExclamationExclamation => "!!",
            OperatorKind::LessThanLessThan => "<<",
            OperatorKind::GreaterThanGreaterThan => ">>",
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
            OperatorKind::StarStarEqual => "**=",
            OperatorKind::PercentPercentEqual => "%%=",
            OperatorKind::AmpersandAmpersandEqual => "&&=",
            OperatorKind::PipePipeEqual => "||=",
            OperatorKind::QuestionMarkEqual => "?=",
            OperatorKind::MinusGreaterThan => "->",
            OperatorKind::EqualGreaterThan => "=>",
            OperatorKind::ColonColon => "::",
            OperatorKind::SlashStar => "/*",
            OperatorKind::StarSlash => "*/",
            OperatorKind::SlashSlashSlash => "///",
            OperatorKind::SlashStarStar => "/**",
            OperatorKind::SlashSlashExclamation => "//!",
            OperatorKind::AtAt => "@@",
            OperatorKind::HashHash => "##",
            OperatorKind::DollarDollar => "$$",
            OperatorKind::BackslashBackslash => "\\\\",
            OperatorKind::UnderscoreUnderscore => "__",
            OperatorKind::Unknown => "????",
        };
        write!(f, "{}", op_str)
    }
}