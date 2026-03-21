use {
    crate::{
        data::*,
        formation::{Classifier, Form},
        parser::{Element, ElementKind, ErrorKind, ParseError, Parser},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::Span,
    },
};

impl<'parser> Parser<'parser> {
    fn builder(
        open: PunctuationKind,
        close: PunctuationKind,
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        let separator = Classifier::predicate(|token: &Token| {
            matches!(
                token.kind,
                TokenKind::Punctuation(PunctuationKind::Comma)
                    | TokenKind::Punctuation(PunctuationKind::Semicolon)
            )
        })
            .into_optional();

        Classifier::sequence([
            Classifier::predicate(move |t: &Token| t.kind == TokenKind::Punctuation(open)),
            item.clone().into_optional(),
            Classifier::persistence(
                Classifier::sequence([separator, item.into_optional()]),
                0,
                None,
            ),
            Classifier::predicate(move |t: &Token| t.kind == TokenKind::Punctuation(close)),
        ])
            .with_transform(move |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();

                let Some(start) = delimiters.first() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(open)),
                        Span::void(),
                    ));
                };
                let Some(end) = delimiters.last() else {
                    return Err(ParseError::new(
                        ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(open)),
                        Span::void(),
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

    pub fn bundle(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::builder(PunctuationKind::LeftBrace, PunctuationKind::RightBrace, item)
    }

    pub fn block(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::builder(PunctuationKind::LeftBrace, PunctuationKind::RightBrace, item)
    }

    pub fn group(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::builder(
            PunctuationKind::LeftParenthesis,
            PunctuationKind::RightParenthesis,
            item,
        )
    }

    pub fn sequence(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::builder(
            PunctuationKind::LeftParenthesis,
            PunctuationKind::RightParenthesis,
            item,
        )
    }

    pub fn collection(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::builder(
            PunctuationKind::LeftBracket,
            PunctuationKind::RightBracket,
            item,
        )
    }

    pub fn series(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::builder(
            PunctuationKind::LeftBracket,
            PunctuationKind::RightBracket,
            item,
        )
    }

    pub fn delimited() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::builder(
                PunctuationKind::LeftBrace,
                PunctuationKind::RightBrace,
                Classifier::deferred(Self::element),
            ),
            Self::builder(
                PunctuationKind::LeftParenthesis,
                PunctuationKind::RightParenthesis,
                Classifier::deferred(Self::element),
            ),
            Self::builder(
                PunctuationKind::LeftBracket,
                PunctuationKind::RightBracket,
                Classifier::deferred(Self::element),
            ),
        ])
    }
}
