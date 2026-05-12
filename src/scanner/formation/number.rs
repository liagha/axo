use {
    crate::{
        data::{Float, Str},
        scanner::{Character, ErrorKind, ScanError, Scanner, Token, TokenKind},
        tracker::Spanned,
    },
    chaint::{Form, Formation},
};

impl<'a> Scanner<'a> {
    pub fn number<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::sequence([
                Formation::literal('0'),
                Formation::alternative([Formation::literal('x'), Formation::literal('X')]),
                Formation::persistence(
                    Formation::alternative([
                        Formation::predicate(|c: &Character| c.is_alphanumeric()),
                        Formation::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            move |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let number: Str = inputs.into_iter().collect();

                match number.parse() {
                    Ok(number) => {
                        *form = Form::output(Token::new(TokenKind::integer(number), span));

                        Ok(())
                    }

                    Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error.into()), span)),
                }
            },
        )
    }

    fn binary<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::sequence([
                Formation::literal('0'),
                Formation::alternative([Formation::literal('b'), Formation::literal('B')]),
                Formation::persistence(
                    Formation::alternative([
                        Formation::predicate(|c: &Character| matches!(c.value, '0' | '1')),
                        Formation::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let number: Str = inputs.into_iter().collect();

                match number.parse() {
                    Ok(number) => {
                        *form = Form::output(Token::new(TokenKind::integer(number), span));

                        Ok(())
                    }

                    Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error.into()), span)),
                }
            },
        )
    }

    fn octal<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::sequence([
                Formation::literal('0'),
                Formation::alternative([Formation::literal('o'), Formation::literal('O')]),
                Formation::persistence(
                    Formation::alternative([
                        Formation::predicate(|c: &Character| ('0'..='7').contains(&c.value)),
                        Formation::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let number: Str = inputs.into_iter().collect();

                match number.parse() {
                    Ok(number) => {
                        *form = Form::output(Token::new(TokenKind::integer(number), span));

                        Ok(())
                    }

                    Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error.into()), span)),
                }
            },
        )
    }

    fn decimal<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::sequence([
                Formation::persistence(
                    Formation::alternative([
                        Formation::predicate(|c: &Character| c.is_numeric()),
                        Formation::literal('.'),
                        Formation::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
                Formation::optional(Formation::sequence([
                    Formation::predicate(|c: &Character| matches!(c.value, 'e' | 'E')),
                    Formation::optional(Formation::predicate(|c: &Character| {
                        matches!(c.value, '+' | '-')
                    })),
                    Formation::persistence(
                        Formation::predicate(|c: &Character| c.is_numeric()),
                        1,
                        None,
                    ),
                ])),
            ]),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let number: Str = inputs.into_iter().collect();

                if number.contains(".") || number.to_lowercase().contains('e') {
                    match number.parse::<f64>() {
                        Ok(number) => {
                            *form = Form::output(Token::new(
                                TokenKind::float(Float::from(number)),
                                span,
                            ));

                            Ok(())
                        }

                        Err(error) => {
                            Err(ScanError::new(ErrorKind::NumberParse(error.into()), span))
                        }
                    }
                } else {
                    match number.parse() {
                        Ok(number) => {
                            *form = Form::output(Token::new(TokenKind::integer(number), span));

                            Ok(())
                        }

                        Err(error) => {
                            Err(ScanError::new(ErrorKind::NumberParse(error.into()), span))
                        }
                    }
                }
            },
        )
    }
}
