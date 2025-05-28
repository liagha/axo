use crate::axo_form::{Action, Form, FormKind, Former, Pattern};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::{Element, ElementKind, ParseError};
use crate::axo_span::Span;
use crate::thread::Arc;
use crate::{Parser, Peekable, PunctuationKind, Token, TokenKind};
use crate::axo_form::delimited::delimited;

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
            Pattern::lazy(primary),
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
        Arc::new(|forms, _span: Span| {
            let sequence = forms[0].unwrap();

            let prefixes = Form::expand_inputs(sequence[0].unwrap());

            let operand = match sequence[1].clone() {
                Form {
                    kind: FormKind::Single(element),
                    ..
                } => element,
                _ => {
                    unreachable!()
                },
            };

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
            unary(),
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
                    Pattern::lazy(move || binary(minimum + 1)),
                ]),
                1,
                None,
            ),
        ]),
        Arc::new(move |forms, _span: Span| {
            let sequence = forms[0].clone().unwrap();
            
            let mut left = sequence[0].unwrap_output().unwrap();
            
            let operations = sequence[1].unwrap();
            
            let mut precedence = 0;

            for operation in operations {
                let (operator, right) = match operation {
                    Form {
                        kind: FormKind::Multiple(items),
                        ..
                    } => {
                        if items.len() != 2 {
                            continue;
                        }
                        
                        let operator = match items[0].clone() {
                            Form {
                                kind: FormKind::Raw(ref token @ Token { kind: TokenKind::Operator(ref operator), .. }),
                                ..
                            } => {
                                if let Some(operator) = operator.precedence() {
                                    if operator >= precedence {
                                        precedence = operator;
                                        
                                        token.clone()
                                    } else { 
                                        continue;
                                    }
                                } else { 
                                    continue;
                                }
                            },
                            _ => continue,
                        };

                        let right = match items[1].clone() {
                            Form {
                                kind: FormKind::Single(element),
                                ..
                            } => element,
                            _ => continue,
                        };

                        (operator, right)
                    },
                    _ => continue,
                };

                let start = left.span.start.clone();
                let end = right.span.end.clone();
                let span = Span::new(start, end);

                left = Element::new(
                    ElementKind::Binary {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    },
                    span,
                );
            }

            Ok(left)
        }),
    )
}

pub fn primary() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([whitespace(), delimited(), token(), fallback()])
}

pub fn expression() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([binary(0), unary(), primary()])
}

pub fn pattern() -> Pattern<Token, Element, ParseError> {
    Pattern::repeat(expression(), 0, None)
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
