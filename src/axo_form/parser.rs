use crate::arc::Arc;
use crate::axo_form::{Action, Form, FormKind, Former, Pattern};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::{Element, ElementKind, ParseError};
use crate::axo_span::Span;
use crate::{Parser, Peekable, PunctuationKind, Token, TokenKind};

fn expand_elements(forms: Vec<Form<Token, Element, ParseError>>) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();
    
    for form in forms {
        match form.kind { 
            FormKind::Single(element) => {
                elements.push(element);
            },
            FormKind::Multiple(sub) => {
                let sub = expand_elements(sub);
                
                elements.extend(sub);
            }
            _ => {}
        }
    }
    
    elements
}

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
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftParen)
            }))),
            Pattern::repeat(
                Pattern::sequence([
                    token(),
                    Pattern::ignore(Pattern::optional(Pattern::predicate(Arc::new(
                        |token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        },
                    )))),
                ]),
                0,
                None,
            ),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightParen)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftParen),
                            Span::default(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = expand_elements(forms);

            Ok(Element::new(ElementKind::Group(elements), span))
        }),
    )
}

pub fn token() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([identifier(), literal()])
}

pub fn pattern() -> Pattern<Token, Element, ParseError> {
    Pattern::repeat(Pattern::alternative([group(), token()]), 0, None)
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
                            FormKind::Multiple(sub_multi) => {
                                for sub_item in sub_multi {
                                    if let FormKind::Single(element) = sub_item.kind {
                                        elements.push(element);
                                    }
                                }
                            }
                            FormKind::Error(err) => {
                                errors.push(err);
                            }
                            _ => {}
                        }
                    }
                }

                FormKind::Error(err) => {
                    errors.push(err);
                }

                FormKind::Empty | FormKind::Raw(_) => {}
            }
        }

        (elements, errors)
    }
}
