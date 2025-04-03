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
    DoublePlus,              // ++ (increment)
    DoubleMinus,             // -- (decrement)

    // Basic arithmetic and logic double operators
    DoubleStar,              // ** (exponentiation)
    DoubleSlash,             // // (integer division or comment)
    DoublePercent,           // %% (custom modulo)
    DoubleTilde,             // ~~ (extended bitwise operation)
    DoubleCaret,             // ^^ (custom exponentiation)

    // Range operators
    DoubleDot,               // .. (range or spread)
    RangeInclusive,          // ..= (inclusive range)
    TripleDot,               // ... (for showing unlimited sequence)

    // Comparison operators
    DoubleEqual,             // == (equality comparison)
    NotEqual,                // != (inequality comparison)
    LessThanOrEqual,         // <= (less than or equal comparison)
    GreaterThanOrEqual,      // >= (greater than or equal comparison)
    TripleEqual,             // === (strict equality)
    StrictNotEqual,          // !== (strict inequality)

    // Logical operators
    DoubleAmpersand,         // && (logical AND)
    DoublePipe,              // || (logical OR)
    DoubleQuestionMark,      // ?? (null coalescing)
    DoubleExclamation,       // !! (double negation)

    // Bitwise shift operators
    LeftShift,               // << (left bitwise shift)
    RightShift,              // >> (right bitwise shift)

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
    DoubleStarEqual,         // **= (exponentiation assignment)
    DoublePercentEqual,      // %%= (custom modulo assignment)
    LogicalAndEqual,         // &&= (logical AND assignment)
    LogicalOrEqual,          // ||= (logical OR assignment)
    QuestionMarkEqual,       // ?= (optional assignment)

    // Arrow operators
    Arrow,                   // -> (function return type or closure)
    LeftArrow,               // <- (left direction arrow)
    LeftFatArrow,            // <= (left fat arrow)
    PipeRight,               // |> (pipe forward operator)
    PipeLeft,                // <| (pipe backward operator)
    AngleRight,              // ~> (tilde forward arrow)
    AngleLeft,               // <~ (tilde backward arrow)
    DoubleArrow,             // --> (double right arrow)
    DoubleLeftArrow,         // <-- (double left arrow)
    DoubleFatArrow,          // ==> (double fat right arrow)
    DoubleFatLeftArrow,      // <== (double fat left arrow)

    // Special path/namespace operators
    DoubleColon,             // :: (path separator or associated function)

    // Comment operators
    SlashStart,              // /* (block comment start)
    StarSlash,               // */ (block comment end)
    TripleSlash,             // /// (documentation comment)
    SlashDoubleStart,        // /** (doc comment start)
    DoubleSlashExclamation,  // //! (module doc comment)

    // Special double character operators
    DoubleAt,                // @@ (extended annotation)
    DoubleHash,              // ## (token concatenation)
    DoubleDollar,            // $$ (macro expansion)
    DoubleBackslash,         // \\ (escaped backslash)
    DoubleUnderscore,        // __ (special identifier)

    // Unknown operator
    Unknown,                 // for when the operator is not detected
}



impl OperatorKind {
    pub fn is_compound(&self) -> bool {
        matches!(
            self,
            OperatorKind::LogicalAndEqual
                | OperatorKind::LogicalOrEqual
                | OperatorKind::QuestionMarkEqual
                | OperatorKind::RangeInclusive
                | OperatorKind::AmpersandEqual
                | OperatorKind::PipeEqual
                | OperatorKind::StarEqual
                | OperatorKind::SlashEqual
                | OperatorKind::PercentEqual
                | OperatorKind::CaretEqual
                | OperatorKind::PlusEqual
                | OperatorKind::MinusEqual
                | OperatorKind::DoubleStarEqual
                | OperatorKind::DoublePercentEqual
        )
    }

    pub fn is_factor(&self) -> bool {
        matches!(
            self,
            OperatorKind::Star
                | OperatorKind::Slash
                | OperatorKind::Percent
                | OperatorKind::DoubleDot
                | OperatorKind::DoubleAmpersand
                | OperatorKind::DoubleStar
                | OperatorKind::In
        )
    }

    pub fn is_term(&self) -> bool {
        self.is_arrow() || self.is_left_arrow() ||
        matches!(
            self,
            OperatorKind::Plus
                | OperatorKind::Minus
                | OperatorKind::DoublePipe
                | OperatorKind::Pipe
                | OperatorKind::Caret

                | OperatorKind::Colon
                | OperatorKind::Dot
                | OperatorKind::DoubleColon

        )
    }

    pub fn is_expression(&self) -> bool {
        self.is_compound() ||
            matches!(
            self,
            OperatorKind::Equal
                | OperatorKind::ColonEqual
                | OperatorKind::DoubleEqual
                | OperatorKind::NotEqual
                | OperatorKind::GreaterThan
                | OperatorKind::LessThan
                | OperatorKind::GreaterThanOrEqual
                | OperatorKind::LessThanOrEqual
        )
    }

    pub fn is_arrow(&self) -> bool {
        matches!(
            self,
            OperatorKind::Arrow
                | OperatorKind::PipeRight
                | OperatorKind::AngleRight
                | OperatorKind::DoubleArrow
                | OperatorKind::DoubleFatArrow
        )
    }

    pub fn is_left_arrow(&self) -> bool {
        matches!(
            self,
            OperatorKind::DoubleLeftArrow
                | OperatorKind::AngleLeft
                | OperatorKind::PipeLeft
                | OperatorKind::LeftArrow
                | OperatorKind::LeftFatArrow
                | OperatorKind::DoubleFatLeftArrow
        )
    }

    pub fn is_prefix(&self) -> bool {
        matches!(
            self,
            OperatorKind::Exclamation
            | OperatorKind::Plus
            | OperatorKind::Minus
            | OperatorKind::Tilde
            | OperatorKind::Ampersand
            | OperatorKind::DoublePlus
            | OperatorKind::DoubleMinus
        )
    }

    pub fn is_postfix(&self) -> bool {
        matches!(
            self,
            OperatorKind::TripleDot
            | OperatorKind::DoublePlus
            | OperatorKind::DoubleMinus
        )
    }

    pub fn decompound(&self) -> OperatorKind {
        match self {
            OperatorKind::LogicalAndEqual => OperatorKind::DoubleAmpersand,
            OperatorKind::LogicalOrEqual => OperatorKind::DoublePipe,
            OperatorKind::QuestionMarkEqual => OperatorKind::QuestionMark,
            OperatorKind::RangeInclusive => OperatorKind::DoubleDot,
            OperatorKind::AmpersandEqual => OperatorKind::Ampersand,
            OperatorKind::PipeEqual => OperatorKind::Pipe,
            OperatorKind::StarEqual => OperatorKind::Star,
            OperatorKind::SlashEqual => OperatorKind::Slash,
            OperatorKind::PercentEqual => OperatorKind::Percent,
            OperatorKind::CaretEqual => OperatorKind::Caret,
            OperatorKind::PlusEqual => OperatorKind::Plus,
            OperatorKind::MinusEqual => OperatorKind::Minus,
            OperatorKind::DoubleStarEqual => OperatorKind::DoubleStar,
            OperatorKind::DoublePercentEqual => OperatorKind::DoublePercent,
            _ => OperatorKind::Unknown,
        }
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
            OperatorKind::DoublePlus => "++",
            OperatorKind::DoubleMinus => "--",
            OperatorKind::DoubleStar => "**",
            OperatorKind::DoubleSlash => "//",
            OperatorKind::DoublePercent => "%%",
            OperatorKind::DoubleTilde => "~~",
            OperatorKind::DoubleCaret => "^^",
            OperatorKind::DoubleDot => "..",
            OperatorKind::RangeInclusive => "..=",
            OperatorKind::TripleDot => "...",
            OperatorKind::DoubleEqual => "==",
            OperatorKind::NotEqual => "!=",
            OperatorKind::LessThanOrEqual => "<=",
            OperatorKind::GreaterThanOrEqual => ">=",
            OperatorKind::TripleEqual => "===",
            OperatorKind::StrictNotEqual => "!==",
            OperatorKind::DoubleAmpersand => "&&",
            OperatorKind::DoublePipe => "||",
            OperatorKind::DoubleQuestionMark => "??",
            OperatorKind::DoubleExclamation => "!!",
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
            OperatorKind::DoubleStarEqual => "**=",
            OperatorKind::DoublePercentEqual => "%%=",
            OperatorKind::LogicalAndEqual => "&&=",
            OperatorKind::LogicalOrEqual => "||=",
            OperatorKind::QuestionMarkEqual => "?=",
            OperatorKind::Arrow => "->",
            OperatorKind::LeftArrow => "<-",
            OperatorKind::LeftFatArrow => "<=",
            OperatorKind::PipeRight => "|>",
            OperatorKind::PipeLeft => "<|",
            OperatorKind::AngleRight => "~>",
            OperatorKind::AngleLeft => "<~",
            OperatorKind::DoubleArrow => "-->",
            OperatorKind::DoubleLeftArrow => "<--",
            OperatorKind::DoubleFatArrow => "==>",
            OperatorKind::DoubleFatLeftArrow => "<==",
            OperatorKind::DoubleColon => "::",
            OperatorKind::SlashStart => "/*",
            OperatorKind::StarSlash => "*/",
            OperatorKind::TripleSlash => "///",
            OperatorKind::SlashDoubleStart => "/**",
            OperatorKind::DoubleSlashExclamation => "//!",
            OperatorKind::DoubleAt => "@@",
            OperatorKind::DoubleHash => "##",
            OperatorKind::DoubleDollar => "$$",
            OperatorKind::DoubleBackslash => "\\\\",
            OperatorKind::DoubleUnderscore => "__",
            OperatorKind::Unknown => "????",
        };
        write!(f, "{}", op_str)
    }
}

pub trait OperatorLexer {
    fn is_operator(&self) -> bool;
    fn to_operator(&self) -> OperatorKind;
}

impl OperatorLexer for char {
    fn is_operator(&self) -> bool {
        matches!(
            self,
            '~' | '=' | ':' | '+' | '-' |
            '*' | '/' | '^' | '|' | '&' |
            '%' | '>' | '<' | '!' | '.' |
            '@' | '\'' | '?' | '#' | '$' |
            '\\' | '`' | '_'
        )
    }

    fn to_operator(&self) -> OperatorKind {
        match self {
            '@' => OperatorKind::At,
            '&' => OperatorKind::Ampersand,
            '\\' => OperatorKind::Backslash,
            '^' => OperatorKind::Caret,
            ':' => OperatorKind::Colon,
            '$' => OperatorKind::Dollar,
            '.' => OperatorKind::Dot,
            '"' => OperatorKind::DoubleQuote,
            '=' => OperatorKind::Equal,
            '!' => OperatorKind::Exclamation,
            '>' => OperatorKind::GreaterThan,
            '#' => OperatorKind::Hash,
            '<' => OperatorKind::LessThan,
            '-' => OperatorKind::Minus,
            '%' => OperatorKind::Percent,
            '|' => OperatorKind::Pipe,
            '+' => OperatorKind::Plus,
            '?' => OperatorKind::QuestionMark,
            '\'' => OperatorKind::SingleQuote,
            '/' => OperatorKind::Slash,
            '*' => OperatorKind::Star,
            '~' => OperatorKind::Tilde,
            '_' => OperatorKind::Underscore,
            '`' => OperatorKind::Backtick,
            _ => OperatorKind::Unknown,
        }
    }
}

impl OperatorLexer for str {
    fn is_operator(&self) -> bool {
        matches!(
            self.as_ref(),
            "~" | "=" | ":" | "+" | "-" |
            "*" | "/" | "^" | "|" | "&" |
            "%" | ">" | "<" | "!" | "." |
            "@" | "\"" | "?" | "#" | "$" |
            "\\" | "`" | "_"
        )
    }
    fn to_operator(&self) -> OperatorKind {
        match self.as_ref() {
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
            "++" => OperatorKind::DoublePlus,
            "--" => OperatorKind::DoubleMinus,
            "**" => OperatorKind::DoubleStar,
            "//" => OperatorKind::DoubleSlash,
            "%%" => OperatorKind::DoublePercent,
            "~~" => OperatorKind::DoubleTilde,
            "^^" => OperatorKind::DoubleCaret,
            ".." => OperatorKind::DoubleDot,
            "..=" => OperatorKind::RangeInclusive,
            "..." => OperatorKind::TripleDot,
            "==" => OperatorKind::DoubleEqual,
            "!=" => OperatorKind::NotEqual,
            "<=" => OperatorKind::LessThanOrEqual,
            ">=" => OperatorKind::GreaterThanOrEqual,
            "===" => OperatorKind::TripleEqual,
            "!==" => OperatorKind::StrictNotEqual,
            "&&" => OperatorKind::DoubleAmpersand,
            "||" => OperatorKind::DoublePipe,
            "??" => OperatorKind::DoubleQuestionMark,
            "!!" => OperatorKind::DoubleExclamation,
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
            "**=" => OperatorKind::DoubleStarEqual,
            "%%=" => OperatorKind::DoublePercentEqual,
            "&&=" => OperatorKind::LogicalAndEqual,
            "||=" => OperatorKind::LogicalOrEqual,
            "?=" => OperatorKind::QuestionMarkEqual,
            "->" => OperatorKind::Arrow,
            "<-" => OperatorKind::LeftArrow,
            "|>" => OperatorKind::PipeRight,
            "<|" => OperatorKind::PipeLeft,
            "~>" => OperatorKind::AngleRight,
            "<~" => OperatorKind::AngleLeft,
            "-->" => OperatorKind::DoubleArrow,
            "<--" => OperatorKind::DoubleLeftArrow,
            "==>" => OperatorKind::DoubleFatArrow,
            "<==" => OperatorKind::DoubleFatLeftArrow,
            "::" => OperatorKind::DoubleColon,
            "/*" => OperatorKind::SlashStart,
            "*/" => OperatorKind::StarSlash,
            "///" => OperatorKind::TripleSlash,
            "/**" => OperatorKind::SlashDoubleStart,
            "//!" => OperatorKind::DoubleSlashExclamation,
            "@@" => OperatorKind::DoubleAt,
            "##" => OperatorKind::DoubleHash,
            "$$" => OperatorKind::DoubleDollar,
            "\\\\" => OperatorKind::DoubleBackslash,
            "__" => OperatorKind::DoubleUnderscore,
            _ => OperatorKind::Unknown,
        }
    }
}