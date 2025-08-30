use {
    super::{
        Character, ScanError, Scanner, Token, TokenKind, ErrorKind,
    },
    crate::{
        data::{Str, Float},
        formation::{form::Form, classifier::Classifier},
        text::parser,
        tracker::Spanned,
    }
};

impl<'scanner> Scanner<'scanner> {
    pub fn number() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
            move |form| {
                let inputs = form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();
                let span = inputs.borrow_span().clone();

                parser.parse(&number)
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), span)))
                    .map_err(move |err| ScanError::new(ErrorKind::NumberParse(err), span))
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
            |form| {
                let inputs = form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();
                let span = inputs.borrow_span().clone();

                parser.parse(&number)
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), span)))
                    .map_err(|err| ScanError::new(ErrorKind::NumberParse(err), span))
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
            |form| {
                let inputs = form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();
                let span = inputs.borrow_span().clone();

                parser.parse(&number)
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), span)))
                    .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), span))
            },
        )
    }

    fn decimal() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_numeric()),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_numeric()),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    0,
                    None,
                ),
                Classifier::optional(Classifier::sequence([
                    Classifier::literal('.'),
                    Classifier::persistence(
                        Classifier::alternative([
                            Classifier::predicate(|c: &Character| c.is_numeric()),
                            Classifier::literal('_').with_ignore(),
                        ]),
                        1,
                        None,
                    ),
                ])),
                Classifier::optional(Classifier::sequence([
                    Classifier::predicate(|c: &Character| matches!(c.value, 'e' | 'E')),
                    Classifier::optional(Classifier::predicate(|c: &Character| matches!(c.value, '+' | '-'))),
                    Classifier::persistence(Classifier::predicate(|c: &Character| c.is_numeric()), 1, None),
                ])),
            ]),
            |form| {
                let inputs = form.collect_inputs();
                let number: Str = inputs.clone().into_iter().collect();
                let span = inputs.borrow_span().clone();

                if number.contains(".") || number.to_lowercase().contains('e') {
                    let parser = parser::<f64>();
                    parser.parse(&number)
                        .map(|num| Form::output(Token::new(TokenKind::Float(Float::from(num)), span)))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), span))
                } else {
                    let parser = parser::<i128>();
                    parser.parse(&number)
                        .map(|num| Form::output(Token::new(TokenKind::Integer(num), span)))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), span))
                }
            },
        )
    }
}