use {
    super::{
        Character, ScanError, Scanner, Token, TokenKind,
        error::ErrorKind,
    },
    crate::{
        axo_cursor::{
            Spanned,
        },
        axo_form::{
            form::Form,
            classifier::Classifier,
        },
        parser,
    }
};

impl<'scanner> Scanner<'scanner> {
    pub fn number() -> Classifier<Character, Token, ScanError> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal() -> Classifier<Character, Token, ScanError> {
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
            |_, form| {
                let inputs = form.collect_inputs();
                let number: String = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();

                parser.parse(&number)
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), inputs.span())))
                    .map_err(|err| ScanError::new(ErrorKind::NumberParse(err), inputs.span()))
            },
        )
    }

    fn binary() -> Classifier<Character, Token, ScanError> {
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
            |_, form| {
                let inputs = form.collect_inputs();
                let number: String = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();

                parser.parse(&number)
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), inputs.span())))
                    .map_err(|err| ScanError::new(ErrorKind::NumberParse(err), inputs.span()))
            },
        )
    }

    fn octal() -> Classifier<Character, Token, ScanError> {
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
            |_, form| {
                let inputs = form.collect_inputs();
                let number: String = inputs.clone().into_iter().collect();
                let parser = parser::<i128>();

                parser.parse(&number)
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), inputs.span())))
                    .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), inputs.span()))
            },
        )
    }

    fn decimal() -> Classifier<Character, Token, ScanError> {
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
            |_, form| {
                let inputs = form.collect_inputs();
                let number: String = inputs.clone().into_iter().collect();

                if number.contains('.') || number.to_lowercase().contains('e') {
                    let parser = parser::<f64>();
                    parser.parse(&number)
                        .map(|num| Form::output(Token::new(TokenKind::Float(num.into()), inputs.span())))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), inputs.span()))
                } else {
                    let parser = parser::<i128>();
                    parser.parse(&number)
                        .map(|num| Form::output(Token::new(TokenKind::Integer(num), inputs.span())))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), inputs.span()))
                }
            },
        )
    }
}