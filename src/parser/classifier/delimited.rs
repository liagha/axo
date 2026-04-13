use crate::{
    combinator::{Classifier, Form},
    data::*,
    parser::{Element, ElementKind, ErrorKind, ParseError, Parser},
    scanner::{PunctuationKind, Token, TokenKind},
    tracker::Span,
};

impl<'a> Parser<'a> {
    fn tail<'source>(
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
        separator: PunctuationKind,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let left = TokenKind::Punctuation(separator);
        let right = left.clone();

        Classifier::alternative([
            Classifier::sequence([
                Classifier::predicate(move |value: &Token| value.kind == left)
                    .with_ignore(),
                item.clone().with_panic(move |former, classifier| {
                    ParseError::new(
                        ErrorKind::Expected("an item after a separator"),
                        Self::span(former, &classifier),
                    )
                }),
            ]),
            item.with_transform(move |_, classifier| {
                Err(ParseError::new(
                    ErrorKind::MissingSeparator(right.clone()),
                    Span::point(classifier.state),
                ))
            }),
        ])
    }

    pub fn bundle<'source>(
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }),
            item.clone().into_optional(),
            Classifier::persistence(Self::tail(item, PunctuationKind::Comma), 0, None),
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
            })
            .with_panic(|former, classifier| {
                Self::delimiter(
                    PunctuationKind::LeftBrace,
                    PunctuationKind::RightBrace,
                    PunctuationKind::Comma,
                    former,
                    &classifier,
                )
            }),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    Span::point(classifier.state),
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
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }),
            item.clone().into_optional(),
            Classifier::persistence(Self::tail(item, PunctuationKind::Semicolon), 0, None),
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
            })
            .with_panic(|former, classifier| {
                Self::delimiter(
                    PunctuationKind::LeftBrace,
                    PunctuationKind::RightBrace,
                    PunctuationKind::Semicolon,
                    former,
                    &classifier,
                )
            }),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBrace,
                    )),
                    Span::point(classifier.state),
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
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }),
            item.clone().into_optional(),
            Classifier::persistence(Self::tail(item, PunctuationKind::Comma), 0, None),
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
            })
            .with_panic(|former, classifier| {
                Self::delimiter(
                    PunctuationKind::LeftParenthesis,
                    PunctuationKind::RightParenthesis,
                    PunctuationKind::Comma,
                    former,
                    &classifier,
                )
            }),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    Span::point(classifier.state),
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
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }),
            item.clone().into_optional(),
            Classifier::persistence(Self::tail(item, PunctuationKind::Semicolon), 0, None),
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
            })
            .with_panic(|former, classifier| {
                Self::delimiter(
                    PunctuationKind::LeftParenthesis,
                    PunctuationKind::RightParenthesis,
                    PunctuationKind::Semicolon,
                    former,
                    &classifier,
                )
            }),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftParenthesis,
                    )),
                    Span::point(classifier.state),
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
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }),
            item.clone().into_optional(),
            Classifier::persistence(Self::tail(item, PunctuationKind::Comma), 0, None),
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
            })
            .with_panic(|former, classifier| {
                Self::delimiter(
                    PunctuationKind::LeftBracket,
                    PunctuationKind::RightBracket,
                    PunctuationKind::Comma,
                    former,
                    &classifier,
                )
            }),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    Span::point(classifier.state),
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
        item: Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }),
            item.clone().into_optional(),
            Classifier::persistence(Self::tail(item, PunctuationKind::Semicolon), 0, None),
            Classifier::predicate(|t: &Token| {
                t.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
            })
            .with_panic(|former, classifier| {
                Self::delimiter(
                    PunctuationKind::LeftBracket,
                    PunctuationKind::RightBracket,
                    PunctuationKind::Semicolon,
                    former,
                    &classifier,
                )
            }),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let delimiters = form.collect_inputs();
            let elements = form.collect_outputs();

            let Some(start) = delimiters.first() else {
                return Err(ParseError::new(
                    ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                        PunctuationKind::LeftBracket,
                    )),
                    Span::point(classifier.state),
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
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
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
