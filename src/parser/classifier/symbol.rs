use crate::{
    combinator::{Classifier, Form},
    data::*,
    parser::{Element, ElementKind, ErrorKind, ParseError, Parser, Symbol, SymbolKind, Visibility},
    scanner::{OperatorKind, Token, TokenKind},
    tracker::{Span, Spanned},
};

impl<'a> Parser<'a> {
    pub fn symbolization<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Self::alternative([
            Classifier::deferred(Self::binding),
            Classifier::deferred(Self::structure),
            Classifier::deferred(Self::union),
            Classifier::deferred(Self::function),
        ])
    }

    pub fn binding<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let Some(id) = token.kind.try_unwrap_identifier() {
                    matches!(id.as_str().unwrap(), "static" | "var" | "const" | "meta")
                } else {
                    false
                }
            }),
            Classifier::deferred(Self::expression).with_panic(|former, classifier| {
                let consumed = classifier
                    .consumed
                    .iter()
                    .map(|index| former.consumed.get(*index).unwrap().clone())
                    .collect::<Vec<_>>();
                let span = consumed.span();

                ParseError::new(ErrorKind::ExpectedBody, span)
            }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();

                let kind = if let Some(identifier) = keyword.kind.try_unwrap_identifier() {
                    match identifier.as_str().unwrap() {
                        "static" => BindingKind::Static,
                        "const" => BindingKind::Constant,
                        "var" => BindingKind::Variable,
                        "meta" => BindingKind::Meta,
                        _ => BindingKind::Constant,
                    }
                } else {
                    BindingKind::Constant
                };

                let mut body = sequence[1].unwrap_output().clone();
                let span = Span::merge(&keyword.span(), &body.span());

                let mut value = None;
                let mut annotation = None;

                if let ElementKind::Binary(binary) = &body.kind.clone() {
                    if let Some(OperatorKind::Equal) = binary.operator.kind.try_unwrap_operator() {
                        if let ElementKind::Binary(inner_binary) = &binary.left.kind {
                            value = Some(binary.right.clone());
                            if let Some(OperatorKind::Colon) = inner_binary.operator.kind.try_unwrap_operator() {
                                body = inner_binary.left.clone();
                                annotation = Some(inner_binary.right.clone());
                            }
                        } else {
                            body = binary.left.clone();
                            value = Some(binary.right.clone());
                        }
                    } else if let Some(OperatorKind::Colon) = binary.operator.kind.try_unwrap_operator() {
                        body = binary.left.clone();
                        annotation = Some(binary.right.clone());
                    } else {
                        if let ElementKind::Binary(assigned) = &binary.left.kind {
                            if let Some(OperatorKind::Equal) = assigned.operator.kind.try_unwrap_operator() {
                                let merged_span =
                                    Span::merge(&assigned.right.span(), &binary.right.span());
                                let merged_value = Element::new(
                                    ElementKind::binary(Binary::new(
                                        assigned.right.clone(),
                                        binary.operator.clone(),
                                        binary.right.clone(),
                                    )),
                                    merged_span,
                                );
                                value = Some(merged_value);

                                body = assigned.left.clone();
                                if let ElementKind::Binary(annotation_pair) = &body.kind.clone() {
                                    if let Some(OperatorKind::Colon) = annotation_pair.operator.kind.try_unwrap_operator() {
                                        body = annotation_pair.left.clone();
                                        annotation = Some(annotation_pair.right.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Box::from(Symbol::new(
                        SymbolKind::binding(Binding::new(body, value, annotation, kind)),
                        span,
                        Visibility::Private,
                    ))),
                    span,
                ));

                Ok(())
            })
    }

    pub fn structure<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::sequence([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let Some(id) = token.kind.try_unwrap_identifier() {
                        id.as_str() == Some("struct")
                    } else {
                        false
                    }
                }),
                Classifier::deferred(Self::literal).with_panic(|former, classifier| {
                    let consumed = classifier
                        .consumed
                        .iter()
                        .map(|index| former.consumed.get(*index).unwrap().clone())
                        .collect::<Vec<_>>();
                    let span = consumed.span();

                    ParseError::new(ErrorKind::ExpectedHead, span)
                }),
            ]),
            Classifier::deferred(Self::expression).with_panic(|former, classifier| {
                let consumed = classifier
                    .consumed
                    .iter()
                    .map(|index| former.consumed.get(*index).unwrap().clone())
                    .collect::<Vec<_>>();
                let span = consumed.span();

                ParseError::new(ErrorKind::ExpectedBody, span)
            }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let head = sequence[0].as_forms();

                let keyword = head[0].unwrap_input();
                let name = head[1].unwrap_output().clone();

                let body = sequence[1].unwrap_output().clone();

                let mut visibility = Visibility::Public;

                let members: Vec<_> = Self::get_body(body.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(*symbol),
                        ElementKind::Literal(token) => {
                            if let Some(identifier) = token.kind.try_unwrap_identifier() {
                                match identifier.as_str().unwrap().to_lowercase().as_str() {
                                    "public" => {
                                        visibility = Visibility::Public;
                                    }

                                    "private" => {
                                        visibility = Visibility::Private;
                                    }

                                    _ => {}
                                }
                            }

                            None
                        }
                        _ => None,
                    })
                    .collect();

                let span = Span::merge(&keyword.span(), &body.span());

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Box::new(Symbol::new(
                        SymbolKind::structure(Aggregate::new(name, members)),
                        span,
                        visibility,
                    ))),
                    span,
                ));

                Ok(())
            })
    }

    pub fn union<'source>() -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>>
    {
        Classifier::sequence([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let Some(id) = token.kind.try_unwrap_identifier() {
                        id.as_str() == Some("union")
                    } else {
                        false
                    }
                }),
                Classifier::deferred(Self::literal).with_panic(|former, classifier| {
                    let consumed = classifier
                        .consumed
                        .iter()
                        .map(|index| former.consumed.get(*index).unwrap().clone())
                        .collect::<Vec<_>>();
                    let span = consumed.span();

                    ParseError::new(ErrorKind::ExpectedHead, span)
                }),
            ]),
            Classifier::deferred(Self::expression).with_panic(|former, classifier| {
                let consumed = classifier
                    .consumed
                    .iter()
                    .map(|index| former.consumed.get(*index).unwrap().clone())
                    .collect::<Vec<_>>();
                let span = consumed.span();

                ParseError::new(ErrorKind::ExpectedBody, span)
            }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let head = sequence[0].as_forms();

                let keyword = head[0].unwrap_input();
                let name = head[1].unwrap_output().clone();

                let body = sequence[1].unwrap_output().clone();

                let mut visibility = Visibility::Public;

                let members: Vec<_> = Self::get_body(body.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(*symbol),
                        ElementKind::Literal(token) => {
                            if let Some(identifier) = token.kind.try_unwrap_identifier() {
                                match identifier.as_str().unwrap().to_lowercase().as_str() {
                                    "public" => {
                                        visibility = Visibility::Public;
                                    }

                                    "private" => {
                                        visibility = Visibility::Private;
                                    }

                                    _ => {}
                                }
                            }

                            None
                        }
                        _ => None,
                    })
                    .collect();

                let span = Span::merge(&keyword.span(), &body.span());

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Box::from(Symbol::new(
                        SymbolKind::union(Aggregate::new(name, members)),
                        span,
                        visibility,
                    ))),
                    span,
                ));

                Ok(())
            })
    }

    pub fn function<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Self::alternative([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let Some(id) = token.kind.try_unwrap_identifier() {
                        *id == Str::from("func")
                    } else {
                        false
                    }
                }),
                Classifier::deferred(Self::literal).with_panic(|former, classifier| {
                    let consumed = classifier
                        .consumed
                        .iter()
                        .map(|index| former.consumed.get(*index).unwrap().clone())
                        .collect::<Vec<_>>();
                    let span = consumed.span();

                    ParseError::new(ErrorKind::ExpectedName, span)
                }),
                Self::group(Self::alternative([
                    Classifier::deferred(Self::symbolization),
                    Classifier::predicate(|token: &Token| {
                        if let Some(OperatorKind::Composite(operator)) = token.kind.try_unwrap_operator() {
                            operator.as_slice() == [OperatorKind::Dot, OperatorKind::Dot, OperatorKind::Dot]
                        } else {
                            false
                        }
                    }).with_transform(|former, classifier| {
                        let form = former.forms.get_mut(classifier.form).unwrap();
                        let span = form.unwrap_input().span();

                        *form = Form::output(Element::new(
                            ElementKind::literal(
                                Token::new(
                                    TokenKind::identifier(Str::from("Variadic")),
                                    span
                                )
                            ),
                            span,
                        ));

                        Ok(())
                    }),
                    Classifier::predicate(|token: &Token| token.kind.is_identifier())
                        .with_transform(|former, classifier| {
                            let form = former.forms.get_mut(classifier.form).unwrap();
                            let input = form.unwrap_input();
                            *form = Form::output(Element::new(
                                ElementKind::literal(input.clone()),
                                input.span,
                            ));
                            Ok(())
                        }),
                ]))
                    .with_panic(|former, classifier| {
                        let stack = classifier
                            .stack
                            .iter()
                            .map(|index| former.forms.get(*index).unwrap().clone())
                            .collect::<Vec<_>>();
                        let span = stack.span();

                        ParseError::new(ErrorKind::ExpectedHead, span)
                    }),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let Some(OperatorKind::Colon) = token.kind.try_unwrap_operator() {
                            true
                        } else {
                            false
                        }
                    })
                        .with_ignore(),
                    Self::alternative([
                        Classifier::deferred(Self::prefixed),
                        Classifier::deferred(Self::primary),
                    ])
                        .with_panic(|former, classifier| {
                            let stack = classifier
                                .stack
                                .iter()
                                .map(|index| former.forms.get(*index).unwrap().clone())
                                .collect::<Vec<_>>();
                            let span = stack.span();

                            ParseError::new(ErrorKind::ExpectedAnnotation, span)
                        }),
                ]).into_optional()
                    .with_transform(|former, classifier| {
                        let form = former.forms.get_mut(classifier.form).unwrap();
                        let output = form.as_forms();
                        *form = output[0].clone();

                        Ok(())
                    }),
                Classifier::deferred(Self::expression).into_optional(),
            ])
                .with_transform(|former, classifier| {
                    let form = former.forms.get_mut(classifier.form).unwrap();
                    let sequence = form.as_forms();
                    let keyword = sequence[0].unwrap_input().clone();
                    let name = sequence[1].unwrap_output().clone();
                    let invoke = sequence[2].unwrap_output().clone();
                    let output = if sequence.len() > 3 {
                        Some(sequence[3].unwrap_output().clone())
                    } else {
                        None
                    };

                    let body = if sequence.len() > 4 {
                        Some(sequence[4].unwrap_output().clone())
                    } else {
                        None
                    };

                    let entry = if let ElementKind::Literal(token) = &name.kind {
                        if let Some(identifier) = token.kind.try_unwrap_identifier() {
                            *identifier == Str::from("main")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let mut visibility = Visibility::Private;
                    let mut interface = Interface::Axo;
                    let mut variadic = false;

                    let members: Vec<_> = Self::get_body(invoke.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(*symbol),
                            ElementKind::Literal(token) => {
                                if let Some(identifier) = token.kind.try_unwrap_identifier() {
                                    match identifier.as_str().unwrap() {
                                        "public" => visibility = Visibility::Public,
                                        "private" => visibility = Visibility::Private,
                                        "C" => interface = Interface::C,
                                        "Axo" => interface = Interface::Axo,
                                        "Compiler" => interface = Interface::Compiler,
                                        "Variadic" => variadic = true,
                                        _ => {}
                                    }
                                }

                                None
                            }
                            _ => None,
                        })
                        .collect();

                    let span = if let Some(ref b) = body {
                        Span::merge(&keyword.span(), &b.span())
                    } else if let Some(ref output) = output {
                        Span::merge(&keyword.span(), &output.span())
                    } else {
                        keyword.span()
                    };

                    *form = Form::output(Element::new(
                        ElementKind::Symbolize(Box::from(Symbol::new(
                            SymbolKind::function(Function::new(
                                name,
                                members,
                                body,
                                output,
                                interface,
                                entry,
                                variadic,
                            )),
                            span,
                            visibility,
                        ))),
                        span,
                    ));
                    Ok(())
                }),
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let Some(id) = token.kind.try_unwrap_identifier() {
                        *id == Str::from("func")
                    } else {
                        false
                    }
                }),
                Classifier::deferred(Self::literal),
                Self::group(Self::alternative([
                    Classifier::deferred(Self::symbolization),
                    Classifier::predicate(|token: &Token| {
                        if let Some(OperatorKind::Composite(operator)) = token.kind.try_unwrap_operator() {
                            operator.as_slice() == [OperatorKind::Dot, OperatorKind::Dot, OperatorKind::Dot]
                        } else {
                            false
                        }
                    }).with_transform(|former, classifier| {
                        let form = former.forms.get_mut(classifier.form).unwrap();
                        let span = form.unwrap_input().span();

                        *form = Form::output(Element::new(
                            ElementKind::literal(
                                Token::new(
                                    TokenKind::identifier(Str::from("Variadic")),
                                    span
                                )
                            ),
                            span,
                        ));

                        Ok(())
                    }),
                    Classifier::predicate(|token: &Token| token.kind.is_identifier())
                        .with_transform(|former, classifier| {
                            let form = former.forms.get_mut(classifier.form).unwrap();
                            let input = form.unwrap_input();

                            *form = Form::output(Element::new(
                                ElementKind::literal(input.clone()),
                                input.span,
                            ));

                            Ok(())
                        }),
                ])),
                Classifier::deferred(Self::expression).into_optional(),
            ])
                .with_transform(|former, classifier| {
                    let form = former.forms.get_mut(classifier.form).unwrap();
                    let sequence = form.as_forms();
                    let keyword = sequence[0].unwrap_input().clone();
                    let name = sequence[1].unwrap_output().clone();
                    let invoke = sequence[2].unwrap_output().clone();

                    let body = if sequence.len() > 3 {
                        Some(sequence[3].unwrap_output().clone())
                    } else {
                        None
                    };

                    let entry = if let ElementKind::Literal(token) = &name.kind {
                        if let Some(identifier) = token.kind.try_unwrap_identifier() {
                            *identifier == Str::from("main")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let mut visibility = Visibility::Private;
                    let mut interface = Interface::Axo;
                    let mut variadic = false;

                    let members: Vec<_> = Self::get_body(invoke.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(*symbol),
                            ElementKind::Literal(token) => {
                                if let Some(identifier) = token.kind.try_unwrap_identifier() {
                                    match identifier.as_str().unwrap() {
                                        "public" => visibility = Visibility::Public,
                                        "private" => visibility = Visibility::Private,
                                        "C" => interface = Interface::C,
                                        "Axo" => interface = Interface::Axo,
                                        "Compiler" => interface = Interface::Compiler,
                                        "Variadic" => variadic = true,
                                        _ => {}
                                    }
                                }

                                None
                            }
                            _ => None,
                        })
                        .collect();

                    let span = if let Some(ref b) = body {
                        Span::merge(&keyword.span(), &b.span())
                    } else {
                        Span::merge(&keyword.span(), &invoke.span())
                    };

                    *form = Form::output(Element::new(
                        ElementKind::Symbolize(Box::from(Symbol::new(
                            SymbolKind::function(Function::new(
                                name,
                                members,
                                body,
                                None,
                                interface,
                                entry,
                                variadic,
                            )),
                            span,
                            visibility,
                        ))),
                        span,
                    ));
                    Ok(())
                }),
        ])
    }
}