#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    FatArrow,                // => (fat arrow)
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
                | OperatorKind::FatArrow
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

    pub fn decompound(&self) -> Option<OperatorKind> {
        match self {
            OperatorKind::LogicalAndEqual => Some(OperatorKind::DoubleAmpersand),
            OperatorKind::LogicalOrEqual => Some(OperatorKind::DoublePipe),
            OperatorKind::QuestionMarkEqual => Some(OperatorKind::QuestionMark),
            OperatorKind::RangeInclusive => Some(OperatorKind::DoubleDot),
            OperatorKind::AmpersandEqual => Some(OperatorKind::Ampersand),
            OperatorKind::PipeEqual => Some(OperatorKind::Pipe),
            OperatorKind::StarEqual => Some(OperatorKind::Star),
            OperatorKind::SlashEqual => Some(OperatorKind::Slash),
            OperatorKind::PercentEqual => Some(OperatorKind::Percent),
            OperatorKind::CaretEqual => Some(OperatorKind::Caret),
            OperatorKind::PlusEqual => Some(OperatorKind::Plus),
            OperatorKind::MinusEqual => Some(OperatorKind::Minus),
            OperatorKind::DoubleStarEqual => Some(OperatorKind::DoubleStar),
            OperatorKind::DoublePercentEqual => Some(OperatorKind::DoublePercent),
            _ => None,
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
            OperatorKind::FatArrow => "=>",
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
        };
        write!(f, "{}", op_str)
    }
}

pub trait OperatorLexer {
    fn is_operator(&self) -> bool;
    fn to_operator(&self) -> Option<OperatorKind>;
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

    fn to_operator(&self) -> Option<OperatorKind> {
        match self {
            '@' => Some(OperatorKind::At),
            '&' => Some(OperatorKind::Ampersand),
            '\\' => Some(OperatorKind::Backslash),
            '^' => Some(OperatorKind::Caret),
            ':' => Some(OperatorKind::Colon),
            '$' => Some(OperatorKind::Dollar),
            '.' => Some(OperatorKind::Dot),
            '"' => Some(OperatorKind::DoubleQuote),
            '=' => Some(OperatorKind::Equal),
            '!' => Some(OperatorKind::Exclamation),
            '>' => Some(OperatorKind::GreaterThan),
            '#' => Some(OperatorKind::Hash),
            '<' => Some(OperatorKind::LessThan),
            '-' => Some(OperatorKind::Minus),
            '%' => Some(OperatorKind::Percent),
            '|' => Some(OperatorKind::Pipe),
            '+' => Some(OperatorKind::Plus),
            '?' => Some(OperatorKind::QuestionMark),
            '\'' => Some(OperatorKind::SingleQuote),
            '/' => Some(OperatorKind::Slash),
            '*' => Some(OperatorKind::Star),
            '~' => Some(OperatorKind::Tilde),
            '_' => Some(OperatorKind::Underscore),
            '`' => Some(OperatorKind::Backtick),
            _ => None,
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
    fn to_operator(&self) -> Option<OperatorKind> {
        match self.as_ref() {
            // Single character operators
            "@" => Some(OperatorKind::At),
            "&" => Some(OperatorKind::Ampersand),
            "\\" => Some(OperatorKind::Backslash),
            "^" => Some(OperatorKind::Caret),
            ":" => Some(OperatorKind::Colon),
            "$" => Some(OperatorKind::Dollar),
            "." => Some(OperatorKind::Dot),
            "\"" => Some(OperatorKind::DoubleQuote),
            "=" => Some(OperatorKind::Equal),
            "!" => Some(OperatorKind::Exclamation),
            ">" => Some(OperatorKind::GreaterThan),
            "#" => Some(OperatorKind::Hash),
            "<" => Some(OperatorKind::LessThan),
            "-" => Some(OperatorKind::Minus),
            "%" => Some(OperatorKind::Percent),
            "|" => Some(OperatorKind::Pipe),
            "+" => Some(OperatorKind::Plus),
            "?" => Some(OperatorKind::QuestionMark),
            "'" => Some(OperatorKind::SingleQuote),
            "/" => Some(OperatorKind::Slash),
            "*" => Some(OperatorKind::Star),
            "~" => Some(OperatorKind::Tilde),
            "_" => Some(OperatorKind::Underscore),
            "`" => Some(OperatorKind::Backtick),

            // Word operators
            "in" => Some(OperatorKind::In),

            // Double character operators
            "++" => Some(OperatorKind::DoublePlus),
            "--" => Some(OperatorKind::DoubleMinus),
            "**" => Some(OperatorKind::DoubleStar),
            "//" => Some(OperatorKind::DoubleSlash),
            "%%" => Some(OperatorKind::DoublePercent),
            "~~" => Some(OperatorKind::DoubleTilde),
            "^^" => Some(OperatorKind::DoubleCaret),
            ".." => Some(OperatorKind::DoubleDot),
            "..=" => Some(OperatorKind::RangeInclusive),
            "..." => Some(OperatorKind::TripleDot),
            "==" => Some(OperatorKind::DoubleEqual),
            "!=" => Some(OperatorKind::NotEqual),
            "<=" => Some(OperatorKind::LessThanOrEqual),
            ">=" => Some(OperatorKind::GreaterThanOrEqual),
            "===" => Some(OperatorKind::TripleEqual),
            "!==" => Some(OperatorKind::StrictNotEqual),
            "&&" => Some(OperatorKind::DoubleAmpersand),
            "||" => Some(OperatorKind::DoublePipe),
            "??" => Some(OperatorKind::DoubleQuestionMark),
            "!!" => Some(OperatorKind::DoubleExclamation),
            "<<" => Some(OperatorKind::LeftShift),
            ">>" => Some(OperatorKind::RightShift),
            ":=" => Some(OperatorKind::ColonEqual),
            "=:" => Some(OperatorKind::EqualColon),
            "+=" => Some(OperatorKind::PlusEqual),
            "-=" => Some(OperatorKind::MinusEqual),
            "*=" => Some(OperatorKind::StarEqual),
            "/=" => Some(OperatorKind::SlashEqual),
            "%=" => Some(OperatorKind::PercentEqual),
            "^=" => Some(OperatorKind::CaretEqual),
            "&=" => Some(OperatorKind::AmpersandEqual),
            "|=" => Some(OperatorKind::PipeEqual),
            "**=" => Some(OperatorKind::DoubleStarEqual),
            "%%=" => Some(OperatorKind::DoublePercentEqual),
            "&&=" => Some(OperatorKind::LogicalAndEqual),
            "||=" => Some(OperatorKind::LogicalOrEqual),
            "?=" => Some(OperatorKind::QuestionMarkEqual),
            "->" => Some(OperatorKind::Arrow),
            "=>" => Some(OperatorKind::FatArrow),
            "<-" => Some(OperatorKind::LeftArrow),
            "|>" => Some(OperatorKind::PipeRight),
            "<|" => Some(OperatorKind::PipeLeft),
            "~>" => Some(OperatorKind::AngleRight),
            "<~" => Some(OperatorKind::AngleLeft),
            "-->" => Some(OperatorKind::DoubleArrow),
            "<--" => Some(OperatorKind::DoubleLeftArrow),
            "==>" => Some(OperatorKind::DoubleFatArrow),
            "<==" => Some(OperatorKind::DoubleFatLeftArrow),
            "::" => Some(OperatorKind::DoubleColon),
            "/*" => Some(OperatorKind::SlashStart),
            "*/" => Some(OperatorKind::StarSlash),
            "///" => Some(OperatorKind::TripleSlash),
            "/**" => Some(OperatorKind::SlashDoubleStart),
            "//!" => Some(OperatorKind::DoubleSlashExclamation),
            "@@" => Some(OperatorKind::DoubleAt),
            "##" => Some(OperatorKind::DoubleHash),
            "$$" => Some(OperatorKind::DoubleDollar),
            "\\\\" => Some(OperatorKind::DoubleBackslash),
            "__" => Some(OperatorKind::DoubleUnderscore),
            _ => None,
        }
    }
}