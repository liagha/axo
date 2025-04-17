use OperatorKind::*;
use core::fmt::Write;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
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
    RightAngle,              // > (greater than comparison)
    Hash,                    // # (attribute or preprocessor)
    LeftAngle,               // < (less than comparison)
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

    Composite(Vec<OperatorKind>),
}

impl PartialEq<[OperatorKind]> for OperatorKind {
    fn eq(&self, other: &[OperatorKind]) -> bool {
        match self {
            Composite(ops) => ops.as_slice() == other,
            _ => false,
        }
    }
}

impl OperatorKind {
    pub fn as_slice(&self) -> &[OperatorKind] {
        match self {
            Composite(ops) => ops.as_slice(),
            _ => core::slice::from_ref(self),
        }
    }

    pub fn precedence(&self) -> Option<u8> {
        match self.as_slice() {
            // Single operators
            [Dot] => Some(10),
            [Colon] => Some(9),
            [Star] | [Slash] | [Percent] => Some(6),
            [Plus] | [Minus] => Some(5),
            [LeftAngle] | [RightAngle] => Some(3),
            [Ampersand] | [Caret] | [Pipe] => Some(1),
            [In] | [Equal] => Some(0),

            // Composite operators
            [Colon, Colon] => Some(10),
            [Star, Star] | [Caret, Caret] => Some(7),
            [Slash, Slash] | [Percent, Percent] => Some(6),
            [Dot, Dot] | [Dot, Dot, Equal] | [Dot, Dot, Dot] => Some(4),
            [LeftAngle, Equal] | [RightAngle, Equal] => Some(3),
            [Equal, Equal] | [Exclamation, Equal] => Some(2),
            [Ampersand, Ampersand] => Some(1),
            [Pipe, Pipe] => Some(0),
            [Colon, Equal]
            | [Plus, Equal]
            | [Minus, Equal]
            | [Star, Equal]
            | [Slash, Equal]
            | [Percent, Equal]
            | [Caret, Equal]
            | [Ampersand, Equal]
            | [Pipe, Equal]
            | [Star, Star, Equal]
            | [Percent, Percent, Equal]
            | [Ampersand, Ampersand, Equal]
            | [Pipe, Pipe, Equal]
            | [QuestionMark, Equal] => Some(0),

            // Arrows
            [Minus, RightAngle]
            | [LeftAngle, Minus]
            | [Equal, RightAngle]
            | [Pipe, RightAngle]
            | [LeftAngle, Pipe]
            | [Minus, Minus, RightAngle]
            | [LeftAngle, Minus, Minus]
            | [Equal, Equal, RightAngle]
            | [LeftAngle, Equal, Equal] => Some(0),

            _ => None,
        }
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self.as_slice(),
            [Minus, RightAngle]
            | [Equal, RightAngle]
            | [Plus, RightAngle]
            | [Minus, Minus, RightAngle]
            | [Equal, Equal, RightAngle]
        )
    }

    pub fn is_left_arrow(&self) -> bool {
        matches!(self.as_slice(),
            [LeftAngle, Pipe]
            | [LeftAngle, Minus]
            | [LeftAngle, Minus, Minus]
            | [LeftAngle, Equal, Equal]
        )
    }

    pub fn is_prefix(&self) -> bool {
        match self {
            Exclamation | Plus | Minus | Tilde | Ampersand => true,
            _ => matches!(self.as_slice(), [Plus, Plus] | [Minus, Minus]),
        }
    }

    pub fn is_postfix(&self) -> bool {
        matches!(self.as_slice(), [Plus, Plus] | [Minus, Minus])
    }

    pub fn decompound(&self) -> Option<OperatorKind> {
        match self.as_slice() {
            [Ampersand, Ampersand, Equal] => Some(Composite(vec![Ampersand, Ampersand])),
            [Pipe, Pipe, Equal] => Some(Composite(vec![Pipe, Pipe])),
            [QuestionMark, Equal] => Some(QuestionMark),
            [Dot, Dot, Equal] => Some(Composite(vec![Dot, Dot])),
            [Ampersand, Equal] => Some(Ampersand),
            [Pipe, Equal] => Some(Pipe),
            [Star, Equal] => Some(Star),
            [Slash, Equal] => Some(Slash),
            [Percent, Equal] => Some(Percent),
            [Caret, Equal] => Some(Caret),
            [Plus, Equal] => Some(Plus),
            [Minus, Equal] => Some(Minus),
            [Star, Star, Equal] => Some(Composite(vec![Star, Star])),
            [Percent, Percent, Equal] => Some(Composite(vec![Percent, Percent])),
            _ => None,
        }
    }
}

impl core::fmt::Display for OperatorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            In => write!(f, "in"),
            Tilde => write!(f, "~"),
            Equal => write!(f, "="),
            Colon => write!(f, ":"),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Star => write!(f, "*"),
            Slash => write!(f, "/"),
            Caret => write!(f, "^"),
            Pipe => write!(f, "|"),
            Ampersand => write!(f, "&"),
            Percent => write!(f, "%"),
            RightAngle => write!(f, ">"),
            LeftAngle => write!(f, "<"),
            Exclamation => write!(f, "!"),
            Dot => write!(f, "."),
            At => write!(f, "@"),
            Hash => write!(f, "#"),
            QuestionMark => write!(f, "?"),
            Dollar => write!(f, "$"),
            Backslash => write!(f, "\\"),
            DoubleQuote => write!(f, "\""),
            SingleQuote => write!(f, "'"),
            Backtick => write!(f, "`"),
            Underscore => write!(f, "_"),
            Composite(ops) => {
                // Handle composite operators
                let mut result = String::new();
                for op in ops {
                    // Recursively format each operator in the composite
                    write!(result, "{}", op)?;
                }
                write!(f, "{}", result)
            },
        }
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
            '@' => At,
            '&' => Ampersand,
            '\\' => Backslash,
            '^' => Caret,
            ':' => Colon,
            '$' => Dollar,
            '.' => Dot,
            '"' => DoubleQuote,
            '=' => Equal,
            '!' => Exclamation,
            '>' => RightAngle,
            '#' => Hash,
            '<' => LeftAngle,
            '-' => Minus,
            '%' => Percent,
            '|' => Pipe,
            '+' => Plus,
            '?' => QuestionMark,
            '\'' => SingleQuote,
            '/' => Slash,
            '*' => Star,
            '~' => Tilde,
            '_' => Underscore,
            '`' => Backtick,
            _ => unreachable!(),
        }
    }
}

impl OperatorLexer for str {
    fn is_operator(&self) -> bool {
        matches!(
            self,
            "~" | "=" | ":" | "+" | "-" |
            "*" | "/" | "^" | "|" | "&" |
            "%" | ">" | "<" | "!" | "." |
            "@" | "\"" | "?" | "#" | "$" |
            "\\" | "`" | "_" | "in"
        )
    }

    fn to_operator(&self) -> OperatorKind {
        match self {
            // Single character operators
            "@" => At,
            "&" => Ampersand,
            "\\" => Backslash,
            "^" => Caret,
            ":" => Colon,
            "$" => Dollar,
            "." => Dot,
            "\"" => DoubleQuote,
            "=" => Equal,
            "!" => Exclamation,
            ">" => RightAngle,
            "#" => Hash,
            "<" => LeftAngle,
            "-" => Minus,
            "%" => Percent,
            "|" => Pipe,
            "+" => Plus,
            "?" => QuestionMark,
            "'" => SingleQuote,
            "/" => Slash,
            "*" => Star,
            "~" => Tilde,
            "_" => Underscore,
            "`" => Backtick,

            "in" => In,

            _ => {
                let mut ops = Vec::new();

                for c in self.chars() {
                    ops.push(c.to_operator());
                }

                Composite(ops)
            },
        }
    }
}