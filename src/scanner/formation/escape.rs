use {
    crate::{
        data::{
            character::{from_u32, parse_radix},
            Str,
        },
        scanner::{Character, CharacterError, ErrorKind, EscapeError, ScanError, Scanner, Token},
        tracker::Spanned,
    },
    chaint::{Form, Formation},
};

impl<'a> Scanner<'a> {
    pub fn simple_escape<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('\\'),
            Formation::predicate(|c: &Character| match c.value {
                '\\' | '"' | '\'' | 'a' | 'b' | 'e' | 'f' | 'n' | 'r' | 't' | 'v' | '0' => true,
                c if c.is_alphanumeric() => true,
                _ => false,
            }),
        ])
        .with_transform(|joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let span = inputs.span().clone();
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

            *form = Form::Input(Character::new(escaped, span));

            Ok(())
        })
    }

    pub fn octal_escape<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('\\'),
            Formation::persistence(
                Formation::predicate(|c: &Character| c.value.is_digit(8)),
                1,
                Some(3),
            ),
        ])
        .with_transform(|joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter().skip(1).map(|c| c.value).collect();
            let span = inputs.span().clone();

            match parse_radix(digits, 8).map(|parsed| parsed as u32) {
                Some(code_point) => {
                    if code_point > 255 {
                        return Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::OutOfRange),
                            span,
                        ));
                    }

                    match from_u32(code_point) {
                        Some(ch) => {
                            *form = Form::Input(Character::new(ch, span));

                            Ok(())
                        }
                        None => Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::Invalid),
                            span,
                        )),
                    }
                }
                None => Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Overflow),
                    span,
                )),
            }
        })
    }

    pub fn hex_escape<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>>
    {
        Formation::sequence([
            Formation::literal('\\'),
            Formation::alternative([Formation::literal('x'), Formation::literal('X')]),
            Formation::persistence(
                Formation::predicate(|c: &Character| c.value.is_ascii_hexdigit()),
                1,
                Some(2),
            ),
        ])
        .with_transform(|joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter().skip(2).map(|c| c.value).collect();
            let span = inputs.span().clone();

            match parse_radix(digits, 16).map(|parsed| parsed as u32) {
                Some(code_point) => {
                    if code_point > 255 {
                        return Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::OutOfRange),
                            span,
                        ));
                    }

                    match from_u32(code_point) {
                        Some(ch) => {
                            *form = Form::Input(Character::new(ch, span));

                            Ok(())
                        }
                        None => Err(ScanError::new(
                            ErrorKind::InvalidEscape(EscapeError::Invalid),
                            span,
                        )),
                    }
                }
                None => Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Invalid),
                    span,
                )),
            }
        })
    }

    pub fn unicode_escape<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('\\'),
            Formation::alternative([Formation::literal('u'), Formation::literal('U')]),
            Formation::literal('{'),
            Formation::persistence(
                Formation::predicate(|c: &Character| c.value.is_ascii_hexdigit()),
                1,
                Some(6),
            ),
            Formation::literal('}'),
        ])
        .with_transform(|joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let digits: Str = inputs
                .iter()
                .skip(3)
                .take(inputs.len() - 4)
                .map(|c| c.value)
                .collect();
            let span = inputs.span().clone();

            if digits.is_empty() {
                return Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Empty),
                    span,
                ));
            }

            match parse_radix(digits, 16).map(|parsed| parsed as u32) {
                Some(code_point) => match from_u32(code_point) {
                    Some(ch) => {
                        *form = Form::Input(Character::new(ch, span));

                        Ok(())
                    }
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
                },
                None => Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Invalid),
                    span,
                )),
            }
        })
    }

    pub fn unicode_escape_simple<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('\\'),
            Formation::alternative([Formation::literal('u'), Formation::literal('U')]),
            Formation::persistence(
                Formation::predicate(|c: &Character| c.value.is_ascii_hexdigit()),
                4,
                Some(4),
            ),
        ])
        .with_transform(move |joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let digits: Str = inputs.iter().skip(2).map(|c| c.value).collect();
            let span = inputs.span().clone();

            match parse_radix(digits, 16).map(|parsed| parsed as u32) {
                Some(code_point) => match from_u32(code_point) {
                    Some(ch) => {
                        *form = Form::Input(Character::new(ch, span));

                        Ok(())
                    }
                    None => {
                        let err = if (0xD800..=0xDFFF).contains(&code_point) {
                            ErrorKind::InvalidCharacter(CharacterError::Surrogate)
                        } else {
                            ErrorKind::InvalidEscape(EscapeError::Invalid)
                        };
                        Err(ScanError::new(err, span))
                    }
                },
                None => Err(ScanError::new(
                    ErrorKind::InvalidEscape(EscapeError::Invalid),
                    span,
                )),
            }
        })
    }

    pub fn escape_sequence<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::alternative([
            Self::unicode_escape(),
            Self::unicode_escape_simple(),
            Self::hex_escape(),
            Self::octal_escape(),
            Self::simple_escape(),
        ])
    }
}
