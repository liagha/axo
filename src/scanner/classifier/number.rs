use {
    super::super::{Character, ErrorKind, ScanError, Scanner, Token, TokenKind},
    crate::{
        data::{Float, Str},
        formation::{classifier::Classifier, form::Form},
        text::parser,
        tracker::Spanned,
    },
};

impl<'scanner> Scanner<'scanner> {
    pub fn number(
    ) -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal(
    ) -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
            move |classifier| {
                let inputs = classifier.form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();
                let span = inputs.borrow_span().clone();

                match parser.parse(&number) { 
                    Ok(number) => {
                        classifier.form = Form::output(Token::new(TokenKind::Integer(number), span));
                        
                        Ok(())
                    }
                    
                    Err(error) => {
                        Err(ScanError::new(ErrorKind::NumberParse(error), span))
                    }
                }
            },
        )
    }

    fn binary() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();
                let span = inputs.borrow_span().clone();

                match parser.parse(&number) {
                    Ok(number) => {
                        classifier.form = Form::output(Token::new(TokenKind::Integer(number), span));

                        Ok(())
                    }

                    Err(error) => {
                        Err(ScanError::new(ErrorKind::NumberParse(error), span))
                    }
                }
            },
        )
    }

    fn octal() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();
                let span = inputs.borrow_span().clone();

                match parser.parse(&number) {
                    Ok(number) => {
                        classifier.form = Form::output(Token::new(TokenKind::Integer(number), span));

                        Ok(())
                    }

                    Err(error) => {
                        Err(ScanError::new(ErrorKind::NumberParse(error), span))
                    }
                }
            },
        )
    }

    fn decimal() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>
    {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.value == '-').as_optional(),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_numeric()),
                        Classifier::literal('.'),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    0,
                    None,
                ).as_optional(),
                Classifier::optional(
                    Classifier::sequence([
                        Classifier::predicate(|c: &Character| matches!(c.value, 'e' | 'E')),
                        Classifier::optional(
                            Classifier::predicate(|c: &Character| {
                                matches!(c.value, '+' | '-')
                            })
                        ),
                        Classifier::persistence(
                            Classifier::predicate(|c: &Character| c.is_numeric()),
                            1,
                            None,
                        ),
                    ])
                ),
            ]),
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let span = inputs.borrow_span().clone();

                if number.contains(".") || number.to_lowercase().contains('e') {
                    let parser = parser::<f64>();
                    
                    match parser.parse(&number) {
                        Ok(number) => {
                            classifier.form = Form::output(Token::new(TokenKind::Float(Float::from(number)), span));

                            Ok(())
                        }

                        Err(error) => {
                            Err(ScanError::new(ErrorKind::NumberParse(error), span))
                        }
                    }
                } else {
                    let parser = parser::<i128>();
                    
                    match parser.parse(&number) {
                        Ok(number) => {
                            classifier.form = Form::output(Token::new(TokenKind::Integer(number), span));

                            Ok(())
                        }

                        Err(error) => {
                            Err(ScanError::new(ErrorKind::NumberParse(error), span))
                        }
                    }
                }
            },
        )
    }
}
