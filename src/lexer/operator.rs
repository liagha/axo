use crate::lexer::TokenKind;

#[derive(Clone, Debug, PartialEq)]
pub enum OperatorKind {
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

impl OperatorKind {
    pub fn from_str(s: &str) -> Self {
        match s {
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
            "," => OperatorKind::Comma,
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
            "+=" => OperatorKind::PlusEqual,
            "-=" => OperatorKind::MinusEqual,
            "*=" => OperatorKind::StarEqual,
            "/=" => OperatorKind::SlashEqual,
            "%=" => OperatorKind::PercentEqual,
            "?" => OperatorKind::QuestionMark,
            "..=" => OperatorKind::DotDotEqual,
            "->" => OperatorKind::Arrow,
            "=>" => OperatorKind::FatArrow,
            "@" => OperatorKind::At,
            "#" => OperatorKind::Hash,
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
            "^=" => OperatorKind::CaretEqual,
            "&=" => OperatorKind::AmpersandEqual,
            "|=" => OperatorKind::PipeEqual,
            "&&=" => OperatorKind::LogicalAndEqual,
            "||=" => OperatorKind::LogicalOrEqual,
            "?=" => OperatorKind::QuestionMarkEqual,
            _ => unreachable!(),
        }
    }

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

    pub fn is_compound_token_op(input: Option<&TokenKind>) -> bool {
        if let Some(TokenKind::Operator(operator)) = input {
            operator.is_compound()
        } else {
            false
        }
    }

    pub fn is_compound_token(input: &TokenKind) -> bool {
        if let TokenKind::Operator(operator) = input {
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
            OperatorKind::Star
                | OperatorKind::Slash
                | OperatorKind::Percent
                | OperatorKind::DotDot
                | OperatorKind::LogicalAnd
                | OperatorKind::DoubleStar
        )
    }

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

    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            OperatorKind::Exclamation | OperatorKind::Minus | OperatorKind::Tilde
        )
    }

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
}

impl core::fmt::Display for OperatorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let op_str = match self {
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
            OperatorKind::Comma => ",",
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
            OperatorKind::PlusEqual => "+=",
            OperatorKind::MinusEqual => "-=",
            OperatorKind::StarEqual => "*=",
            OperatorKind::SlashEqual => "/=",
            OperatorKind::PercentEqual => "%=",
            OperatorKind::QuestionMark => "?",
            OperatorKind::DotDotEqual => "..=",
            OperatorKind::Arrow => "->",
            OperatorKind::FatArrow => "=>",
            OperatorKind::At => "@",
            OperatorKind::Hash => "#",
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
            OperatorKind::CaretEqual => "^=",
            OperatorKind::AmpersandEqual => "&=",
            OperatorKind::PipeEqual => "|=",
            OperatorKind::LogicalAndEqual => "&&=",
            OperatorKind::LogicalOrEqual => "||=",
            OperatorKind::QuestionMarkEqual => "?=",
        };
        write!(f, "{}", op_str)
    }
}
