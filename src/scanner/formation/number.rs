use crate::{
    combinator::{Formation, Form},
    data::{Float, Integer, Str},
    scanner::{Character, ErrorKind, ScanError, Scanner, Token, TokenKind},
    text::parser,
    tracker::Spanned,
};

impl<'a> Scanner<'a> {
    pub fn number<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal<'source>(
    ) -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
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
            move |former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let parser = parser::<Integer>();
                let number: Str = inputs.into_iter().collect();

                match parser.parse(&number) {
                    Ok(number) => {
                        *form = Form::output(Token::new(TokenKind::integer(number), span));

                        Ok(())
                    }

                    Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error), span)),
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
            |former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let parser = parser::<Integer>();
                let number: Str = inputs.into_iter().collect();

                match parser.parse(&number) {
                    Ok(number) => {
                        *form = Form::output(Token::new(TokenKind::integer(number), span));

                        Ok(())
                    }

                    Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error), span)),
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
            |former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let parser = parser::<Integer>();
                let number: Str = inputs.into_iter().collect();

                match parser.parse(&number) {
                    Ok(number) => {
                        *form = Form::output(Token::new(TokenKind::integer(number), span));

                        Ok(())
                    }

                    Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error), span)),
                }
            },
        )
    }

    fn decimal<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>>
    {
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
            |former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let number: Str = inputs.into_iter().collect();

                if number.contains(".") || number.to_lowercase().contains('e') {
                    let parser = parser::<f64>();

                    match parser.parse(&number) {
                        Ok(number) => {
                            *form = Form::output(Token::new(
                                TokenKind::float(Float::from(number)),
                                span,
                            ));

                            Ok(())
                        }

                        Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error), span)),
                    }
                } else {
                    let parser = parser::<Integer>();

                    match parser.parse(&number) {
                        Ok(number) => {
                            *form = Form::output(Token::new(TokenKind::integer(number), span));

                            Ok(())
                        }

                        Err(error) => Err(ScanError::new(ErrorKind::NumberParse(error), span)),
                    }
                }
            },
        )
    }
}
