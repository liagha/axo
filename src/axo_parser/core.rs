use {
    super::{
        Parser,
        Element, ElementKind, ParseError,

        error::ErrorKind,
    },

    crate::{
        Peekable,

        thread::Arc,

        axo_lexer::{
            Token, TokenKind,
            PunctuationKind,
        },

        axo_form::{
            former::{Form, FormKind, Former},
            pattern::Pattern,
            action::Action,
        },

        axo_span::{Span, Spanned},
    }
};
use crate::axo_parser::ItemKind;


impl Parser {
    pub fn identifier() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::predicate(Arc::new(|token: &Token| {
                matches!(token.kind, TokenKind::Identifier(_))
            })),
            Arc::new(|_, form| {
                let input = form.inputs()[0].clone();

                if let Token {
                    kind: TokenKind::Identifier(identifier),
                    span,
                } = input
                {
                    Ok(Element::new(ElementKind::Identifier(identifier), span))
                } else {
                    unreachable!()
                }
            }),
        )
    }

    pub fn literal() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::predicate(Arc::new(|token: &Token| {
                matches!(
                token.kind,
                TokenKind::String(_)
                    | TokenKind::Character(_)
                    | TokenKind::Boolean(_)
                    | TokenKind::Float(_)
                    | TokenKind::Integer(_)
            )
            })),
            Arc::new(|_, form| {
                form.expand().first()
                    .and_then(|token| match token.kind.clone() {
                        FormKind::Input(Token { kind, span }) => {
                            Some(Element::new(ElementKind::Literal(kind), span))
                        }
                        _ => None,
                    })
                    .ok_or_else(|| unreachable!())
            }),
        )
    }

    pub fn unary() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(Arc::new(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        operator.is_prefix()
                    } else {
                        false
                    }
                }))
                    .repeat_self(0, None)
                    .optional_self(),
                Self::primary(),
                Pattern::predicate(Arc::new(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        operator.is_postfix()
                    } else {
                        false
                    }
                }))
                    .repeat_self(0, None)
                    .optional_self(),
            ]),
            Arc::new(|_, form| {
                let sequence = form.unwrap()[0].clone().unwrap();

                let prefixes = Form::expand_inputs(sequence[0].unwrap());

                let operand = sequence[1].unwrap_output().unwrap();

                let mut unary = operand.clone();

                for prefix in prefixes {
                    unary = Element::new(
                        ElementKind::Unary {
                            operand: unary.into(),
                            operator: prefix,
                        },
                        Span::default(),
                    );
                }

                let postfixes = Form::expand_inputs(sequence[2].unwrap());

                for postfix in postfixes {
                    unary = Element::new(
                        ElementKind::Unary {
                            operand: unary.into(),
                            operator: postfix,
                        },
                        Span::default(),
                    );
                }

                Ok(unary)
            }),
        )
    }

    pub fn binary(minimum: u8) -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Self::unary(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::predicate(Arc::new(move |token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                if let Some(precedence) = operator.precedence() {
                                    precedence >= minimum
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })),
                        Pattern::lazy(move || Self::unary()),
                    ]),
                    0,
                    None,
                ),
            ]),
            Arc::new(move |_, form| {
                let sequence = form.unwrap()[0].clone().unwrap();

                let mut left = sequence[0].unwrap_output().unwrap();
                let operations = sequence[1].unwrap();

                let mut pairs = Vec::new();

                for operation in operations {
                    let sequence = operation.unwrap();

                    if sequence.len() >= 2 {
                        let operator = sequence[0].unwrap_input().unwrap();
                        let operand = sequence[1].unwrap_output().unwrap();

                        let precedence = if let TokenKind::Operator(op) = &operator.kind {
                            op.precedence().unwrap_or(0)
                        } else {
                            0
                        };

                        pairs.push((operator, operand, precedence));
                    }
                }

                left = Self::climb(left, pairs, minimum);

                Ok(left)
            }),
        )
    }

    fn climb(mut left: Element, pairs: Vec<(Token, Element, u8)>, threshold: u8) -> Element {
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

            let start = left.span.start.clone();
            let end = right.span.end.clone();
            let span = Span::new(start, end);

            left = Element::new(
                ElementKind::Binary {
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: Box::new(right),
                },
                span,
            );

            current = lookahead;
        }

        left
    }

    pub fn conditional() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(Arc::new(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "if"
                    } else {
                        false
                    }
                }))
                    .with_ignore(),
                Pattern::required(
                    Pattern::lazy(|| Self::pattern()),
                    Action::Error(Arc::new(|span| {
                        ParseError::new(ErrorKind::ExpectedCondition, span)
                    })),
                ),
                Pattern::required(
                    Pattern::lazy(|| Self::pattern()),
                    Action::Error(Arc::new(|span| {
                        ParseError::new(ErrorKind::ExpectedBody, span)
                    })),
                ),
                Pattern::optional(
                    Pattern::sequence([
                        Pattern::predicate(Arc::new(|token: &Token| {
                            if let TokenKind::Identifier(identifier) = &token.kind {
                                identifier == "else"
                            } else {
                                false
                            }
                        })).with_ignore(),
                        Pattern::lazy(|| Self::pattern()),
                    ])
                )
            ]),
            Arc::new(|_, form| {
                let sequence = form.outputs();

                let condition = sequence[0].clone();

                let then = sequence[1].clone();

                if let Some(alternate) = sequence.get(2).cloned() {
                    let span = condition.span.mix(&alternate.span);

                    Ok(Element::new(
                        ElementKind::Conditional {
                            condition: condition.into(),
                            then: then.into(),
                            alternate: Some(alternate.into()),
                        },
                        span,
                    ))
                } else {
                    let span = condition.span.mix(&then.span);

                    Ok(Element::new(
                        ElementKind::Conditional {
                            condition: condition.into(),
                            then: then.into(),
                            alternate: None,
                        },
                        span,
                    ))
                }

            }),
        )
    }

    pub fn loops() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::alternative([
                Pattern::sequence([
                    Pattern::predicate(Arc::new(|token: &Token| {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == "loop"
                        } else {
                            false
                        }
                    })).with_ignore(),
                    Pattern::required(
                        Pattern::lazy(|| Self::pattern()),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(ErrorKind::ExpectedBody, span)
                        })),
                    ),
                ]),
                Pattern::sequence([
                    Pattern::predicate(Arc::new(|token: &Token| {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == "while"
                        } else {
                            false
                        }
                    })).with_ignore(),
                    Pattern::required(
                        Pattern::lazy(|| Self::pattern()),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(ErrorKind::ExpectedCondition, span)
                        })),
                    ),
                    Pattern::required(
                        Pattern::lazy(|| Self::pattern()),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(ErrorKind::ExpectedBody, span)
                        })),
                    ),
                ])
            ]),
            Arc::new(|_, form| {
                let sequence = form.outputs();

                if sequence.len() == 1 {
                    let body = sequence[0].clone();
                    let span = body.span.clone();

                    Ok(Element::new(
                        ElementKind::Loop {
                            condition: None,
                            body: body.into()
                        },
                        span
                    ))
                } else if sequence.len() == 2 {
                    let condition = sequence[0].clone();
                    let body = sequence[1].clone();

                    let span = condition.span.mix(&body.span);

                    Ok(Element::new(
                        ElementKind::Loop {
                            condition: Some(condition.into()),
                            body: body.into()
                        },
                        span
                    ))
                } else {
                    unreachable!()
                }
            })
        )
    }

    pub fn variable() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(Arc::new(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "var"
                    } else {
                        false
                    }
                })).with_ignore(),
                Pattern::lazy(|| {
                    Pattern::capture(
                        0,
                        Self::pattern()
                    )
                })
            ]),
            Arc::new(move |context, form| {
                let symbols = context.resolver.scope.symbols.clone();
                let formed = symbols.iter().find(|item| matches!(item.kind, ItemKind::Formed { identifier: 0, .. }));

                println!("formed: {:?}", formed);

                Ok(Element::new(
                    ElementKind::Invalid(ParseError::new(ErrorKind::PatternError, Span::default())),
                    Span::default()
                ))
            })
        )
    }

    pub fn statement() -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([Self::conditional(), Self::loops(), Self::variable()])
    }

    pub fn token() -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([Self::identifier(), Self::literal()])
    }

    pub fn whitespace() -> Pattern<Token, Element, ParseError> {
        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Space)
        })))
    }

    pub fn fallback() -> Pattern<Token, Element, ParseError> {
        Pattern::conditional(
            Pattern::predicate(Arc::new(|_token: &Token| {
                true
            })),
            Action::Error(Arc::new(|span| {
                ParseError::new(ErrorKind::PatternError, span)
            })),
            Action::Ignore,
        )
    }

    pub fn primary() -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([Self::delimited(), Self::token(), Self::whitespace()])
    }

    pub fn expression(minimum: u8) -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([Self::binary(minimum), Self::unary(), Self::primary()])
    }

    pub fn pattern() -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([Self::statement(), Self::expression(0)])
    }

    pub fn parser() -> Pattern<Token, Element, ParseError> {
        Pattern::repeat(Pattern::alternative([Self::pattern(), Self::fallback()]), 0, None)
    }
}
