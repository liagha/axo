use crate::thread::Arc;
use crate::axo_form::{Action, Form, FormKind, Former, Pattern};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::{Element, ElementKind, ParseError};
use crate::axo_span::Span;
use crate::{Parser, Peekable, PunctuationKind, Token, TokenKind};


pub fn identifier() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::predicate(Arc::new(|token: &Token| {
            matches!(token.kind, TokenKind::Identifier(_))
        })),
        Arc::new(|form, _| {
            form.first()
                .and_then(|token| match token.kind.clone() {
                    FormKind::Raw(Token {
                                      kind: TokenKind::Identifier(ident),
                                      span,
                                  }) => Some(Element::new(ElementKind::Identifier(ident), span)),
                    _ => None,
                })
                .ok_or_else(|| unreachable!())
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
        Arc::new(|form, _| {
            form.first()
                .and_then(|token| match token.kind.clone() {
                    FormKind::Raw(Token { kind, span }) => {
                        Some(Element::new(ElementKind::Literal(kind), span))
                    }
                    _ => None,
                })
                .ok_or_else(|| unreachable!())
        }),
    )
}

pub fn group() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Group(elements), span))
        }),
    )
}

pub fn sequence() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::SemiColon,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Sequence(elements), span))
        }),
    )
}

pub fn collection() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Collection(elements), span))
        }),
    )
}

pub fn series() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::SemiColon,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Series(elements), span))
        }),
    )
}

pub fn bundle() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Bundle(elements), span))
        }),
    )
}

pub fn scope() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::SemiColon,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Sequence(elements), span))
        }),
    )
}

pub fn token() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([identifier(), literal()])
}

pub fn whitespace() -> Pattern<Token, Element, ParseError> {
    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
        token.kind == TokenKind::Punctuation(PunctuationKind::Space)
    })))
}

pub fn fallback() -> Pattern<Token, Element, ParseError> {
    Pattern::conditional(
        Pattern::anything(),
        Action::Error(Arc::new(|span| {
            ParseError::new(ErrorKind::PatternError, span)
        })),
        Action::Ignore,
    )
}

pub fn delimited() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([
        group(),
        sequence(),
        collection(),
        series(),
        bundle(),
        scope(),
    ])
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
            })).repeat_self(0, None).optional_self(),
            Pattern::lazy(|| expression(0)),
            Pattern::predicate(Arc::new(|token: &Token| {
                if let TokenKind::Operator(operator) = &token.kind {
                    operator.is_postfix()
                } else {
                    false
                }
            })).repeat_self(0, None).optional_self(),
        ]),
        Arc::new(|forms, span: Span| {
            let sequence = forms[0].unwrap();
            
            let prefixes = Form::expand_inputs(sequence[0].unwrap());
            
            let operand = match sequence[1].clone() {
                Form { kind: FormKind::Single(element), .. } => element,
                _ => unreachable!()
            };
            
            let mut unary : Option<Element> = None;
            
            for prefix in prefixes {
                if let Some(operand) = unary.clone() {
                    unary = Some(
                        Element::new(
                            ElementKind::Unary {
                                operand: operand.clone().into(),
                                operator: prefix,
                            },
                            Span::default(),
                        )
                    );
                } else { 
                    unary = Some(Element::new(
                        ElementKind::Unary {
                            operand: operand.clone().into(),
                            operator: prefix,
                        },
                        Span::default(), 
                    ))
                }
            }

            let postfixes = Form::expand_inputs(sequence[2].unwrap());

            for postfix in postfixes {
                if let Some(operand) = unary.clone() {
                    unary = Some(
                        Element::new(
                            ElementKind::Unary {
                                operand: operand.clone().into(),
                                operator: postfix,
                            },
                            Span::default(),
                        )
                    );
                } else {
                    unary = Some(Element::new(
                        ElementKind::Unary {
                            operand: operand.clone().into(),
                            operator: postfix,
                        },
                        Span::default(),
                    ))
                }
            }
            
            Ok(unary.unwrap())
        }),
    )
}

pub fn binary(precedence: usize) -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::lazy(move || expression(precedence + 1)),
            Pattern::predicate(Arc::new(move |token: &Token| {
                if let TokenKind::Operator(operator) = &token.kind {
                    operator.precedence().is_some()
                } else {
                    false
                }
            })),
            Pattern::lazy(move || expression(precedence + 1)),
        ]),
        Arc::new(|forms, span: Span| {
            let sequence = forms[0].clone().unwrap();

            let left = match sequence[0].clone() {
                Form { kind: FormKind::Single(element), .. } => element,
                _ => unreachable!(),
            };

            let operator = match sequence[1].clone() {
                Form { kind: FormKind::Raw(operator @ Token { kind: TokenKind::Operator(_), .. }), .. } => operator,
                _ => unreachable!(),
            };

            let right = match sequence[2].clone() {
                Form { kind: FormKind::Single(element), .. } => element,
                _ => unreachable!(),
            };

            Ok(Element::new(
                ElementKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                span,
            ))
        }),
    )
}

pub fn expression(precedence: usize) -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([
        whitespace(),
        binary(precedence),
        unary(),
        delimited(),
        token(),
        fallback(),
    ])
}

pub fn pattern() -> Pattern<Token, Element, ParseError> {
    Pattern::repeat(expression(0), 0, None)
}

impl Parser {
    pub fn parse_program(&mut self) -> (Vec<Element>, Vec<ParseError>) {
        let mut elements = Vec::new();
        let mut errors = Vec::new();

        while self.peek().is_some() {
            let form = self.form(pattern());

            match form.kind {
                FormKind::Single(element) => {
                    elements.push(element);
                }

                FormKind::Multiple(multi) => {
                    for item in multi {
                        match item.kind {
                            FormKind::Single(element) => {
                                elements.push(element);
                            }
                            FormKind::Multiple(sub) => {
                                for item in sub {
                                    if let FormKind::Single(element) = item.kind {
                                        elements.push(element);
                                    }
                                }
                            }
                            FormKind::Error(error) => {
                                errors.push(error);
                            }
                            _ => {}
                        }
                    }
                }

                FormKind::Error(error) => {
                    errors.push(error);
                }

                FormKind::Empty | FormKind::Raw(_) => {}
            }
        }

        (elements, errors)
    }
}