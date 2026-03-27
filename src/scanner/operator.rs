use {
    crate::{
        data::{slice, Str},
        format::{Debug, Display, Formatter, Result},
        internal::cache::{Decode, Encode},
        scanner::Character,
    },
    OperatorKind::*,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum OperatorKind {
    At,
    Ampersand,
    Backslash,
    Caret,
    Colon,
    Dollar,
    Dot,
    DoubleQuote,
    Equal,
    Exclamation,
    RightAngle,
    Hash,
    LeftAngle,
    Minus,
    Percent,
    Pipe,
    Plus,
    QuestionMark,
    SingleQuote,
    Slash,
    Star,
    Tilde,
    Backtick,

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
            _ => slice::from_ref(self),
        }
    }

    pub fn precedence(&self) -> Option<u8> {
        match self.as_slice() {
            [Dot] => Some(10),
            [Colon] => Some(9),
            [Star] | [Slash] | [Percent] => Some(6),
            [Plus] | [Minus] => Some(5),
            [LeftAngle] | [RightAngle] => Some(3),
            [LeftAngle, LeftAngle] | [RightAngle, RightAngle] => Some(4),
            [Ampersand] | [Caret] | [Pipe] => Some(1),
            [Equal] => Some(0),

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
        matches!(
            self.as_slice(),
            [Minus, RightAngle]
                | [Equal, RightAngle]
                | [Plus, RightAngle]
                | [Minus, Minus, RightAngle]
                | [Equal, Equal, RightAngle]
        )
    }

    pub fn is_left_arrow(&self) -> bool {
        matches!(
            self.as_slice(),
            [LeftAngle, Pipe]
                | [LeftAngle, Minus]
                | [LeftAngle, Minus, Minus]
                | [LeftAngle, Equal, Equal]
        )
    }

    pub fn is_prefix(&self) -> bool {
        match self {
            Exclamation | Plus | Minus | Tilde | Ampersand | Star => true,
            _ => matches!(self.as_slice(), [Plus, Plus] | [Minus, Minus]),
        }
    }

    pub fn is_suffix(&self) -> bool {
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

impl Display for OperatorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
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
            Composite(operators) => {
                write!(
                    f,
                    "{}",
                    operators
                        .iter()
                        .map(|operator| operator.to_string())
                        .collect::<Str>()
                )
            }
        }
    }
}

pub trait Operator {
    fn is_operator(&self) -> bool;
    fn to_operator(&self) -> OperatorKind;
}

impl<'character> Operator for Character<'character> {
    fn is_operator(&self) -> bool {
        matches!(
            self.value,
            '~' | '='
                | ':'
                | '+'
                | '-'
                | '*'
                | '/'
                | '^'
                | '|'
                | '&'
                | '%'
                | '>'
                | '<'
                | '!'
                | '.'
                | '@'
                | '\''
                | '?'
                | '#'
                | '$'
                | '\\'
                | '`'
                | '"'
        )
    }

    fn to_operator(&self) -> OperatorKind {
        match self.value {
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
            '`' => Backtick,
            _ => unreachable!(),
        }
    }
}

impl Operator for char {
    fn is_operator(&self) -> bool {
        matches!(
            self,
            '~' | '='
                | ':'
                | '+'
                | '-'
                | '*'
                | '/'
                | '^'
                | '|'
                | '&'
                | '%'
                | '>'
                | '<'
                | '!'
                | '.'
                | '@'
                | '\''
                | '?'
                | '#'
                | '$'
                | '\\'
                | '`'
                | '"'
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
            '`' => Backtick,
            _ => unreachable!(),
        }
    }
}

impl Operator for str {
    fn is_operator(&self) -> bool {
        matches!(
            self,
            "~" | "="
                | ":"
                | "+"
                | "-"
                | "*"
                | "/"
                | "^"
                | "|"
                | "&"
                | "%"
                | ">"
                | "<"
                | "!"
                | "."
                | "@"
                | "\""
                | "?"
                | "'"
                | "#"
                | "$"
                | "\\"
                | "`"
                | "in"
        )
    }

    fn to_operator(&self) -> OperatorKind {
        match self {
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
            "`" => Backtick,

            _ => {
                let mut ops = Vec::new();

                for c in self.chars() {
                    ops.push(c.to_operator());
                }

                Composite(ops)
            }
        }
    }
}

impl Encode for OperatorKind {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            At => buffer.push(0),
            Ampersand => buffer.push(1),
            Backslash => buffer.push(2),
            Caret => buffer.push(3),
            Colon => buffer.push(4),
            Dollar => buffer.push(5),
            Dot => buffer.push(6),
            DoubleQuote => buffer.push(7),
            Equal => buffer.push(8),
            Exclamation => buffer.push(9),
            RightAngle => buffer.push(10),
            Hash => buffer.push(11),
            LeftAngle => buffer.push(12),
            Minus => buffer.push(13),
            Percent => buffer.push(14),
            Pipe => buffer.push(15),
            Plus => buffer.push(16),
            QuestionMark => buffer.push(17),
            SingleQuote => buffer.push(18),
            Slash => buffer.push(19),
            Star => buffer.push(20),
            Tilde => buffer.push(21),
            Backtick => buffer.push(22),
            Composite(ops) => {
                buffer.push(23);
                ops.encode(buffer);
            }
        }
    }
}

impl<'a> Decode<'a> for OperatorKind {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => At,
            1 => Ampersand,
            2 => Backslash,
            3 => Caret,
            4 => Colon,
            5 => Dollar,
            6 => Dot,
            7 => DoubleQuote,
            8 => Equal,
            9 => Exclamation,
            10 => RightAngle,
            11 => Hash,
            12 => LeftAngle,
            13 => Minus,
            14 => Percent,
            15 => Pipe,
            16 => Plus,
            17 => QuestionMark,
            18 => SingleQuote,
            19 => Slash,
            20 => Star,
            21 => Tilde,
            22 => Backtick,
            23 => Composite(Vec::decode(buffer, cursor)),
            _ => panic!(),
        }
    }
}
