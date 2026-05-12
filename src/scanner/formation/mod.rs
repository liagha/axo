mod escape;
mod number;

use {
    crate::{
        data::Str,
        scanner::{
            Character, CharacterError, ErrorKind, Operator, Punctuation, PunctuationKind,
            ScanError, Scanner, Token, TokenKind,
        },
        tracker::Spanned,
    },
    chaint::{Form, Formation},
};

impl<'a> Scanner<'a> {
    fn string<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('"'),
            Formation::repetition(
                Formation::alternative([
                    Formation::predicate(|c: &Character| !matches!(c.value, '"' | '\\')),
                    Self::escape_sequence(),
                ]),
                0,
                None,
            ),
            Formation::literal('"'),
        ])
        .with_transform(move |joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let mut inputs = form.collect_inputs();
            let span = inputs.span().clone();
            if inputs.len() >= 2 {
                inputs.drain(0..1);
                inputs.pop();
            }
            let content: Str = inputs.into_iter().collect();

            *form = Form::output(Token::new(TokenKind::string(content), span));

            Ok(())
        })
    }

    fn backtick<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('`'),
            Formation::repetition(
                Formation::alternative([
                    Formation::predicate(|c: &Character| !matches!(c.value, '`' | '\\')),
                    Self::escape_sequence(),
                ]),
                0,
                None,
            ),
            Formation::literal('`'),
        ])
        .with_transform(move |joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let mut inputs = form.collect_inputs();
            let span = inputs.span().clone();
            if inputs.len() >= 2 {
                inputs.drain(0..1);
                inputs.pop();
            }
            let content: Str = inputs.into_iter().collect();

            *form = Form::output(Token::new(TokenKind::string(content), span));

            Ok(())
        })
    }

    fn character<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::sequence([
            Formation::literal('\''),
            Formation::alternative([
                Formation::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
                Self::escape_sequence(),
            ]),
            Formation::literal('\''),
        ])
        .with_transform(|joint| {
            let (former, formation) = (&mut joint.0, &mut joint.1);

            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();
            let character = inputs[1];
            let span = inputs.span().clone();

            *form = Form::output(Token::new(TokenKind::character(character.value), span));

            Ok(())
        })
    }

    fn identifier<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::sequence([
                Formation::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Formation::persistence(
                    Formation::predicate(|c: &Character| c.is_alphanumeric() || *c == '_'),
                    0,
                    None,
                ),
            ]),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
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

    fn operator<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::persistence(
                Formation::predicate(|c: &Character| c.is_operator()),
                1,
                Some(3),
            ),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let inputs = form.collect_inputs();
                let span = inputs.span().clone();
                let content = inputs.into_iter().collect::<Str>();

                *form = Form::output(Token::new(TokenKind::operator(content.to_operator()), span));

                Ok(())
            },
        )
    }

    fn punctuation<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::predicate(|c: &Character| c.is_punctuation()),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
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

    fn whitespace<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::predicate(|c: &Character| c.is_whitespace() && *c != '\n'),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let input = form.unwrap_input();
                let span = input.span().clone();

                *form = Form::output(Token::new(
                    TokenKind::Punctuation(PunctuationKind::Space),
                    span,
                ));

                Ok(())
            },
        )
    }

    fn comment<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_transform(
            Formation::sequence([Formation::alternative([
                Formation::sequence([
                    Formation::sequence([Formation::literal('/'), Formation::literal('/')]),
                    Formation::persistence(
                        Formation::predicate(|c: &Character| *c != '\n'),
                        0,
                        None,
                    ),
                ]),
                Formation::sequence([
                    Formation::sequence([Formation::literal('/'), Formation::literal('*')]),
                    Formation::persistence(
                        Formation::predicate(|c: &Character| *c != '*'),
                        0,
                        None,
                    ),
                    Formation::sequence([Formation::literal('*'), Formation::literal('/')]),
                ]),
            ])]),
            |joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let mut inputs = form.collect_inputs();
                let span = inputs.span().clone();

                if inputs.len() >= 4
                    && inputs[0].value == '/'
                    && inputs[1].value == '*'
                    && inputs[inputs.len() - 2].value == '*'
                    && inputs[inputs.len() - 1].value == '/'
                {
                    inputs.drain(0..2);
                    inputs.drain(inputs.len() - 2..);
                } else if inputs.len() >= 2 && inputs[0].value == '/' && inputs[1].value == '/' {
                    inputs.drain(0..2);
                }

                let content: Str = inputs.into_iter().collect();

                *form = Form::output(Token::new(TokenKind::comment(content), span));

                Ok(())
            },
        )
    }

    fn fallback<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>> {
        Formation::with_combinator(
            Formation::anything(),
            Formation::fail(|joint| {
                let (former, formation) = (&mut joint.0, &mut joint.1);

                let form = former.forms.get_mut(formation.form).unwrap();
                let ch: &Character = form.unwrap_input();

                ScanError::new(
                    ErrorKind::InvalidCharacter(CharacterError::Unexpected(*ch)),
                    ch.span,
                )
            }),
        )
    }

    pub fn formation<'source>() -> Formation<'a, 'source, Self, Character, Token<'a>, ScanError<'a>>
    {
        Formation::persistence(
            Formation::alternative([
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
