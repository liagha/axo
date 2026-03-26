use {
    crate::{
        data::*,
        combinator::{Classifier, Form},
        parser::{Element, ElementKind, ErrorKind, ParseError, Parser},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::{Span, Spanned},
    },
};

impl<'a> Parser<'a> {
    pub fn bundle<'src>(
        item: Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::RightBrace))
                .with_panic(|former, classifier| {
                    let consumed: Vec<Token> = classifier
                        .consumed
                        .iter()
                        .filter_map(|index| former.consumed.get(*index).cloned())
                        .collect();

                    let span = if consumed.is_empty() {
                        Span::point(classifier.position)
                    } else {
                        consumed.span()
                    };

                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
                        span,
                    )
                }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
                        Span::point(classifier.position),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
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

    pub fn block<'src>(
        item: Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::RightBrace))
                .with_panic(|former, classifier| {
                    let consumed: Vec<Token> = classifier
                        .consumed
                        .iter()
                        .filter_map(|index| former.consumed.get(*index).cloned())
                        .collect();

                    let span = if consumed.is_empty() {
                        Span::point(classifier.position)
                    } else {
                        consumed.span()
                    };

                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
                        span,
                    )
                }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
                        Span::point(classifier.position),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
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

    pub fn group<'src>(
        item: Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis))
                .with_panic(|former, classifier| {
                    let consumed: Vec<Token> = classifier
                        .consumed
                        .iter()
                        .filter_map(|index| former.consumed.get(*index).cloned())
                        .collect();

                    let span = if consumed.is_empty() {
                        Span::point(classifier.position)
                    } else {
                        consumed.span()
                    };

                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
                        span,
                    )
                }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
                        Span::point(classifier.position),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
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

    pub fn sequence<'src>(
        item: Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis))
                .with_panic(|former, classifier| {
                    let consumed: Vec<Token> = classifier
                        .consumed
                        .iter()
                        .filter_map(|index| former.consumed.get(*index).cloned())
                        .collect();

                    let span = if consumed.is_empty() {
                        Span::point(classifier.position)
                    } else {
                        consumed.span()
                    };

                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
                        span,
                    )
                }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
                        Span::point(classifier.position),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis)),
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

    pub fn collection<'src>(
        item: Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::RightBracket))
                .with_panic(|former, classifier| {
                    let consumed: Vec<Token> = classifier
                        .consumed
                        .iter()
                        .filter_map(|index| former.consumed.get(*index).cloned())
                        .collect();

                    let span = if consumed.is_empty() {
                        Span::point(classifier.position)
                    } else {
                        consumed.span()
                    };

                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
                        span,
                    )
                }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
                        Span::point(classifier.position),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
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

    pub fn series<'src>(
        item: Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let separator = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(|t: &Token| t.kind == TokenKind::Punctuation(PunctuationKind::RightBracket))
                .with_panic(|former, classifier| {
                    let consumed: Vec<Token> = classifier
                        .consumed
                        .iter()
                        .filter_map(|index| former.consumed.get(*index).cloned())
                        .collect();

                    let span = if consumed.is_empty() {
                        Span::point(classifier.position)
                    } else {
                        consumed.span()
                    };

                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
                        span,
                    )
                }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
                        Span::point(classifier.position),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
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

    pub fn delimited<'src>() -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let item = Classifier::deferred(Self::element);

        Classifier::alternative([
            Self::bundle(item.clone()),
            Self::block(item.clone()),
            Self::group(item.clone()),
            Self::sequence(item.clone()),
            Self::collection(item.clone()),
            Self::series(item),
        ])
    }
}
