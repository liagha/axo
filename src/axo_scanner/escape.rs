use {
    crate::{
        axo_form::{
            form::{Form, FormKind},
            pattern::Classifier,
        },
        axo_scanner::{
            Character, ScanError, Token, ErrorKind, Scanner,
            error::{CharacterError, EscapeError},
        },
        character::{parse_radix, from_u32},
    }
};

impl Scanner {
    pub fn simple_escape() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::predicate(|c: &Character| {
                match c.value {
                    '\\' | '"' | '\'' | 'a' | 'b' | 'e' | 'f' | 'n' | 'r' | 't' | 'v' | '0' => true,
                    c if c.is_alphanumeric() => { true }
                    _ => false,
                }
            })
        ]).with_transform(|_, form| {
            let escape = form.inputs()[1];

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
                        form.span,
                    ));
                }
            };

            Ok(Form::new(FormKind::Input(Character::new(escaped, form.span)), form.span))
        })
    }

    pub fn octal_escape() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::literal('\\'),
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.value.is_digit(8)),
                1,
                Some(3),
            ),
        ]).with_transform(|_, form| {
            let digits: String = form.inputs().iter().skip(1).map(|c| c.value).collect();

            match parse_radix(&digits, 8) {
                Some(code_point) => {
                    if code_point > 255 {
                        return Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::OutOfRange),
                            form.span,
                        ));
                    }

                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::new(
                            FormKind::Input(Character::new(ch, form.span)),
                            form.span,
                        )),
                        None => {
                            Err(ScanError::new(
                                ErrorKind::InvalidEscape(EscapeError::Invalid),
                                form.span,
                            ))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Overflow),
                        form.span,
                    ))
                }
            }
        })
    }

    pub fn hex_escape() -> Classifier<Character, Token, ScanError> {
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
        ]).with_transform(|_, form| {
            let hex_digits: String = form.inputs().iter().skip(2).map(|c| c.value).collect();

            match parse_radix(&hex_digits, 16) {
                Some(code_point) => {
                    if code_point > 255 {
                        return Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::OutOfRange),
                            form.span,
                        ));
                    }

                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::new(
                            FormKind::Input(Character::new(ch, form.span)),
                            form.span
                        )),
                        None => {
                            Err(ScanError::new(
                                ErrorKind::InvalidEscape(EscapeError::Invalid),
                                form.span,
                            ))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        form.span,
                    ))
                }
            }
        })
    }

    pub fn unicode_escape() -> Classifier<Character, Token, ScanError> {
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
        ]).with_transform(|_, form| {
            let inputs = form.inputs();
            let hex_digits: String = inputs.iter()
                .skip(3)
                .take(inputs.len() - 4)
                .map(|c| c.value)
                .collect();

            if hex_digits.is_empty() {
                return Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Empty),
                    form.span,
                ));
            }

            match parse_radix(&hex_digits, 16) {
                Some(code_point) => {
                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::new(
                            FormKind::Input(Character::new(ch, form.span)),
                            form.span
                        )),
                        None => {
                            let err = if code_point > 0x10FFFF {
                                ErrorKind::InvalidCharacter(CharacterError::OutOfRange)
                            } else if (0xD800..=0xDFFF).contains(&code_point) {
                                ErrorKind::InvalidCharacter(CharacterError::Surrogate)
                            } else {
                                ErrorKind::InvalidEscape(EscapeError::Invalid)
                            };
                            Err(ScanError::new(err, form.span))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        form.span,
                    ))
                }
            }
        })
    }

    pub fn unicode_escape_simple() -> Classifier<Character, Token, ScanError> {
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
        ]).with_transform(|_, form| {
            let hex_digits: String = form.inputs().iter().skip(2).map(|c| c.value).collect();

            match parse_radix(&hex_digits, 16) {
                Some(code_point) => {
                    match from_u32(code_point) {
                        Some(ch) => Ok(Form::new(
                            FormKind::Input(Character::new(ch, form.span)),
                            form.span
                        )),
                        None => {
                            let err = if (0xD800..=0xDFFF).contains(&code_point) {
                                ErrorKind::InvalidCharacter(CharacterError::Surrogate)
                            } else {
                                ErrorKind::InvalidEscape(EscapeError::Invalid)
                            };
                            Err(ScanError::new(err, form.span))
                        }
                    }
                }
                None => {
                    Err(ScanError::new(
                        ErrorKind::InvalidEscape(EscapeError::Invalid),
                        form.span,
                    ))
                }
            }
        })
    }

    pub fn escape_sequence() -> Classifier<Character, Token, ScanError> {
        Classifier::alternative([
            Self::unicode_escape(),
            Self::unicode_escape_simple(),
            Self::hex_escape(),
            Self::octal_escape(),
            Self::simple_escape(),
        ])
    }
}