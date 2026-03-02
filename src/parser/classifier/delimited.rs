use {
    super::super::{Element, ElementKind, ErrorKind, ParseError, Parser},
    crate::{
        formation::{classifier::Classifier, form::Form},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::Span,
    },
};
use crate::data::*;

impl<'parser> Parser<'parser> {
    fn delimited_form(
        open: PunctuationKind,
        close: PunctuationKind,
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        let separator = Classifier::predicate(|t: &Token| {
            matches!(
                t.kind,
                TokenKind::Punctuation(PunctuationKind::Comma)
                    | TokenKind::Punctuation(PunctuationKind::Semicolon)
            )
        })
            .as_optional();

        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(move |t: &Token| t.kind == TokenKind::Punctuation(open)),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([separator, item.as_optional()]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(move |t: &Token| t.kind == TokenKind::Punctuation(close)),
                    Classifier::fail(move |_form: Form<Token, Element, ParseError>| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(open)),
                            Span::void(),
                        )
                    }),
                ),
            ]),
            move |form| {
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

                let kind = match (open, separator_token.as_ref().map(|t| &t.kind)) {
                    (PunctuationKind::LeftBrace, None) => {
                        Self::delimited_kind(start, &elements, None, end)
                    }
                    (
                        PunctuationKind::LeftBrace,
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    ) => Self::delimited_kind(start, &elements, separator_token.as_ref(), end),
                    (
                        PunctuationKind::LeftBrace,
                        Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    ) => Self::delimited_kind(start, &elements, separator_token.as_ref(), end),

                    (PunctuationKind::LeftParenthesis, None) => {
                        Self::delimited_kind(start, &elements, None, end)
                    }
                    (
                        PunctuationKind::LeftParenthesis,
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    ) => Self::delimited_kind(start, &elements, separator_token.as_ref(), end),
                    (
                        PunctuationKind::LeftParenthesis,
                        Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    ) => Self::delimited_kind(start, &elements, separator_token.as_ref(), end),

                    (PunctuationKind::LeftBracket, None) => {
                        Self::delimited_kind(start, &elements, None, end)
                    }
                    (
                        PunctuationKind::LeftBracket,
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    ) => Self::delimited_kind(start, &elements, separator_token.as_ref(), end),
                    (
                        PunctuationKind::LeftBracket,
                        Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    ) => Self::delimited_kind(start, &elements, separator_token.as_ref(), end),

                    _ => unreachable!("unexpected bracket/separator combination"),
                };

                Ok(Form::output(Element::new(kind, span)))
            },
        )
    }

    fn delimited_kind(
        start: &Token<'parser>,
        elements: &[Element<'parser>],
        sep: Option<&Token<'parser>>,
        end: &Token<'parser>,
    ) -> ElementKind<'parser> {
        ElementKind::delimited(Delimited::new(
            start.clone(),
            elements.to_vec(),
            sep.cloned(),
            end.clone(),
        ))
    }

    pub fn bundle(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::delimited_form(
            PunctuationKind::LeftBrace,
            PunctuationKind::RightBrace,
            item,
        )
    }

    pub fn block(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::delimited_form(
            PunctuationKind::LeftBrace,
            PunctuationKind::RightBrace,
            item,
        )
    }

    pub fn group(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::delimited_form(
            PunctuationKind::LeftParenthesis,
            PunctuationKind::RightParenthesis,
            item,
        )
    }

    pub fn sequence(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::delimited_form(
            PunctuationKind::LeftParenthesis,
            PunctuationKind::RightParenthesis,
            item,
        )
    }

    pub fn collection(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::delimited_form(
            PunctuationKind::LeftBracket,
            PunctuationKind::RightBracket,
            item,
        )
    }

    pub fn series(
        item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::delimited_form(
            PunctuationKind::LeftBracket,
            PunctuationKind::RightBracket,
            item,
        )
    }

    pub fn delimited() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>
    {
        Classifier::alternative([
            Self::delimited_form(
                PunctuationKind::LeftBrace,
                PunctuationKind::RightBrace,
                Classifier::deferred(Self::element),
            ),
            Self::delimited_form(
                PunctuationKind::LeftParenthesis,
                PunctuationKind::RightParenthesis,
                Classifier::deferred(Self::element),
            ),
            Self::delimited_form(
                PunctuationKind::LeftBracket,
                PunctuationKind::RightBracket,
                Classifier::deferred(Self::element),
            ),
        ])
    }
}
