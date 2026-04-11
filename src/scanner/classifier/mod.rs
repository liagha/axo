mod escape;
mod number;

use crate::{
    combinator::{Classifier, Form},
    data::Str,
    scanner::{
        Character, CharacterError, ErrorKind, Operator, Punctuation, PunctuationKind, ScanError,
        Scanner, Token, TokenKind,
    },
    tracker::Spanned,
};

impl<'a> Scanner<'a> {
    fn string<'source>() -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>> {
        Classifier::sequence([
            Classifier::literal('"').with_ignore(),
            Classifier::repetition(
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '"' | '\\')),
                    Self::escape_sequence(),
                ]),
                0,
                None,
            ),
            Classifier::literal('"').with_ignore(),
        ])
        .with_transform(move |former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let inputs = form.collect_inputs();
            let span = inputs.span().clone();
            let content = inputs.into_iter().collect::<Str>();

            *form = Form::output(Token::new(TokenKind::string(content), span));

            Ok(())
        })
    }

    fn backtick<'source>() -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>>
    {
        Classifier::sequence([
            Classifier::literal('`').with_ignore(),
            Classifier::repetition(
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '`' | '\\')),
                    Self::escape_sequence(),
                ]),
                0,
                None,
            ),
            Classifier::literal('`').with_ignore(),
        ])
        .with_transform(move |former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let inputs = form.collect_inputs();
            let span = inputs.span().clone();
            let content = inputs.into_iter().collect::<Str>();

            *form = Form::output(Token::new(TokenKind::string(content), span));

            Ok(())
        })
    }

    fn character<'source>() -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>>
    {
        Classifier::sequence([
            Classifier::literal('\''),
            Classifier::alternative([
                Classifier::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
                Self::escape_sequence(),
            ]),
            Classifier::literal('\''),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let inputs = form.collect_inputs();
            let character = inputs[1];

            *form = Form::output(Token::new(
                TokenKind::character(character.value),
                character.span,
            ));

            Ok(())
        })
    }

    fn identifier<'source>(
    ) -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Classifier::persistence(
                    Classifier::predicate(|c: &Character| c.is_alphanumeric() || *c == '_'),
                    0,
                    None,
                ),
            ]),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let content = inputs.into_iter().collect::<Str>();

                let token = match content.unwrap_str() {
                    "true" => TokenKind::Boolean(true),
                    "false" => TokenKind::Boolean(false),
                    _ => TokenKind::identifier(content),
                };

                *form = Form::output(Token::new(token, span));

                Ok(())
            },
        )
    }

    fn operator<'source>() -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>>
    {
        Classifier::with_transform(
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.is_operator()),
                1,
                Some(3),
            ),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let content = inputs.into_iter().collect::<Str>();

                *form = Form::output(Token::new(TokenKind::Operator(content.to_operator()), span));

                Ok(())
            },
        )
    }

    fn punctuation<'source>(
    ) -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>> {
        Classifier::with_transform(
            Classifier::predicate(|c: &Character| c.is_punctuation()),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let content = inputs.into_iter().collect::<Str>();

                *form = Form::output(Token::new(
                    TokenKind::Punctuation(content.to_punctuation()),
                    span,
                ));

                Ok(())
            },
        )
    }

    fn whitespace<'source>(
    ) -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>> {
        Classifier::with_transform(
            Classifier::predicate(|c: &Character| c.is_whitespace() && *c != '\n'),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let input = form.unwrap_input();
                let span = input.span().clone();

                *form = Form::output(Token::new(TokenKind::Punctuation(PunctuationKind::Space), span));

                Ok(())
            },
        )
    }

    fn comment<'source>() -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>>
    {
        Classifier::with_transform(
            Classifier::sequence([Classifier::alternative([
                Classifier::sequence([
                    Classifier::sequence([Classifier::literal('/'), Classifier::literal('/')])
                        .with_ignore(),
                    Classifier::persistence(
                        Classifier::predicate(|c: &Character| *c != '\n'),
                        0,
                        None,
                    ),
                ]),
                Classifier::sequence([
                    Classifier::sequence([Classifier::literal('/'), Classifier::literal('*')])
                        .with_ignore(),
                    Classifier::persistence(
                        Classifier::predicate(|c: &Character| *c != '*'),
                        0,
                        None,
                    ),
                    Classifier::sequence([Classifier::literal('*'), Classifier::literal('/')])
                        .with_ignore(),
                ]),
            ])]),
            |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let content = inputs.into_iter().collect::<Str>();

                *form = Form::output(Token::new(TokenKind::comment(content), span));

                Ok(())
            },
        )
    }

    fn fallback<'source>() -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>>
    {
        Classifier::with_action(
            Classifier::anything(),
            Classifier::fail(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let ch: &Character = form.unwrap_input();

                ScanError::new(
                    ErrorKind::InvalidCharacter(CharacterError::Unexpected(*ch)),
                    ch.span,
                )
            }),
        )
    }

    pub fn classifier<'source>(
    ) -> Classifier<'a, 'source, Self, Character<'a>, Token<'a>, ScanError<'a>> {
        Classifier::persistence(
            Classifier::alternative([
                Self::whitespace(),
                Self::comment(),
                Self::identifier(),
                Self::number(),
                Self::string(),
                Self::backtick(),
                Self::character(),
                Self::operator(),
                Self::punctuation(),
                Self::fallback(),
            ]),
            0,
            None,
        )
    }
}
