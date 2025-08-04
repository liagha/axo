use {
    super::{
        error::ErrorKind,
        Element, ElementKind,
        ParseError, Parser
    },
    crate::{
        axo_cursor::{
            Span, Spanned,
        },
        axo_form::{
            form::Form,
            former::Former,
            classifier::Classifier,
        },
        axo_scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        axo_schema::{
            Access, Assign,
            Binary, Binding, Conditional,
            Construct, Enumeration,
            Index, Invoke,
            Label,
            Repeat, Structure, Unary,
        },
    },
};

impl<'parser> Parser<'parser> {
    pub fn identifier() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::with_transform(
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    !["loop", "if", "while", "var", "const", "struct", "enum", "func", "impl"].contains(&identifier.as_str())
                } else {
                    false
                }
            }),
            |_, form| {
                let input = form.collect_inputs()[0].clone();
                let identifier = input.kind.unwrap_identifier();

                Ok(Form::output(
                    Element::new(ElementKind::identifier(identifier), input.span)
                ))
            },
        )
    }

    pub fn literal() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::with_transform(
            Classifier::predicate(|token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::String(_)
                        | TokenKind::Character(_)
                        | TokenKind::Boolean(_)
                        | TokenKind::Float(_)
                        | TokenKind::Integer(_)
                )
            }),
            |_, form| {
                let input = form.collect_inputs()[0].clone();

                Ok(Form::output(
                    Element::new(ElementKind::literal(input.kind), input.span)
                ))
            },
        )
    }

    pub fn token() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::alternative([Self::identifier(), Self::literal()])
    }

    pub fn whitespace() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
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

    pub fn primary() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::alternative([Self::delimited(), Self::token()])
    }

    pub fn prefixed() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
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
            |_, form: Form<Token, Element, ParseError>| {
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

    pub fn suffixed() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
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
            |_, form| {
                let sequence = form.as_forms();
                let operand = sequence[0].unwrap_output();
                let suffixes = sequence[1].as_forms();
                let mut unary = operand.clone();

                for suffix in suffixes {
                    if let Some(token) = suffix.get_input() {
                        let span = Span::merge(&unary.borrow_span(), &token.borrow_span());

                        unary = Element::new(
                            ElementKind::Unary(Unary::new(token, unary.into())),
                            span,
                        );
                    } else if let Some(element) = suffix.get_output() {
                        let span = Span::merge(&unary.borrow_span(), &element.borrow_span());

                        match element.kind {
                            ElementKind::Group(group) => {
                                unary = Element::new(
                                    ElementKind::invoke(Invoke::new(Box::new(unary), group.items)),
                                    span,
                                )
                            }
                            ElementKind::Collection(collection) => {
                                unary = Element::new(
                                    ElementKind::index(Index::new(Box::new(unary), collection.items)),
                                    span,
                                )
                            }
                            ElementKind::Bundle(bundle) => {
                                unary = Element::new(
                                    ElementKind::construct(Construct::new(Box::new(unary), bundle.items)),
                                    span,
                                )
                            }
                            _ => {}
                        }
                    }
                }

                Ok(Form::output(unary))
            },
        )
    }


    pub fn unary() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::alternative([
            Self::prefixed(),
            Self::suffixed(),
            Self::primary(),
        ])
    }

    pub fn binary() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
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
                move |_, form| {
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

    fn climb(mut left: Element, pairs: Vec<(Token, Element, u8)>, threshold: u8) -> Element<'parser> {
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

            match &operator.kind {
                TokenKind::Operator(OperatorKind::Dot) => {
                    left = Element::new(
                        ElementKind::Access(Access::new(Box::new(left), Box::new(right))),
                        span,
                    );
                }
                TokenKind::Operator(OperatorKind::Equal) => {
                    left = Element::new(
                        ElementKind::Assign(Assign::new(Box::new(left), Box::new(right))),
                        span,
                    );
                }
                TokenKind::Operator(OperatorKind::Colon) => {
                    left = Element::new(
                        ElementKind::Label(Label::new(Box::new(left), Box::new(right))),
                        span,
                    );
                }
                _ => {
                    left = Element::new(
                        ElementKind::Binary(Binary::new(Box::new(left), operator.clone(), Box::new(right))),
                        span,
                    );
                }
            }

            current = lookahead;
        }

        left
    }

    pub fn expression() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::alternative([Self::binary(), Self::unary(), Self::primary()])
    }

    pub fn element() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::alternative([
            Self::statement(),
            Self::expression()
        ])
    }

    pub fn fallback() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::with_fail(
            Classifier::anything(),
            |_, form: Form<Token, Element, ParseError>| {
                let token = form.unwrap_input();

                ParseError::new(
                    ErrorKind::UnexpectedToken(form.unwrap_input().clone().kind),
                    token.span,
                )
            },
        )
    }

    pub fn parser() -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
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