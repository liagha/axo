use crate::{
    combinator::{Classifier, Form},
    data::{Float, Integer, Str},
    scanner::{Character, ErrorKind, ScanError, Scanner, Token, TokenKind},
    text::parser,
    tracker::Spanned,
};

impl<'a> Scanner<'a> {
    pub fn number<'source>(
    ) -> Classifier<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal<'source>(
    ) -> Classifier<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('x'), Classifier::literal('X')]),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_alphanumeric()),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            move |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
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

    fn binary<'source>() -> Classifier<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('b'), Classifier::literal('B')]),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| matches!(c.value, '0' | '1')),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
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

    fn octal<'source>() -> Classifier<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('o'), Classifier::literal('O')]),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| ('0'..='7').contains(&c.value)),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
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

    fn decimal<'source>() -> Classifier<'a, 'source, Self, Character, Token<'a>, ScanError<'a>>
    {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.value == '-').into_optional(),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_numeric()),
                        Classifier::literal('.'),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    0,
                    None,
                )
                    .into_optional(),
                Classifier::optional(Classifier::sequence([
                    Classifier::predicate(|c: &Character| matches!(c.value, 'e' | 'E')),
                    Classifier::optional(Classifier::predicate(|c: &Character| {
                        matches!(c.value, '+' | '-')
                    })),
                    Classifier::persistence(
                        Classifier::predicate(|c: &Character| c.is_numeric()),
                        1,
                        None,
                    ),
                ])),
            ]),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
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
