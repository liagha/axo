use crate::{
    combinator::{Classifier, Form},
    data::{Binding, BindingKind, Str},
    initializer::{InitializeError, Initializer},
    parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
    scanner::{OperatorKind, Token, TokenKind},
    tracker::Spanned,
};

impl<'a> Initializer<'a> {
    fn path_string(tokens: Vec<Token<'a>>) -> String {
        let mut result = String::new();
        for input in tokens {
            match input.kind {
                TokenKind::Identifier(identifier) => result.push_str(&identifier),
                TokenKind::String(value) => result.push_str(value.as_str().unwrap_or("")),
                TokenKind::Integer(value) => result.push_str(&value.to_string()),
                TokenKind::Operator(OperatorKind::Slash) => result.push('/'),
                TokenKind::Operator(OperatorKind::Dot) => result.push('.'),
                TokenKind::Operator(OperatorKind::Backslash) => result.push('\\'),
                TokenKind::Operator(OperatorKind::Colon) => result.push(':'),
                _ => {}
            }
        }
        result
    }

    fn path_value<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::Identifier(_)
                        | TokenKind::String(_)
                        | TokenKind::Integer(_)
                        | TokenKind::Operator(OperatorKind::Slash)
                        | TokenKind::Operator(OperatorKind::Backslash)
                        | TokenKind::Operator(OperatorKind::Dot)
                )
            }),
            Classifier::repetition(
                Classifier::predicate(|token: &Token| {
                    matches!(
                        token.kind,
                        TokenKind::Identifier(_)
                            | TokenKind::String(_)
                            | TokenKind::Integer(_)
                            | TokenKind::Operator(OperatorKind::Slash)
                            | TokenKind::Operator(OperatorKind::Backslash)
                            | TokenKind::Operator(OperatorKind::Dot)
                            | TokenKind::Operator(OperatorKind::Colon)
                    )
                }),
                0,
                None,
            ),
        ])
    }

    fn path_directive<'source>(
        name: Str<'a>,
        matcher: fn(&Str<'a>) -> bool,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
                })
                .with_ignore(),
                Classifier::predicate(move |token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        matcher(identifier)
                    } else {
                        false
                    }
                })
                .with_transform(move |former, classifier| {
                    let form = former.forms.get_mut(classifier.form).unwrap();
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    *form = Form::Input(Token::new(TokenKind::Identifier(name.clone()), span));

                    Ok(())
                }),
                Self::path_value(),
            ]),
            move |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let forms = form.as_forms();
                let identifier = forms[0].unwrap_input().clone();
                let path = forms[1].collect_inputs();

                let target = Element::new(
                    ElementKind::Literal(Token::new(
                        TokenKind::Identifier(name),
                        identifier.span().clone(),
                    )),
                    identifier.span(),
                );

                let value = Element::new(
                    ElementKind::Literal(Token::new(
                        TokenKind::Identifier(Str::from(Self::path_string(path.clone()))),
                        path.clone().span(),
                    )),
                    path.span(),
                );

                let symbol = Symbol::new(
                    SymbolKind::Binding(Binding::new(
                        Box::from(target),
                        Some(Box::new(value)),
                        None,
                        BindingKind::Meta,
                    )),
                    identifier.span().merge(&path.span()),
                    Visibility::Public,
                );

                *form = Form::output(symbol);

                Ok(())
            },
        )
    }

    pub fn verbosity<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
            })
            .with_ignore(),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    identifier == "v" || identifier == "verbosity"
                } else {
                    false
                }
            })
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let identifier = form.collect_inputs()[0].clone();
                let span = identifier.span();

                *form = Form::Input(Token::new(
                    TokenKind::Identifier(Str::from("Verbosity")),
                    span,
                ));

                Ok(())
            }),
            Classifier::predicate(|token: &Token| matches!(token.kind, TokenKind::Integer(_))),
        ])
        .with_transform(|former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let identifier = form.collect_inputs()[0].clone();
            let value = form.collect_inputs()[1].clone();
            let span = identifier.span.merge(&value.span);

            let target = Element::new(ElementKind::Literal(identifier.clone()), identifier.span);
            let value = Element::new(ElementKind::Literal(value.clone()), value.span);

            let symbol = Symbol::new(
                SymbolKind::Binding(Binding::new(
                    Box::from(target),
                    Some(Box::new(value)),
                    None,
                    BindingKind::Meta,
                )),
                span,
                Visibility::Public,
            );

            *form = Form::output(symbol);

            Ok(())
        })
    }

    pub fn input<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::path_directive(Str::from("Input"), |identifier| {
            identifier == "i" || identifier == "input"
        })
    }

    pub fn implicit_input<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Classifier::with_transform(Self::path_value(), |former, classifier| {
            let form = former.forms.get_mut(classifier.form).unwrap();
            let inputs = form.collect_inputs();

            if inputs.is_empty() {
                *form = Form::blank();
                return Ok(());
            }

            let span = inputs[0].clone().span();
            let value = Token::new(
                TokenKind::Identifier(Str::from(Self::path_string(inputs))),
                span,
            );

            let identifier = Token::new(TokenKind::Identifier(Str::from("Input")), span);

            let target = Element::new(ElementKind::Literal(identifier.clone()), identifier.span);
            let value = Element::new(ElementKind::Literal(value.clone()), value.span);

            let symbol = Symbol::new(
                SymbolKind::Binding(Binding::new(
                    Box::from(target),
                    Some(Box::new(value)),
                    None,
                    BindingKind::Meta,
                )),
                span,
                Visibility::Public,
            );

            *form = Form::output(symbol);

            Ok(())
        })
    }

    pub fn output<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Self::path_directive(Str::from("Output"), |identifier| {
            identifier == "o" || identifier == "output"
        })
    }
}
