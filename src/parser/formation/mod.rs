mod delimited;
mod symbol;

use crate::{
    combinator::{Form, Formation, Former},
    data::*,
    parser::{Element, ElementKind, ErrorKind, ParseError, Parser},
    scanner::{PunctuationKind, Token, TokenKind},
    tracker::{Peekable, Span, Spanned},
};

impl<'a> Parser<'a> {
    #[inline]
    fn alternative<'source, const SIZE: Scale>(
        patterns: [Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>; SIZE],
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Formation::alternative_with(
            patterns,
            |state| state.is_aligned(),
            |new, old| {
                if new.is_aligned() != old.is_aligned() {
                    return new.is_aligned();
                }

                if new.is_aligned() {
                    return new.marker > old.marker;
                }

                if new.is_panicked() != old.is_panicked() {
                    return new.is_panicked();
                }

                if new.is_panicked() {
                    return new.marker > old.marker;
                }

                if new.is_failed() != old.is_failed() {
                    return new.is_failed();
                }

                new.marker > old.marker
            },
        )
    }

    #[inline]
    fn recover_sync(token: &Token<'a>) -> bool {
        matches!(
            token.kind,
            TokenKind::Punctuation(PunctuationKind::Semicolon)
                | TokenKind::Punctuation(PunctuationKind::Comma)
                | TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                | TokenKind::Punctuation(PunctuationKind::RightBracket)
                | TokenKind::Punctuation(PunctuationKind::RightBrace)
        )
    }

    #[inline]
    fn recover_emit<'source>(
        former: &mut Former<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
        formation: Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>,
    ) -> ParseError<'a> {
        if let Some(form) = former.forms.get(formation.form) {
            if let Some(error) = form.get_failure() {
                return error.clone();
            }
        }

        if let Some(token) = former.source.get(formation.marker) {
            return ParseError::new(ErrorKind::UnexpectedToken(token.kind.clone()), token.span);
        }

        ParseError::new(ErrorKind::ExpectedBody, Span::point(formation.state))
    }

    pub fn get_body(element: Element<'a>) -> Vec<Element<'a>> {
        match element.kind {
            ElementKind::Delimited(delimited) => delimited.members,
            _ => vec![element],
        }
    }

    pub fn literal<'source>() -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Formation::predicate(|token: &Token| match &token.kind {
            TokenKind::String(_)
            | TokenKind::Character(_)
            | TokenKind::Boolean(_)
            | TokenKind::Float(_)
            | TokenKind::Integer(_) => true,
            TokenKind::Identifier(identifier) => !matches!(
                identifier.unwrap_str(),
                "static" | "let" | "struct" | "union" | "func" | "module"
            ),
            _ => false,
        })
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let input = inputs.into_iter().next().unwrap();
            let span = input.span;

            *form = Form::output(Element::new(ElementKind::literal(input), span));

            Ok(())
        })
    }

    pub fn primary<'source>() -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Self::alternative([
            Formation::deferred(Self::delimited),
            Formation::deferred(Self::literal),
        ])
    }

    pub fn prefixed<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Formation::sequence([
            Formation::predicate(|token: &Token| {
                if let TokenKind::Operator(operator) = &token.kind {
                    operator.is_prefix()
                } else {
                    false
                }
            }),
            Formation::deferred(Self::primary),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let prefixes = form.collect_inputs();
            let mut outputs = form.collect_outputs();
            let mut unary = outputs.swap_remove(0);

            for prefix in prefixes {
                let span = Span::merge(&prefix.span(), &unary.span());
                unary = Element::new(ElementKind::unary(Unary::new(prefix, unary)), span);
            }

            *form = Form::output(unary);
            Ok(())
        })
    }

    pub fn suffixed<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Formation::sequence([
            Formation::deferred(Self::primary),
            Formation::repetition(
                Self::alternative([
                    Self::group(Formation::deferred(Self::element)),
                    Self::collection(Formation::deferred(Self::element)),
                    Self::bundle(Formation::deferred(Self::element)),
                    Formation::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            operator.is_suffix()
                        } else {
                            false
                        }
                    }),
                ]),
                0,
                None,
            ),
        ])
        .with_transform(move |former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let sequence = form.as_forms();
            let operand = sequence[0].unwrap_output();
            let suffixes = sequence[1].as_forms();
            let mut unary = operand.clone();

            for suffix in suffixes {
                if let Some(token) = suffix.get_input() {
                    let span = Span::merge(&unary.span(), &token.span());
                    unary = Element::new(ElementKind::unary(Unary::new(token, unary)), span);
                } else if let Some(element) = suffix.get_output() {
                    let span = Span::merge(&unary.span(), &element.span());
                    unary = Self::apply_suffix(unary, element, span);
                }
            }

            *form = Form::output(unary);
            Ok(())
        })
    }

    fn apply_suffix(target: Element<'a>, suffix: Element<'a>, span: Span) -> Element<'a> {
        let ElementKind::Delimited(delimited) = suffix.kind else {
            return target;
        };

        let start = &delimited.start.kind;
        let end = &delimited.end.kind;
        let sep = delimited.separator.as_ref().map(|t| &t.kind);

        match (start, sep, end) {
            (
                TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                TokenKind::Punctuation(PunctuationKind::RightParenthesis),
            ) => Element::new(
                ElementKind::invoke(Invoke::new(target, delimited.members)),
                span,
            ),

            (
                TokenKind::Punctuation(PunctuationKind::LeftBracket),
                None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                TokenKind::Punctuation(PunctuationKind::RightBracket),
            ) => Element::new(
                ElementKind::index(Index::new(target, delimited.members)),
                span,
            ),

            (
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                TokenKind::Punctuation(PunctuationKind::RightBrace),
            ) => Element::new(
                ElementKind::construct(Aggregate::new(target, delimited.members)),
                span,
            ),

            _ => target,
        }
    }

    pub fn unary<'source>() -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Self::alternative([
            Formation::deferred(Self::prefixed),
            Formation::deferred(Self::suffixed),
            Formation::deferred(Self::primary),
        ])
    }

    pub fn binary<'source>() -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Self::alternative([Formation::with_transform(
            Formation::sequence([
                Formation::deferred(Self::unary),
                Formation::repetition(
                    Formation::sequence([
                        Formation::predicate(|token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                operator.precedence().is_some()
                            } else {
                                false
                            }
                        }),
                        Formation::deferred(Self::unary),
                    ]),
                    1,
                    None,
                ),
            ]),
            |former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let sequence = form.as_forms();
                let left = sequence[0].unwrap_output().clone();
                let operations = sequence[1].as_forms();

                let mut pairs = Vec::with_capacity(operations.len());

                for operation in operations {
                    let inner = operation.as_forms();
                    if inner.len() >= 2 {
                        let operator = inner[0].unwrap_input().clone();
                        let operand = inner[1].unwrap_output().clone();
                        let precedence = if let TokenKind::Operator(op) = &operator.kind {
                            op.precedence().unwrap_or(0)
                        } else {
                            0
                        };
                        pairs.push((operator, operand, precedence));
                    }
                }

                let result = Self::climb(left, &pairs, 0, 0).0;

                *form = Form::output(result);
                Ok(())
            },
        )])
    }

    fn climb(
        mut left: Element<'a>,
        pairs: &[(Token<'a>, Element<'a>, u8)],
        threshold: u8,
        start: usize,
    ) -> (Element<'a>, usize) {
        let mut current = start;

        while current < pairs.len() {
            let precedence = pairs[current].2;

            if precedence < threshold {
                break;
            }

            let operator = &pairs[current];
            let mut right = operator.1.clone();
            let op_token = operator.0.clone();
            current += 1;

            while current < pairs.len() && pairs[current].2 > precedence {
                let (new_right, new_current) = Self::climb(right, pairs, precedence + 1, current);
                right = new_right;
                current = new_current;
            }

            let span = left.span().merge(&right.span());

            left = Element::new(
                ElementKind::binary(Binary::new(left, op_token, right)),
                span,
            );
        }

        (left, current)
    }

    pub fn expression<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Self::alternative([
            Formation::deferred(Self::binary),
            Formation::deferred(Self::unary),
            Formation::deferred(Self::primary),
        ])
    }

    pub fn element<'source>() -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Self::alternative([
            Formation::deferred(Self::symbolization),
            Formation::deferred(Self::expression),
        ])
    }

    pub fn fallback<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Formation::with_fail(Formation::anything(), |former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let token = form.unwrap_input();

            ParseError::new(
                ErrorKind::UnexpectedToken(form.unwrap_input().clone().kind),
                token.span,
            )
        })
    }

    pub fn parser<'source>() -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Formation::repetition(
            Self::alternative([Formation::deferred(Self::element)
                .with_recover(Self::recover_sync, Self::recover_emit)]),
            0,
            None,
        )
    }
}
