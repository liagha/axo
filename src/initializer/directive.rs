use crate::{
    combinator::{Form, Formation},
    data::{Binding, BindingKind, Str},
    initializer::{InitializeError, Initializer},
    parser::{Element, ElementKind, Symbol, SymbolKind},
    scanner::{OperatorKind, Token, TokenKind},
    tracker::Spanned,
};

impl<'a> Initializer<'a> {
    fn stringify(tokens: Vec<Token<'a>>) -> String {
        let mut result = String::new();
        for input in tokens {
            if let Some(identifier) = input.kind.try_unwrap_identifier() {
                result.push_str(identifier);
            } else if let Some(value) = input.kind.try_unwrap_string() {
                result.push_str(value.as_str().unwrap_or(""));
            } else if let Some(value) = input.kind.try_unwrap_integer() {
                result.push_str(&value.to_string());
            } else if let Some(operator) = input.kind.try_unwrap_operator() {
                for operator in operator.as_slice() {
                    match operator {
                        OperatorKind::Slash => result.push('/'),
                        OperatorKind::Dot => result.push('.'),
                        OperatorKind::Backslash => result.push('\\'),
                        OperatorKind::Colon => result.push(':'),
                        _ => {}
                    }
                }
            }
        }
        result
    }

    fn separator<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::predicate(|token: &Token| {
            token
                .kind
                .try_unwrap_operator()
                .map(|operator| {
                    operator.as_slice().iter().all(|operator| {
                        matches!(
                            operator,
                            OperatorKind::Slash
                                | OperatorKind::Backslash
                                | OperatorKind::Dot
                                | OperatorKind::Colon
                        )
                    })
                })
                .unwrap_or(false)
        })
    }

    fn segment<'source>() -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>>
    {
        Formation::predicate(|token: &Token| {
            token.kind.is_identifier() || token.kind.is_string() || token.kind.is_integer()
        })
    }

    fn value<'source>() -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>>
    {
        Formation::sequence([
            Formation::repetition(Self::separator(), 0, None),
            Formation::repetition(
                Formation::sequence([
                    Self::segment(),
                    Formation::repetition(Self::separator(), 1, None),
                ]),
                0,
                None,
            ),
            Formation::repetition(Self::segment(), 0, Some(1)),
            Formation::repetition(Self::separator(), 0, None),
        ])
    }

    fn path<'source>(
        name: Str<'a>,
        matcher: fn(&Str<'a>) -> bool,
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::with_transform(
            Formation::sequence([
                Formation::alternative([
                    Formation::repetition(
                        Formation::predicate(|token: &Token| {
                            matches!(token.kind.try_unwrap_operator(), Some(OperatorKind::Minus))
                        }),
                        1,
                        Some(2),
                    )
                    .with_ignore(),
                    Formation::predicate(|token: &Token| {
                        matches!(
                            token.kind.try_unwrap_operator(),
                            Some(op) if op.as_slice() == [OperatorKind::Minus, OperatorKind::Minus]
                        )
                    })
                    .with_ignore(),
                ]),
                Formation::predicate(move |token: &Token| {
                    if let Some(identifier) = token.kind.try_unwrap_identifier() {
                        matcher(identifier)
                    } else {
                        false
                    }
                })
                .with_transform(move |former, formation| {
                    let form = former.forms.get_mut(formation.form).unwrap();
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    *form = Form::Input(Token::new(TokenKind::identifier(name.clone()), span));
                    Ok(())
                }),
                Self::value(),
            ]),
            move |former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let forms = form.as_forms();
                let identifier = forms[0].unwrap_input().clone();
                let input = forms[1].collect_inputs();
                let span = identifier.span().merge(&input.span());

                let target = Element::new(
                    ElementKind::literal(Token::new(
                        TokenKind::identifier(name),
                        identifier.span().clone(),
                    )),
                    identifier.span(),
                );

                let value = Element::new(
                    ElementKind::literal(Token::new(
                        TokenKind::identifier(Str::from(Self::stringify(input.clone()))),
                        input.span(),
                    )),
                    input.span(),
                );

                *form = Form::Output(Symbol::new(
                    SymbolKind::binding(Binding::new(
                        target,
                        Some(value),
                        None,
                        BindingKind::Static,
                    )),
                    span,
                ));
                Ok(())
            },
        )
    }

    fn flag<'source>(
        name: Str<'a>,
        matcher: fn(&Str<'a>) -> bool,
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::sequence([
            Formation::alternative([
                Formation::repetition(
                    Formation::predicate(|token: &Token| {
                        matches!(token.kind.try_unwrap_operator(), Some(OperatorKind::Minus))
                    }),
                    1,
                    Some(2),
                )
                .with_ignore(),
                Formation::predicate(|token: &Token| {
                    matches!(
                        token.kind.try_unwrap_operator(),
                        Some(op) if op.as_slice() == [OperatorKind::Minus, OperatorKind::Minus]
                    )
                })
                .with_ignore(),
            ]),
            Formation::predicate(move |token: &Token| {
                if let Some(identifier) = token.kind.try_unwrap_identifier() {
                    matcher(identifier)
                } else {
                    false
                }
            }),
        ])
        .with_transform(move |former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let identifier = form.collect_inputs()[0].clone();
            let span = identifier.span;

            let target = Element::new(
                ElementKind::literal(Token::new(TokenKind::identifier(name.clone()), span)),
                span,
            );

            let value = Element::new(
                ElementKind::literal(Token::new(TokenKind::identifier(Str::from("true")), span)),
                span,
            );

            *form = Form::Output(Symbol::new(
                SymbolKind::binding(Binding::new(target, Some(value), None, BindingKind::Static)),
                span,
            ));
            Ok(())
        })
    }

    pub fn verbosity<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::sequence([
            Formation::repetition(
                Formation::predicate(|token: &Token| {
                    matches!(token.kind.try_unwrap_operator(), Some(OperatorKind::Minus))
                }),
                1,
                Some(2),
            )
            .with_ignore(),
            Formation::predicate(|token: &Token| {
                if let Some(identifier) = token.kind.try_unwrap_identifier() {
                    *identifier == "v" || *identifier == "verbosity"
                } else {
                    false
                }
            })
            .with_transform(|former, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let identifier = form.collect_inputs()[0].clone();
                let span = identifier.span();

                *form = Form::Input(Token::new(
                    TokenKind::identifier(Str::from("Verbosity")),
                    span,
                ));
                Ok(())
            }),
            Formation::predicate(|token: &Token| token.kind.is_integer()),
        ])
        .with_transform(|former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let identifier = form.collect_inputs()[0].clone();
            let value = form.collect_inputs()[1].clone();
            let span = identifier.span.merge(&value.span);

            let target = Element::new(ElementKind::literal(identifier.clone()), identifier.span);
            let value = Element::new(ElementKind::literal(value.clone()), value.span);

            *form = Form::Output(Symbol::new(
                SymbolKind::binding(Binding::new(target, Some(value), None, BindingKind::Static)),
                span,
            ));
            Ok(())
        })
    }

    pub fn target<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::path(Str::from("Target"), |identifier| {
            *identifier == "t" || *identifier == "target"
        })
    }

    pub fn input<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::path(Str::from("Input"), |identifier| {
            *identifier == "i" || *identifier == "input"
        })
    }

    pub fn output<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::path(Str::from("Output"), |identifier| {
            *identifier == "o" || *identifier == "output"
        })
    }

    pub fn discard<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::flag(Str::from("Discard"), |identifier| *identifier == "discard")
    }

    pub fn bare<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::flag(Str::from("Bare"), |identifier| *identifier == "bare")
    }

    pub fn cranelift<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::flag(Str::from("Cranelift"), |identifier| {
            *identifier == "cranelift"
        })
    }

    pub fn implicit<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::with_transform(Self::value(), |former, formation| {
            let form = former.forms.get_mut(formation.form).unwrap();
            let inputs = form.collect_inputs();

            if inputs.is_empty() {
                *form = Form::blank();
                return Ok(());
            }

            let span = inputs.span();
            let value = Token::new(
                TokenKind::identifier(Str::from(Self::stringify(inputs))),
                span,
            );
            let identifier = Token::new(TokenKind::identifier(Str::from("Input")), span);

            let target = Element::new(ElementKind::literal(identifier.clone()), identifier.span);
            let value = Element::new(ElementKind::literal(value.clone()), value.span);

            *form = Form::Output(Symbol::new(
                SymbolKind::binding(Binding::new(target, Some(value), None, BindingKind::Static)),
                span,
            ));
            Ok(())
        })
    }
}
