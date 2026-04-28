use crate::{
    combinator::{Form, Formation},
    data::*,
    parser::{Element, ElementKind, ErrorKind, ParseError, Parser},
    scanner::{PunctuationKind, Token, TokenKind},
    tracker::{Span, Spanned},
};

impl<'a> Parser<'a> {
    pub fn bundle<'source>(
        item: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Formation::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
        })
        .into_optional();

        Formation::sequence([
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }),
            item.clone().into_optional(),
            Formation::persistence(
                Formation::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
            })
            .with_panic(|former, formation| {
                let consumed: Vec<Token> = formation
                    .consumed
                    .iter()
                    .filter_map(|index| former.consumed.get(*index).cloned())
                    .collect();

                let span = if consumed.is_empty() {
                    Span::point(formation.state)
                } else {
                    consumed.span()
                };

                ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    span,
                )
            }),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    Span::point(formation.state),
                ));
            };
            let Some(end) = delimiters.last() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    start.span,
                ));
            };
            let span = Span::merge(&start.span, &end.span);

            let separator_token = if delimiters.len() > 2 {
                Some(delimiters[1].clone())
            } else {
                None
            };

            let kind = ElementKind::delimited(Delimited::new(
                start.clone(),
                elements,
                separator_token,
                end.clone(),
            ));

            *form = Form::output(Element::new(kind, span));

            Ok(())
        })
    }

    pub fn block<'source>(
        item: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Formation::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
        })
        .into_optional();

        Formation::sequence([
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }),
            item.clone().into_optional(),
            Formation::persistence(
                Formation::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
            })
            .with_panic(|former, formation| {
                let consumed: Vec<Token> = formation
                    .consumed
                    .iter()
                    .filter_map(|index| former.consumed.get(*index).cloned())
                    .collect();

                let span = if consumed.is_empty() {
                    Span::point(formation.state)
                } else {
                    consumed.span()
                };

                ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    span,
                )
            }),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    Span::point(formation.state),
                ));
            };
            let Some(end) = delimiters.last() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    start.span,
                ));
            };
            let span = Span::merge(&start.span, &end.span);

            let separator_token = if delimiters.len() > 2 {
                Some(delimiters[1].clone())
            } else {
                None
            };

            let kind = ElementKind::delimited(Delimited::new(
                start.clone(),
                elements,
                separator_token,
                end.clone(),
            ));

            *form = Form::output(Element::new(kind, span));

            Ok(())
        })
    }

    pub fn group<'source>(
        item: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Formation::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
        })
        .into_optional();

        Formation::sequence([
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }),
            item.clone().into_optional(),
            Formation::persistence(
                Formation::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
            })
            .with_panic(|former, formation| {
                let consumed: Vec<Token> = formation
                    .consumed
                    .iter()
                    .filter_map(|index| former.consumed.get(*index).cloned())
                    .collect();

                let span = if consumed.is_empty() {
                    Span::point(formation.state)
                } else {
                    consumed.span()
                };

                ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    span,
                )
            }),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    Span::point(formation.state),
                ));
            };
            let Some(end) = delimiters.last() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    start.span,
                ));
            };
            let span = Span::merge(&start.span, &end.span);

            let separator_token = if delimiters.len() > 2 {
                Some(delimiters[1].clone())
            } else {
                None
            };

            let kind = ElementKind::delimited(Delimited::new(
                start.clone(),
                elements,
                separator_token,
                end.clone(),
            ));

            *form = Form::output(Element::new(kind, span));

            Ok(())
        })
    }

    pub fn sequence<'source>(
        item: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Formation::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
        })
        .into_optional();

        Formation::sequence([
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }),
            item.clone().into_optional(),
            Formation::persistence(
                Formation::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
            })
            .with_panic(|former, formation| {
                let consumed: Vec<Token> = formation
                    .consumed
                    .iter()
                    .filter_map(|index| former.consumed.get(*index).cloned())
                    .collect();

                let span = if consumed.is_empty() {
                    Span::point(formation.state)
                } else {
                    consumed.span()
                };

                ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    span,
                )
            }),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    Span::point(formation.state),
                ));
            };
            let Some(end) = delimiters.last() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    start.span,
                ));
            };
            let span = Span::merge(&start.span, &end.span);

            let separator_token = if delimiters.len() > 2 {
                Some(delimiters[1].clone())
            } else {
                None
            };

            let kind = ElementKind::delimited(Delimited::new(
                start.clone(),
                elements,
                separator_token,
                end.clone(),
            ));

            *form = Form::output(Element::new(kind, span));

            Ok(())
        })
    }

    pub fn collection<'source>(
        item: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Formation::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
        })
        .into_optional();

        Formation::sequence([
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }),
            item.clone().into_optional(),
            Formation::persistence(
                Formation::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
            })
            .with_panic(|former, formation| {
                let consumed: Vec<Token> = formation
                    .consumed
                    .iter()
                    .filter_map(|index| former.consumed.get(*index).cloned())
                    .collect();

                let span = if consumed.is_empty() {
                    Span::point(formation.state)
                } else {
                    consumed.span()
                };

                ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    span,
                )
            }),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    Span::point(formation.state),
                ));
            };
            let Some(end) = delimiters.last() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    start.span,
                ));
            };
            let span = Span::merge(&start.span, &end.span);

            let separator_token = if delimiters.len() > 2 {
                Some(delimiters[1].clone())
            } else {
                None
            };

            let kind = ElementKind::delimited(Delimited::new(
                start.clone(),
                elements,
                separator_token,
                end.clone(),
            ));

            *form = Form::output(Element::new(kind, span));

            Ok(())
        })
    }

    pub fn series<'source>(
        item: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Formation::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
        })
        .into_optional();

        Formation::sequence([
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }),
            item.clone().into_optional(),
            Formation::persistence(
                Formation::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Formation::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
            })
            .with_panic(|former, formation| {
                let consumed: Vec<Token> = formation
                    .consumed
                    .iter()
                    .filter_map(|index| former.consumed.get(*index).cloned())
                    .collect();

                let span = if consumed.is_empty() {
                    Span::point(formation.state)
                } else {
                    consumed.span()
                };

                ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    span,
                )
            }),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    Span::point(formation.state),
                ));
            };
            let Some(end) = delimiters.last() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    start.span,
                ));
            };
            let span = Span::merge(&start.span, &end.span);

            let separator_token = if delimiters.len() > 2 {
                Some(delimiters[1].clone())
            } else {
                None
            };

            let kind = ElementKind::delimited(Delimited::new(
                start.clone(),
                elements,
                separator_token,
                end.clone(),
            ));

            *form = Form::output(Element::new(kind, span));

            Ok(())
        })
    }

    pub fn delimited<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let item = Formation::deferred(Self::element);

        Self::alternative([
            Self::bundle(item.clone()),
            Self::block(item.clone()),
            Self::group(item.clone()),
            Self::sequence(item.clone()),
            Self::collection(item.clone()),
            Self::series(item),
        ])
    }
}
