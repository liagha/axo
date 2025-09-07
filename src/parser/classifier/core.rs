use {
    crate::{
        formation::{
            form::Form,
            classifier::Classifier,
        },
        data::Str,
        scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        schema::{
            Binary, Structure,
            Index, Invoke, Unary,
        },
        parser::{
            ErrorKind,
            Element, ElementKind,
            Parser, ParseError,
        },
        tracker::{
            Span, Spanned,
            Location,
        },
    },
};

impl<'parser> Parser<'parser> {
    pub fn get_body(element: Element<'parser>) -> Vec<Element<'parser>> {
        match element.kind {
            ElementKind::Delimited(delimited) => {
                delimited.items
            }
            _ => {
                vec![element]
            }
        }
    }

    pub fn literal() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::predicate(|token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::String(_)
                        | TokenKind::Character(_)
                        | TokenKind::Boolean(_)
                        | TokenKind::Float(_)
                        | TokenKind::Integer(_)
                ) || if let TokenKind::Identifier(identifier) = &token.kind {
                    !["loop", "if", "while", "var", "const", "struct", "enum", "func", "impl", "module"].contains(&identifier.unwrap_str())
                } else {
                    false
                }
            }),
            |form| {
                let input = form.collect_inputs()[0].clone();

                Ok(Form::output(
                    Element::new(ElementKind::literal(input.clone()), input.span)
                ))
            },
        )
    }

    pub fn whitespace() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([Classifier::predicate(
            |token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::Comment(_)
                        | TokenKind::Punctuation(PunctuationKind::Newline)
                        | TokenKind::Punctuation(PunctuationKind::Tab)
                        | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                        | TokenKind::Punctuation(PunctuationKind::Space)
                )
            },
        )])
    }

    pub fn primary() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([Self::delimited(), Self::literal()])
    }

    pub fn prefixed() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Operator(operator) = &token.kind {
                    operator.is_prefix()
                } else {
                    false
                }
            }),
            Self::primary(),
        ]).with_transform(
            |form: Form<Token, Element, ParseError>| {
                let prefixes = form.collect_inputs();
                let operand = form.collect_outputs()[0].clone();
                let mut unary = operand.clone();

                for prefix in prefixes {
                    let span = Span::merge(&prefix.borrow_span(), &unary.borrow_span());

                    unary = Element::new(
                        ElementKind::unary(Unary::new(
                            prefix,
                            Box::new(unary),
                        )),
                        span,
                    );
                }

                Ok(Form::output(unary))
            }
        )
    }

    pub fn suffixed() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Self::primary(),
                Classifier::repetition(
                    Classifier::alternative([
                        Self::group(Classifier::deferred(Self::element)),
                        Self::collection(Classifier::deferred(Self::element)),
                        Self::bundle(Classifier::deferred(Self::element)),
                        Classifier::predicate(|token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                operator.is_suffix()
                            } else {
                                false
                            }
                        })
                    ]),
                    0,
                    None
                ),
            ]),
            |form| {
                let sequence = form.as_forms();
                let operand = sequence[0].unwrap_output();
                let suffixes = sequence[1].as_forms();
                let mut unary = operand.clone();

                for suffix in suffixes {
                    if let Some(token) = suffix.get_input() {
                        let span = Span::merge(&unary.borrow_span(), &token.borrow_span());

                        unary = Element::new(
                            ElementKind::Unary(Unary::new(token, Box::new(unary))),
                            span,
                        );
                    } else if let Some(element) = suffix.get_output() {
                        let span = Span::merge(&unary.borrow_span(), &element.borrow_span());

                        match element.kind {
                            ElementKind::Delimited(delimited) => {
                                match (delimited.start.kind, delimited.separator.map(|token| token.kind), delimited.end.kind) {
                                    (
                                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                                        None,
                                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                                    ) | (
                                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                                    ) => {
                                        unary = Element::new(
                                            ElementKind::invoke(Invoke::new(Box::new(unary), delimited.items)),
                                            span,
                                        )
                                    }

                                    (
                                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                        None,
                                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                                    ) | (
                                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                                    ) => {
                                        unary = Element::new(
                                            ElementKind::index(Index::new(Box::new(unary), delimited.items)),
                                            span,
                                        )
                                    }

                                    (
                                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                                        None,
                                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                                    ) | (
                                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                                    ) => {
                                        unary = Element::new(
                                            ElementKind::construct(Structure::new(Box::new(unary), delimited.items)),
                                            span,
                                        )
                                    }

                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }

                Ok(Form::output(unary))
            },
        )
    }


    pub fn unary() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::prefixed(),
            Self::suffixed(),
            Self::primary(),
        ])
    }

    pub fn binary() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Classifier::with_transform(
                Classifier::sequence([
                    Classifier::alternative([
                        Self::statement(),
                        Self::unary(),
                    ]),
                    Classifier::repetition(
                        Classifier::sequence([
                            Classifier::predicate(move |token: &Token| {
                                if let TokenKind::Operator(operator) = &token.kind {
                                    if let Some(_) = operator.precedence() {
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }),
                            Classifier::alternative([
                                Self::statement(),
                                Self::unary(),
                            ])
                        ]),
                        1,
                        None,
                    ),
                ]),
                move |form| {
                    let sequence = form.as_forms();
                    let mut left = sequence[0].unwrap_output().clone();
                    let operations = sequence[1].as_forms();
                    let mut pairs = Vec::new();

                    for operation in operations {
                        let sequence = operation.as_forms();
                        if sequence.len() >= 2 {
                            let operator = sequence[0].unwrap_input().clone();
                            let operand = sequence[1].unwrap_output().clone();
                            let precedence = if let TokenKind::Operator(op) = &operator.kind {
                                op.precedence().unwrap_or(0)
                            } else {
                                0
                            };
                            pairs.push((operator, operand, precedence));
                        }
                    }

                    left = Self::climb(left, pairs, 0);

                    Ok(Form::output(left))
                },
            )
        ])
    }

    fn climb(mut left: Element<'parser>, pairs: Vec<(Token<'parser>, Element<'parser>, u8)>, threshold: u8) -> Element<'parser> {
        let mut current = 0;

        while current < pairs.len() {
            let (operator, operand, precedence) = &pairs[current];

            if *precedence < threshold {
                break;
            }

            let mut right = operand.clone();
            let mut lookahead = current + 1;

            while lookahead < pairs.len() {
                let (_, _, priority) = &pairs[lookahead];

                if *priority > *precedence {
                    let mut higher = Vec::new();
                    while lookahead < pairs.len() && pairs[lookahead].2 > *precedence {
                        higher.push(pairs[lookahead].clone());
                        lookahead += 1;
                    }

                    right = Self::climb(right, higher, *precedence + 1);
                    break;
                } else {
                    break;
                }
            }

            let start = left.borrow_span().start.clone();
            let end = right.borrow_span().end.clone();
            let span = Span::new(start, end);

            left = Element::new(
                ElementKind::Binary(Binary::new(Box::new(left), operator.clone(), Box::new(right))),
                span,
            );

            current = lookahead;
        }

        left
    }

    pub fn expression() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([Self::binary(), Self::unary(), Self::primary()])
    }

    pub fn element() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::statement(),
            Self::expression()
        ])
    }

    pub fn fallback() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_fail(
            Classifier::anything(),
            |form: Form<Token, Element, ParseError>| {
                let token = form.unwrap_input();

                ParseError::new(
                    ErrorKind::UnexpectedToken(form.unwrap_input().clone().kind),
                    token.span,
                )
            },
        )
    }

    pub fn parser() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::repetition(
            Classifier::alternative([
                Self::symbolization(),
                Self::element(),
                Self::fallback()
            ]),
            0,
            None,
        )
    }
}