use core::fmt;
use core::str::FromStr;

/// Represents different number formats supported by the parser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormat {
    /// Decimal format (base 10)
    Decimal,
    /// Hexadecimal format (base 16, prefixed with 0x)
    Hexadecimal,
    /// Octal format (base 8, prefixed with 0o)
    Octal,
    /// Binary format (base 2, prefixed with 0b)
    Binary,
    /// Scientific notation (e.g., 1.23e-4)
    Scientific,
    /// Custom radix (2-36)
    Custom(u8),
}

/// Possible error types that can occur during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNumberError {
    /// The input string is empty
    EmptyString,
    /// The radix provided is invalid (must be 2-36)
    InvalidRadix(u8),
    /// The input contains invalid digits for the given radix
    InvalidDigit(char),
    /// Multiple decimal points were found
    MultipleDecimalPoints,
    /// The input is too large for the target numeric type
    Overflow,
    /// Scientific notation is malformed
    MalformedExponent,
    /// The input string format is invalid
    InvalidFormat(String),
    /// Error when trying to extract a value of the wrong type
    TypeError(String),
}

impl fmt::Display for ParseNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyString => write!(f, "cannot parse an empty string"),
            Self::InvalidRadix(radix) => write!(f, "invalid radix: {}, must be between 2 and 36", radix),
            Self::InvalidDigit(c) => write!(f, "invalid digit for the given radix: '{}'", c),
            Self::MultipleDecimalPoints => write!(f, "multiple decimal points found in input"),
            Self::Overflow => write!(f, "numeric overflow occurred"),
            Self::MalformedExponent => write!(f, "malformed exponent in scientific notation"),
            Self::InvalidFormat(details) => write!(f, "invalid number format: {}", details),
            Self::TypeError(details) => write!(f, "type error: {}", details),
        }
    }
}

impl core::error::Error for ParseNumberError {}

/// Trait for numeric types that can be parsed from strings
pub trait NumericParser: Sized {
    /// Parse a string into the numeric type with the specified radix
    fn from_str_radix(s: &str, radix: u8) -> Result<Self, ParseNumberError>;

    /// Parse a string into the numeric type with auto-detection of format
    fn parse(s: &str) -> Result<Self, ParseNumberError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseNumberError::EmptyString);
        }

        // Detect format and radix
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

/// Detects the format of a number string and returns the cleaned string and format
fn detect_number_format(s: &str) -> Result<(&str, NumberFormat), ParseNumberError> {
    if s.is_empty() {
        return Err(ParseNumberError::EmptyString);
    }

    // Check for explicit radix prefix (0x, 0o, 0b)
    if s.len() >= 2 && s.starts_with('0') {
        match s.chars().nth(1).unwrap() {
            'x' | 'X' => return Ok((&s[2..], NumberFormat::Hexadecimal)),
            'o' | 'O' => return Ok((&s[2..], NumberFormat::Octal)),
            'b' | 'B' => return Ok((&s[2..], NumberFormat::Binary)),
            _ => {}
        }
    }

    // Check for scientific notation
    if s.contains(['e', 'E']) && !s.starts_with(['e', 'E']) {
        return Ok((s, NumberFormat::Scientific));
    }

    // If no special format detected, use decimal by default
    Ok((s, NumberFormat::Decimal))
}

/// Parses a scientific notation number into a standard float
fn parse_scientific(s: &str) -> Result<f64, ParseNumberError> {
    let parts: Vec<&str> = s.split(['e', 'E']).collect();
    if parts.len() != 2 {
        return Err(ParseNumberError::MalformedExponent);
    }

    // Parse base part
    let base = parts[0].parse::<f64>().map_err(|_| {
        ParseNumberError::InvalidFormat(format!("invalid base: {}", parts[0]))
    })?;

    // Parse exponent part
    let exp = parts[1].parse::<i32>().map_err(|_| {
        ParseNumberError::InvalidFormat(format!("invalid exponent: {}", parts[1]))
    })?;

    // Calculate the result using the power of the exponent
    Ok(base * 10.0_f64.powi(exp))
}

// Implementation for primitive numeric types
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

                // Handle sign
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

                // Parse digits
                let mut result: $t = 0;
                for c in s.chars() {
                    let digit = match c.to_digit(radix as u32) {
                        Some(d) => d as $t,
                        None => return Err(ParseNumberError::InvalidDigit(c)),
                    };

                    // Check for overflow
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

                // Apply sign
                if is_negative {
                    result = result.wrapping_neg();

                    // Check for overflow on negation (e.g., for MIN_VALUE)
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
                    // For non-decimal bases, we need to implement our own float parsing
                    let s = s.trim();
                    if s.is_empty() {
                        return Err(ParseNumberError::EmptyString);
                    }

                    // Handle sign
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

                    // Split by decimal point
                    let parts: Vec<&str> = s.split('.').collect();
                    if parts.len() > 2 {
                        return Err(ParseNumberError::MultipleDecimalPoints);
                    }

                    // Parse integer part
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

                    // Parse fractional part
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
                    // For decimal or scientific notation, use the standard library parser
                    <$t as FromStr>::from_str(s).map_err(|_| {
                        ParseNumberError::InvalidFormat(format!("failed to parse '{}' as {}", s, stringify!($t)))
                    })
                }
            }
        }
    };
}

// Implement for common integer types
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

// Implement for floating point types
impl_numeric_parser_for_float!(f32, "Implementation for f32");
impl_numeric_parser_for_float!(f64, "Implementation for f64");

/// A generic number parser that can parse to any target type
pub struct NumberParser<T> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: NumericParser> NumberParser<T> {
    /// Creates a new parser for the specified numeric type
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    /// Parse a string into the numeric type
    pub fn parse(&self, s: &str) -> Result<T, ParseNumberError> {
        T::parse(s)
    }

    /// Parse a string with a specific radix
    pub fn parse_radix(&self, s: &str, radix: u8) -> Result<T, ParseNumberError> {
        T::from_str_radix(s, radix)
    }
}

// Helper function to easily create a parser for a specific type
pub fn parser<T: NumericParser>() -> NumberParser<T> {
    NumberParser::new()
}

/// Represents different automatically detected number types
#[derive(Debug, Clone, PartialEq)]
pub enum AutoNumber {
    /// 8-bit unsigned integer
    U8(u8),
    /// 16-bit unsigned integer
    U16(u16),
    /// 32-bit unsigned integer
    U32(u32),
    /// 64-bit unsigned integer
    U64(u64),
    /// 128-bit unsigned integer
    U128(u128),
    /// 8-bit signed integer
    I8(i8),
    /// 16-bit signed integer
    I16(i16),
    /// 32-bit signed integer
    I32(i32),
    /// 64-bit signed integer
    I64(i64),
    /// 128-bit signed integer
    I128(i128),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
}

impl AutoNumber {
    /// Checks if the value is an integer type
    pub fn is_int(&self) -> bool {
        match self {
            Self::U8(_) | Self::U16(_) | Self::U32(_) | Self::U64(_) | Self::U128(_) |
            Self::I8(_) | Self::I16(_) | Self::I32(_) | Self::I64(_) | Self::I128(_) => true,
            _ => false,
        }
    }

    /// Checks if the value is a floating point type
    pub fn is_float(&self) -> bool {
        match self {
            Self::F32(_) | Self::F64(_) => true,
            _ => false,
        }
    }

    /// Attempts to extract the value as an i64, if it's an integer type
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

    /// Attempts to extract the value as an f64, if it's a floating point type or can be converted to one
    pub fn as_float(&self) -> Result<f64, ParseNumberError> {
        match self {
            // Integer types can be converted to float
            Self::U8(v) => Ok(*v as f64),
            Self::U16(v) => Ok(*v as f64),
            Self::U32(v) => Ok(*v as f64),
            Self::U64(v) => Ok(*v as f64),
            Self::U128(v) => {
                // Large u128 values can lose precision when converted to f64
                if *v > 9007199254740992u128 { // 2^53, limit of exact integer representation in f64
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
                // Large i128 values can lose precision when converted to f64
                if *v > 9007199254740992i128 || *v < -9007199254740992i128 {
                    Err(ParseNumberError::Overflow)
                } else {
                    Ok(*v as f64)
                }
            }
            // Float types
            Self::F32(v) => Ok(*v as f64),
            Self::F64(v) => Ok(*v),
        }
    }
}

/// A type-inferring number parser that automatically detects the most appropriate
/// numeric type based on the input string
pub struct AutoNumberParser;

impl AutoNumberParser {
    /// Creates a new auto-detecting parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a string and automatically determine the best numeric type
    pub fn parse(&self, s: &str) -> Result<AutoNumber, ParseNumberError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseNumberError::EmptyString);
        }

        // Check if we have a floating point value
        if s.contains('.') || s.contains(['e', 'E']) {
            // Try f32 first
            if let Ok(value) = parser::<f32>().parse(s) {
                return Ok(AutoNumber::F32(value));
            }

            // If f32 fails or overflows, try f64
            if let Ok(value) = parser::<f64>().parse(s) {
                return Ok(AutoNumber::F64(value));
            }

            return Err(ParseNumberError::InvalidFormat(format!("Could not parse '{}' as a floating point number", s)));
        }

        // Check for sign to determine if we need signed integers
        let is_negative = s.starts_with('-');

        if is_negative {
            // Try signed integers in ascending size order
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
            // Try unsigned integers in ascending size order
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

    /// Parse a string with a specific radix and automatically determine the best numeric type
    pub fn parse_radix(&self, s: &str, radix: u8) -> Result<AutoNumber, ParseNumberError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseNumberError::EmptyString);
        }

        // Check if we have a floating point value
        if s.contains('.') {
            // Try f32 first
            if let Ok(value) = parser::<f32>().parse_radix(s, radix) {
                return Ok(AutoNumber::F32(value));
            }

            // If f32 fails or overflows, try f64
            if let Ok(value) = parser::<f64>().parse_radix(s, radix) {
                return Ok(AutoNumber::F64(value));
            }

            return Err(ParseNumberError::InvalidFormat(format!("Could not parse '{}' as a floating point number", s)));
        }

        // Check for sign to determine if we need signed integers
        let is_negative = s.starts_with('-');

        if is_negative {
            // Try signed integers in ascending size order
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
            // Try unsigned integers in ascending size order
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

/// Helper function to create an auto-detecting parser
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
        assert_eq!(parser.parse_radix("A.B", 16), Ok(10.6875));  // 10 + 11/16
    }

    #[test]
    fn test_auto_parser() {
        let parser = auto_parser();

        // Test unsigned integer auto-detection
        assert!(matches!(parser.parse("42"), Ok(AutoNumber::U8(42))));
        assert!(matches!(parser.parse("256"), Ok(AutoNumber::U16(256))));
        assert!(matches!(parser.parse("65536"), Ok(AutoNumber::U32(_))));

        // Test signed integer auto-detection
        assert!(matches!(parser.parse("-42"), Ok(AutoNumber::I8(-42))));
        assert!(matches!(parser.parse("-129"), Ok(AutoNumber::I16(-129))));
        assert!(matches!(parser.parse("-32769"), Ok(AutoNumber::I32(_))));

        // Test float auto-detection
        assert!(matches!(parser.parse("3.14"), Ok(AutoNumber::F32(_))));
        assert!(matches!(parser.parse("1.23e-2"), Ok(AutoNumber::F32(_))));

        // Test very large float (should be F64)
        assert!(matches!(parser.parse("1.23e38"), Ok(AutoNumber::F64(_))));

        // Test custom radix
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
        let large_value = parser.parse("9223372036854775808").unwrap(); // 2^63, too large for i64

        // Test as_int()
        assert_eq!(int_value.as_int(), Ok(42));
        assert!(matches!(float_value.as_int(), Err(ParseNumberError::TypeError(_))));
        assert!(matches!(large_value.as_int(), Err(ParseNumberError::Overflow)));

        // Test as_float()
        assert_eq!(int_value.as_float(), Ok(42.0));
        assert_eq!(float_value.as_float().unwrap(), 3.14);
        assert!(matches!(parser.parse("9007199254740993").unwrap().as_float(), Ok(_))); // 2^53 + 1, might lose precision
    }
}