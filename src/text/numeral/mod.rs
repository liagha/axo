use {
    crate::{
        format::{self, Debug, Display, Formatter},
        data::{
            string::FromStr,
            memory::PhantomData,
        },
    }
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumberFormat {
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
    Scientific,
    Custom(u8),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ParseNumberError {
    EmptyString,
    InvalidRadix(u8),
    InvalidDigit(char),
    MultipleDecimalPoints,
    Overflow,
    MalformedExponent,
    InvalidFormat(String),
    TypeError(String),
}

impl Display for ParseNumberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self {
            Self::EmptyString => write!(f, "cannot parse an empty string"),
            Self::InvalidRadix(radix) => write!(f, "invalid radix: {}, must be between 2 and 36", radix),
            Self::InvalidDigit(c) => write!(f, "invalid digit for the given radix: '{}'", c),
            Self::MultipleDecimalPoints => write!(f, "multiple decimal points found in input"),
            Self::Overflow => write!(f, "numeric overflow occurred"),
            Self::MalformedExponent => write!(f, "malformed exponent in scientific notation"),
            Self::InvalidFormat(details) => write!(f, "invalid number format: {}", details),
            Self::TypeError(details) => write!(f, "type reporter: {}", details),
        }
    }
}

pub trait NumericParser: Sized {
    fn from_str_radix(s: &str, radix: u8) -> Result<Self, ParseNumberError>;

    fn parse(s: &str) -> Result<Self, ParseNumberError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseNumberError::EmptyString);
        }

        let (s, format) = detect_number_format(s)?;

        match format {
            NumberFormat::Decimal => Self::from_str_radix(s, 10),
            NumberFormat::Hexadecimal => Self::from_str_radix(s, 16),
            NumberFormat::Octal => Self::from_str_radix(s, 8),
            NumberFormat::Binary => Self::from_str_radix(s, 2),
            NumberFormat::Scientific => parse_scientific(s).and_then(|f| {
                Self::from_str_radix(&f.to_string(), 10)
            }),
            NumberFormat::Custom(radix) => Self::from_str_radix(s, radix),
        }
    }
}

fn detect_number_format(s: &str) -> Result<(&str, NumberFormat), ParseNumberError> {
    if s.is_empty() {
        return Err(ParseNumberError::EmptyString);
    }

    if s.len() >= 2 && s.starts_with('0') {
        match s.chars().nth(1).unwrap() {
            'x' | 'X' => return Ok((&s[2..], NumberFormat::Hexadecimal)),
            'o' | 'O' => return Ok((&s[2..], NumberFormat::Octal)),
            'b' | 'B' => return Ok((&s[2..], NumberFormat::Binary)),
            _ => {}
        }
    }

    if s.contains(['e', 'E']) && !s.starts_with(['e', 'E']) {
        return Ok((s, NumberFormat::Scientific));
    }

    Ok((s, NumberFormat::Decimal))
}

fn parse_scientific(s: &str) -> Result<f64, ParseNumberError> {
    let parts: Vec<&str> = s.split(['e', 'E']).collect();
    if parts.len() != 2 {
        return Err(ParseNumberError::MalformedExponent);
    }

    let base = parts[0].parse::<f64>().map_err(|_| {
        ParseNumberError::InvalidFormat(format!("invalid base: {}", parts[0]))
    })?;

    let exp = parts[1].parse::<i32>().map_err(|_| {
        ParseNumberError::InvalidFormat(format!("invalid exponent: {}", parts[1]))
    })?;

    Ok(base * 10.0_f64.powi(exp))
}

macro_rules! impl_numeric_parser_for_integer {
    ($t:ty, $doc:expr) => {
        #[doc = $doc]
        impl NumericParser for $t {
            fn from_str_radix(s: &str, radix: u8) -> Result<Self, ParseNumberError> {
                if radix < 2 || radix > 36 {
                    return Err(ParseNumberError::InvalidRadix(radix));
                }

                let s = s.trim();
                if s.is_empty() {
                    return Err(ParseNumberError::EmptyString);
                }

                let (is_negative, s) = if s.starts_with('-') {
                    (true, &s[1..])
                } else if s.starts_with('+') {
                    (false, &s[1..])
                } else {
                    (false, s)
                };

                if s.is_empty() {
                    return Err(ParseNumberError::InvalidFormat("only a sign character found".to_string()));
                }

                let mut result: $t = 0;
                for c in s.chars() {
                    let digit = match c.to_digit(radix as u32) {
                        Some(d) => d as $t,
                        None => return Err(ParseNumberError::InvalidDigit(c)),
                    };

                    if let Some(new_result) = result.checked_mul(radix as $t) {
                        if let Some(new_result) = new_result.checked_add(digit) {
                            result = new_result;
                        } else {
                            return Err(ParseNumberError::Overflow);
                        }
                    } else {
                        return Err(ParseNumberError::Overflow);
                    }
                }

                if is_negative {
                    result = result.wrapping_neg();

                    if result > 0 && s != "0" {
                        return Err(ParseNumberError::Overflow);
                    }
                }

                Ok(result)
            }
        }
    };
}

macro_rules! impl_numeric_parser_for_float {
    ($t:ty, $doc:expr) => {
        #[doc = $doc]
        impl NumericParser for $t {
            fn from_str_radix(s: &str, radix: u8) -> Result<Self, ParseNumberError> {
                if radix != 10 && !s.contains(['e', 'E']) {
                    let s = s.trim();
                    if s.is_empty() {
                        return Err(ParseNumberError::EmptyString);
                    }

                    let (is_negative, s) = if s.starts_with('-') {
                        (true, &s[1..])
                    } else if s.starts_with('+') {
                        (false, &s[1..])
                    } else {
                        (false, s)
                    };

                    if s.is_empty() {
                        return Err(ParseNumberError::InvalidFormat("only a sign character found".to_string()));
                    }

                    let parts: Vec<&str> = s.split('.').collect();
                    if parts.len() > 2 {
                        return Err(ParseNumberError::MultipleDecimalPoints);
                    }

                    let int_part = if parts[0].is_empty() {
                        0.0
                    } else {
                        let mut result = 0.0;
                        for c in parts[0].chars() {
                            let digit = match c.to_digit(radix as u32) {
                                Some(d) => d as $t,
                                None => return Err(ParseNumberError::InvalidDigit(c)),
                            };
                            result = result * (radix as $t) + digit;
                        }
                        result
                    };

                    let frac_part = if parts.len() == 2 {
                        let mut result = 0.0;
                        let mut factor = 1.0 / (radix as $t);
                        for c in parts[1].chars() {
                            let digit = match c.to_digit(radix as u32) {
                                Some(d) => d as $t,
                                None => return Err(ParseNumberError::InvalidDigit(c)),
                            };
                            result += digit * factor;
                            factor /= radix as $t;
                        }
                        result
                    } else {
                        0.0
                    };

                    let result = int_part + frac_part;
                    Ok(if is_negative { -result } else { result })
                } else {
                    <$t as FromStr>::from_str(s).map_err(|_| {
                        ParseNumberError::InvalidFormat(format!("failed to parse '{}' as {}", s, stringify!($t)))
                    })
                }
            }
        }
    };
}

impl_numeric_parser_for_integer!(u8, "Implementation for u8");
impl_numeric_parser_for_integer!(u16, "Implementation for u16");
impl_numeric_parser_for_integer!(u32, "Implementation for u32");
impl_numeric_parser_for_integer!(u64, "Implementation for u64");
impl_numeric_parser_for_integer!(u128, "Implementation for u128");
impl_numeric_parser_for_integer!(usize, "Implementation for usize");

impl_numeric_parser_for_integer!(i8, "Implementation for i8");
impl_numeric_parser_for_integer!(i16, "Implementation for i16");
impl_numeric_parser_for_integer!(i32, "Implementation for i32");
impl_numeric_parser_for_integer!(i64, "Implementation for i64");
impl_numeric_parser_for_integer!(i128, "Implementation for i128");
impl_numeric_parser_for_integer!(isize, "Implementation for isize");

impl_numeric_parser_for_float!(f32, "Implementation for f32");
impl_numeric_parser_for_float!(f64, "Implementation for f64");

pub struct NumberParser<T> {
    _marker: PhantomData<T>,
}

impl<T: NumericParser> NumberParser<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn parse(&self, s: &str) -> Result<T, ParseNumberError> {
        T::parse(s)
    }

    pub fn parse_radix(&self, s: &str, radix: u8) -> Result<T, ParseNumberError> {
        T::from_str_radix(s, radix)
    }
}

pub fn parser<T: NumericParser>() -> NumberParser<T> {
    NumberParser::new()
}

#[derive(Clone, Debug, PartialEq)]
pub enum AutoNumber {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
}

impl AutoNumber {
    pub fn is_int(&self) -> bool {
        match self {
            Self::U8(_) | Self::U16(_) | Self::U32(_) | Self::U64(_) | Self::U128(_) |
            Self::I8(_) | Self::I16(_) | Self::I32(_) | Self::I64(_) | Self::I128(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::F32(_) | Self::F64(_) => true,
            _ => false,
        }
    }

    pub fn as_int(&self) -> Result<i64, ParseNumberError> {
        match self {
            Self::U8(v) => Ok(*v as i64),
            Self::U16(v) => Ok(*v as i64),
            Self::U32(v) => Ok(*v as i64),
            Self::U64(v) => {
                if *v <= i64::MAX as u64 {
                    Ok(*v as i64)
                } else {
                    Err(ParseNumberError::Overflow)
                }
            }
            Self::U128(v) => {
                if *v <= i64::MAX as u128 {
                    Ok(*v as i64)
                } else {
                    Err(ParseNumberError::Overflow)
                }
            }
            Self::I8(v) => Ok(*v as i64),
            Self::I16(v) => Ok(*v as i64),
            Self::I32(v) => Ok(*v as i64),
            Self::I64(v) => Ok(*v),
            Self::I128(v) => {
                if *v >= i64::MIN as i128 && *v <= i64::MAX as i128 {
                    Ok(*v as i64)
                } else {
                    Err(ParseNumberError::Overflow)
                }
            }
            _ => Err(ParseNumberError::TypeError("not an integer type".to_string())),
        }
    }

    pub fn as_float(&self) -> Result<f64, ParseNumberError> {
        match self {
            Self::U8(v) => Ok(*v as f64),
            Self::U16(v) => Ok(*v as f64),
            Self::U32(v) => Ok(*v as f64),
            Self::U64(v) => Ok(*v as f64),
            Self::U128(v) => {
                if *v > 9007199254740992u128 {
                    Err(ParseNumberError::Overflow)
                } else {
                    Ok(*v as f64)
                }
            }
            Self::I8(v) => Ok(*v as f64),
            Self::I16(v) => Ok(*v as f64),
            Self::I32(v) => Ok(*v as f64),
            Self::I64(v) => Ok(*v as f64),
            Self::I128(v) => {
                if *v > 9007199254740992i128 || *v < -9007199254740992i128 {
                    Err(ParseNumberError::Overflow)
                } else {
                    Ok(*v as f64)
                }
            }
            Self::F32(v) => Ok(*v as f64),
            Self::F64(v) => Ok(*v),
        }
    }
}

pub struct AutoNumberParser;

impl AutoNumberParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, s: &str) -> Result<AutoNumber, ParseNumberError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseNumberError::EmptyString);
        }

        if s.contains('.') || s.contains(['e', 'E']) {
            if let Ok(value) = parser::<f32>().parse(s) {
                return Ok(AutoNumber::F32(value));
            }

            if let Ok(value) = parser::<f64>().parse(s) {
                return Ok(AutoNumber::F64(value));
            }

            return Err(ParseNumberError::InvalidFormat(format!("Could not parse '{}' as a floating point number", s)));
        }

        let is_negative = s.starts_with('-');

        if is_negative {
            if let Ok(value) = parser::<i8>().parse(s) {
                return Ok(AutoNumber::I8(value));
            }

            if let Ok(value) = parser::<i16>().parse(s) {
                return Ok(AutoNumber::I16(value));
            }

            if let Ok(value) = parser::<i32>().parse(s) {
                return Ok(AutoNumber::I32(value));
            }

            if let Ok(value) = parser::<i64>().parse(s) {
                return Ok(AutoNumber::I64(value));
            }

            if let Ok(value) = parser::<i128>().parse(s) {
                return Ok(AutoNumber::I128(value));
            }

            Err(ParseNumberError::Overflow)
        } else {
            if let Ok(value) = parser::<u8>().parse(s) {
                return Ok(AutoNumber::U8(value));
            }

            if let Ok(value) = parser::<u16>().parse(s) {
                return Ok(AutoNumber::U16(value));
            }

            if let Ok(value) = parser::<u32>().parse(s) {
                return Ok(AutoNumber::U32(value));
            }

            if let Ok(value) = parser::<u64>().parse(s) {
                return Ok(AutoNumber::U64(value));
            }

            if let Ok(value) = parser::<u128>().parse(s) {
                return Ok(AutoNumber::U128(value));
            }

            Err(ParseNumberError::Overflow)
        }
    }

    pub fn parse_radix(&self, s: &str, radix: u8) -> Result<AutoNumber, ParseNumberError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseNumberError::EmptyString);
        }

        if s.contains('.') {
            if let Ok(value) = parser::<f32>().parse_radix(s, radix) {
                return Ok(AutoNumber::F32(value));
            }

            if let Ok(value) = parser::<f64>().parse_radix(s, radix) {
                return Ok(AutoNumber::F64(value));
            }

            return Err(ParseNumberError::InvalidFormat(format!("Could not parse '{}' as a floating point number", s)));
        }

        let is_negative = s.starts_with('-');

        if is_negative {
            if let Ok(value) = parser::<i8>().parse_radix(s, radix) {
                return Ok(AutoNumber::I8(value));
            }

            if let Ok(value) = parser::<i16>().parse_radix(s, radix) {
                return Ok(AutoNumber::I16(value));
            }

            if let Ok(value) = parser::<i32>().parse_radix(s, radix) {
                return Ok(AutoNumber::I32(value));
            }

            if let Ok(value) = parser::<i64>().parse_radix(s, radix) {
                return Ok(AutoNumber::I64(value));
            }

            if let Ok(value) = parser::<i128>().parse_radix(s, radix) {
                return Ok(AutoNumber::I128(value));
            }

            Err(ParseNumberError::Overflow)
        } else {
            if let Ok(value) = parser::<u8>().parse_radix(s, radix) {
                return Ok(AutoNumber::U8(value));
            }

            if let Ok(value) = parser::<u16>().parse_radix(s, radix) {
                return Ok(AutoNumber::U16(value));
            }

            if let Ok(value) = parser::<u32>().parse_radix(s, radix) {
                return Ok(AutoNumber::U32(value));
            }

            if let Ok(value) = parser::<u64>().parse_radix(s, radix) {
                return Ok(AutoNumber::U64(value));
            }

            if let Ok(value) = parser::<u128>().parse_radix(s, radix) {
                return Ok(AutoNumber::U128(value));
            }

            Err(ParseNumberError::Overflow)
        }
    }
}

pub fn auto_parser() -> AutoNumberParser {
    AutoNumberParser::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        let parser = parser::<i32>();
        assert_eq!(parser.parse("123"), Ok(123));
        assert_eq!(parser.parse("-123"), Ok(-123));
        assert_eq!(parser.parse("+123"), Ok(123));
    }

    #[test]
    fn test_parse_hexadecimal() {
        let parser = parser::<i32>();
        assert_eq!(parser.parse("0xFF"), Ok(255));
        assert_eq!(parser.parse("0xff"), Ok(255));
        assert_eq!(parser.parse("-0xFF"), Ok(-255));
    }

    #[test]
    fn test_parse_octal() {
        let parser = parser::<i32>();
        assert_eq!(parser.parse("0o10"), Ok(8));
        assert_eq!(parser.parse("0o17"), Ok(15));
        assert_eq!(parser.parse("-0o10"), Ok(-8));
    }

    #[test]
    fn test_parse_binary() {
        let parser = parser::<i32>();
        assert_eq!(parser.parse("0b1010"), Ok(10));
        assert_eq!(parser.parse("-0b1010"), Ok(-10));
    }

    #[test]
    fn test_parse_scientific() {
        let parser = parser::<f64>();
        assert_eq!(parser.parse("1.23e2"), Ok(123.0));
        assert_eq!(parser.parse("1.23E-2"), Ok(0.0123));
        assert_eq!(parser.parse("-1.23e2"), Ok(-123.0));
    }

    #[test]
    fn test_custom_radix() {
        let parser = parser::<i32>();
        assert_eq!(parser.parse_radix("FF", 16), Ok(255));
        assert_eq!(parser.parse_radix("101", 2), Ok(5));
        assert_eq!(parser.parse_radix("Z", 36), Ok(35));
    }

    #[test]
    fn test_errors() {
        let parser = parser::<i32>();
        assert!(matches!(parser.parse(""), Err(ParseNumberError::EmptyString)));
        assert!(matches!(parser.parse_radix("F", 15), Err(ParseNumberError::InvalidDigit('F'))));
        assert!(matches!(parser.parse_radix("123", 1), Err(ParseNumberError::InvalidRadix(1))));
        assert!(matches!(parser.parse_radix("123", 37), Err(ParseNumberError::InvalidRadix(37))));
    }

    #[test]
    fn test_overflow() {
        let parser = parser::<u8>();
        assert!(matches!(parser.parse("256"), Err(ParseNumberError::Overflow)));
    }

    #[test]
    fn test_float_parsing() {
        let parser = parser::<f64>();
        assert_eq!(parser.parse("123.456"), Ok(123.456));
        assert_eq!(parser.parse("-123.456"), Ok(-123.456));
        assert_eq!(parser.parse_radix("A.B", 16), Ok(10.6875));
    }

    #[test]
    fn test_auto_parser() {
        let parser = auto_parser();

        assert!(matches!(parser.parse("42"), Ok(AutoNumber::U8(42))));
        assert!(matches!(parser.parse("256"), Ok(AutoNumber::U16(256))));
        assert!(matches!(parser.parse("65536"), Ok(AutoNumber::U32(_))));

        assert!(matches!(parser.parse("-42"), Ok(AutoNumber::I8(-42))));
        assert!(matches!(parser.parse("-129"), Ok(AutoNumber::I16(-129))));
        assert!(matches!(parser.parse("-32769"), Ok(AutoNumber::I32(_))));

        assert!(matches!(parser.parse("3.14"), Ok(AutoNumber::F32(_))));
        assert!(matches!(parser.parse("1.23e-2"), Ok(AutoNumber::F32(_))));

        assert!(matches!(parser.parse("1.23e38"), Ok(AutoNumber::F64(_))));

        assert!(matches!(parser.parse_radix("FF", 16), Ok(AutoNumber::U8(255))));
        assert!(matches!(parser.parse_radix("FFFF", 16), Ok(AutoNumber::U16(65535))));
    }

    #[test]
    fn test_is_int_and_is_float() {
        let parser = auto_parser();

        let int_value = parser.parse("42").unwrap();
        let float_value = parser.parse("3.14").unwrap();

        assert!(int_value.is_int());
        assert!(!int_value.is_float());

        assert!(!float_value.is_int());
        assert!(float_value.is_float());
    }

    #[test]
    fn test_as_int_and_as_float() {
        let parser = auto_parser();

        let int_value = parser.parse("42").unwrap();
        let float_value = parser.parse("3.14").unwrap();
        let large_value = parser.parse("9223372036854775808").unwrap();

        assert_eq!(int_value.as_int(), Ok(42));
        assert!(matches!(float_value.as_int(), Err(ParseNumberError::TypeError(_))));
        assert!(matches!(large_value.as_int(), Err(ParseNumberError::Overflow)));

        assert_eq!(int_value.as_float(), Ok(42.0));
        assert_eq!(float_value.as_float().unwrap(), 3.14);
        assert!(matches!(parser.parse("9007199254740993").unwrap().as_float(), Ok(_)));
    }
}