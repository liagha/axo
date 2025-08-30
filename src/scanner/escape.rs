use {
    super::{
        Character, ScanError, Token, ErrorKind, Scanner, CharacterError, EscapeError,
    },
    crate::{
        data::{
            character::{parse_radix, from_u32},
            Str,
        },
        formation::{
            classifier::Classifier,
            form::Form,
        },
        tracker::Spanned,
    }
};

impl<'scanner> Scanner<'scanner> {
    pub fn simple_escape() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::predicate(|c: &Character| {
                match c.value {
                    '\\' | '"' | '\'' | 'a' | 'b' | 'e' | 'f' | 'n' | 'r' | 't' | 'v' | '0' => true,
                    c if c.is_alphanumeric() => { true }
                    _ => false,
                }
            })
        ]).with_transform(|form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let span = inputs.borrow_span().clone();
            let escape = inputs[1];

            let escaped = match escape.value {
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                'a' => '\x07',
                'b' => '\x08',
                'e' => '\x1B',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\x0B',
                '0' => '\0',
                _ => {
                    return Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        span,
                    ));
                }
            };

            Ok(Form::Input(Character::new(escaped, span)))
        })
    }

    pub fn octal_escape() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.value.is_digit(8)),
                1,
                Some(3),
            ),
        ]).with_transform(|form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter().skip(1).map(|c| c.value).collect();
            let span = inputs.borrow_span().clone();

            match parse_radix(digits, 8).map(|parsed| parsed as u32) {
                Some(code_point) => {
                    if code_point > 255 {
                        return Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::OutOfRange),
                            span,
                        ));
                    }

                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::Input(Character::new(ch, span))),
                        None => {
                            Err(ScanError::new(
                                ErrorKind::InvalidEscape(EscapeError::Invalid),
                                span,
                            ))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Overflow),
                        span,
                    ))
                }
            }
        })
    }

    pub fn hex_escape() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::alternative([
                Classifier::literal('x'),
                Classifier::literal('X'),
            ]),
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.value.is_ascii_hexdigit()),
                1,
                Some(2),
            ),
        ]).with_transform(|form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter().skip(2).map(|c| c.value).collect();
            let span = inputs.borrow_span().clone();

            match parse_radix(digits, 16).map(|parsed| parsed as u32) {
                Some(code_point) => {
                    if code_point > 255 {
                        return Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::OutOfRange),
                            span,
                        ));
                    }

                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::Input(Character::new(ch, span))),
                        None => {
                            Err(ScanError::new(
                                ErrorKind::InvalidEscape(EscapeError::Invalid),
                                span,
                            ))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        span,
                    ))
                }
            }
        })
    }

    pub fn unicode_escape() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::alternative([
                Classifier::literal('u'),
                Classifier::literal('U'),
            ]),
            Classifier::literal('{'),
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.value.is_ascii_hexdigit()),
                1,
                Some(6),
            ),
            Classifier::literal('}'),
        ]).with_transform(|form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter()
                .skip(3)
                .take(inputs.len() - 4)
                .map(|c| c.value)
                .collect();
            let span = inputs.borrow_span().clone();

            if digits.is_empty() {
                return Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Empty),
                    span,
                ));
            }

            match parse_radix(digits, 16).map(|parsed| parsed as u32) {
                Some(code_point) => {
                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::Input(Character::new(ch, span))),
                        None => {
                            let err = if code_point > 0x10FFFF {
                                ErrorKind::InvalidCharacter(CharacterError::OutOfRange)
                            } else if (0xD800..=0xDFFF).contains(&code_point) {
                                ErrorKind::InvalidCharacter(CharacterError::Surrogate)
                            } else {
                                ErrorKind::InvalidEscape(EscapeError::Invalid)
                            };
                            Err(ScanError::new(err, span))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        span,
                    ))
                }
            }
        })
    }

    pub fn unicode_escape_simple() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::alternative([
                Classifier::literal('u'),
                Classifier::literal('U'),
            ]),
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.value.is_ascii_hexdigit()),
                4,
                Some(4),
            ),
        ]).with_transform(move |form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter().skip(2).map(|c| c.value).collect();
            let span = inputs.span().clone();

            match parse_radix(digits, 16).map(|parsed| parsed as u32) {
                Some(code_point) => {
                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::Input(Character::new(ch, span))),
                        None => {
                            let err = if (0xD800..=0xDFFF).contains(&code_point) {
                                ErrorKind::InvalidCharacter(CharacterError::Surrogate)
                            } else {
                                ErrorKind::InvalidEscape(EscapeError::Invalid)
                            };
                            Err(ScanError::new(err, span))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        span,
                    ))
                }
            }
        })
    }

    pub fn escape_sequence() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::alternative([
            Self::unicode_escape(),
            Self::unicode_escape_simple(),
            Self::hex_escape(),
            Self::octal_escape(),
            Self::simple_escape(),
        ])
    }
}