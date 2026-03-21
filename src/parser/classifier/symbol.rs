use {
    crate::{
        data::*,
        formation::{classifier::Classifier, form::Form},
        parser::{
            Element, ElementKind, ErrorKind, ParseError, Parser, Symbol, SymbolKind, Visibility,
        },
        scanner::{OperatorKind, Token, TokenKind},
        tracker::{Span, Spanned},
    },
};

impl<'parser> Parser<'parser> {
    pub fn symbolization() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Classifier::deferred(Self::binding),
            Classifier::deferred(Self::structure),
            Classifier::deferred(Self::union),
            Classifier::deferred(Self::function),
            Classifier::deferred(Self::module),
        ])
    }

    fn extract_visibility(
        elements: Vec<Element<'parser>>,
    ) -> (Visibility, Vec<Symbol<'parser>>) {
        let mut visibility = Visibility::Public;
        let mut members = Vec::with_capacity(elements.len());

        for element in elements {
            match element.kind {
                ElementKind::Symbolize(symbol) => members.push(symbol),
                ElementKind::Literal(Token {
                                         kind: TokenKind::Identifier(ref identifier),
                                         ..
                                     }) => match identifier.as_str().unwrap() {
                    "public" => visibility = Visibility::Public,
                    "private" => visibility = Visibility::Private,
                    _ => {}
                },
                _ => {}
            }
        }

        (visibility, members)
    }

    fn extract_interface(
        elements: Vec<Element<'parser>>,
    ) -> (Visibility, Interface, Vec<Symbol<'parser>>) {
        let mut visibility = Visibility::Private;
        let mut interface = Interface::Axo;
        let mut members = Vec::with_capacity(elements.len());

        for element in elements {
            match element.kind {
                ElementKind::Symbolize(symbol) => members.push(symbol),
                ElementKind::Literal(Token {
                                         kind: TokenKind::Identifier(ref identifier),
                                         ..
                                     }) => match identifier.as_str().unwrap() {
                    "public" => visibility = Visibility::Public,
                    "private" => visibility = Visibility::Private,
                    "C" => interface = Interface::C,
                    "Axo" => interface = Interface::Axo,
                    "Compiler" => interface = Interface::Compiler,
                    _ => {}
                },
                _ => {}
            }
        }

        (visibility, interface, members)
    }

    fn is_entry(name: &Element<'parser>) -> bool {
        if let ElementKind::Literal(token) = &name.kind {
            if let TokenKind::Identifier(identifier) = &token.kind {
                return identifier == &Str::from("main");
            }
        }
        false
    }

    pub fn binding() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(id) = &token.kind {
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

                let kind = if let TokenKind::Identifier(identifier) = keyword.kind {
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
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                let mut value = None;
                let mut annotation = None;

                if let ElementKind::Binary(ref binary) = body.kind.clone() {
                    match (&*binary.left, &binary.operator, &*binary.right) {
                        (
                            Element {
                                kind: ElementKind::Binary(inner),
                                ..
                            },
                            Token {
                                kind: TokenKind::Operator(OperatorKind::Equal),
                                ..
                            },
                            right,
                        ) => {
                            value = Some(Box::new(right.clone()));
                            if matches!(inner.operator.kind, TokenKind::Operator(OperatorKind::Colon))
                            {
                                body = *inner.left.clone();
                                annotation = Some(inner.right.clone());
                            }
                        }

                        (
                            left,
                            Token {
                                kind: TokenKind::Operator(OperatorKind::Equal),
                                ..
                            },
                            right,
                        ) => {
                            body = left.clone();
                            value = Some(Box::new(right.clone()));
                        }

                        (
                            left,
                            Token {
                                kind: TokenKind::Operator(OperatorKind::Colon),
                                ..
                            },
                            right,
                        ) => {
                            body = left.clone();
                            annotation = Some(Box::new(right.clone()));
                        }

                        _ => {
                            if let ElementKind::Binary(ref assigned) = binary.left.kind {
                                if matches!(
                                assigned.operator.kind,
                                TokenKind::Operator(OperatorKind::Equal)
                            ) {
                                    let merged_span = Span::merge(
                                        &assigned.right.borrow_span(),
                                        &binary.right.borrow_span(),
                                    );
                                    let merged_value = Element::new(
                                        ElementKind::Binary(Binary::new(
                                            assigned.right.clone(),
                                            binary.operator.clone(),
                                            binary.right.clone(),
                                        )),
                                        merged_span,
                                    );
                                    value = Some(Box::new(merged_value));
                                    body = *assigned.left.clone();

                                    if let ElementKind::Binary(ref pair) = body.kind.clone() {
                                        if matches!(
                                        pair.operator.kind,
                                        TokenKind::Operator(OperatorKind::Colon)
                                    ) {
                                            body = *pair.left.clone();
                                            annotation = Some(pair.right.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Symbol::new(
                        SymbolKind::Binding(Binding::new(Box::new(body), value, annotation, kind)),
                        span,
                        Visibility::Private,
                    )),
                    span,
                ));

                Ok(())
            })
    }

    fn aggregate_symbol(
        symbol_kind_fn: fn(Aggregate<Box<Element<'parser>>, Symbol<'parser>>) -> SymbolKind<'parser>,
        keyword_str: &'static str,
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::sequence([
                Classifier::predicate(move |token: &Token| {
                    if let TokenKind::Identifier(id) = &token.kind {
                        id.as_str() == Some(keyword_str)
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
            .with_transform(move |former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let head = sequence[0].as_forms();

                let keyword = head[0].unwrap_input();
                let name = head[1].unwrap_output().clone();
                let body = sequence[1].unwrap_output().clone();

                let body_elements = Self::get_body(body.clone());
                let (visibility, members) = Self::extract_visibility(body_elements);

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Symbol::new(
                        symbol_kind_fn(Aggregate::new(Box::new(name), members)),
                        span,
                        visibility,
                    )),
                    span,
                ));

                Ok(())
            })
    }

    pub fn structure() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::aggregate_symbol(SymbolKind::Structure, "struct")
    }

    pub fn union() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Self::aggregate_symbol(SymbolKind::Union, "union")
    }

    fn function_param_classifier() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Classifier::deferred(Self::symbolization),
            Classifier::predicate(|token: &Token| matches!(token.kind, TokenKind::Identifier(_)))
                .with_transform(|former, classifier| {
                    let form = former.forms.get_mut(classifier.form).unwrap();
                    let input = form.unwrap_input();
                    let span = input.span;
                    *form = Form::output(Element::new(ElementKind::literal(input.clone()), span));
                    Ok(())
                }),
        ])
    }

    pub fn function() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        let func_predicate = Classifier::predicate(|token: &Token| {
            token.kind == TokenKind::Identifier(Str::from("func"))
        });

        Classifier::alternative([
            Classifier::sequence([
                func_predicate.clone(),
                Classifier::deferred(Self::literal).with_panic(|former, classifier| {
                    let consumed = classifier
                        .consumed
                        .iter()
                        .map(|index| former.consumed.get(*index).unwrap().clone())
                        .collect::<Vec<_>>();
                    let span = consumed.span();
                    ParseError::new(ErrorKind::ExpectedName, span)
                }),
                Self::group(Self::function_param_classifier()).with_panic(
                    |former, classifier| {
                        let stack = classifier
                            .stack
                            .iter()
                            .map(|index| former.forms.get(*index).unwrap().clone())
                            .collect::<Vec<_>>();
                        let span = stack.span();
                        ParseError::new(ErrorKind::ExpectedHead, span)
                    },
                ),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        matches!(token.kind, TokenKind::Operator(OperatorKind::Colon))
                    })
                        .with_ignore(),
                    Classifier::alternative([
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
                ])
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
                    let return_type = sequence[3].unwrap_output().clone();

                    let body = if sequence.len() > 4 {
                        Some(Box::new(sequence[4].unwrap_output().clone()))
                    } else {
                        None
                    };

                    let entry = Self::is_entry(&name);
                    let body_elements = Self::get_body(invoke.clone());
                    let (visibility, interface, members) = Self::extract_interface(body_elements);

                    let span = if let Some(ref b) = body {
                        Span::merge(&keyword.borrow_span(), &b.borrow_span())
                    } else {
                        Span::merge(&keyword.borrow_span(), &return_type.borrow_span())
                    };

                    *form = Form::output(Element::new(
                        ElementKind::Symbolize(Symbol::new(
                            SymbolKind::Function(Function::new(
                                Box::new(name),
                                members,
                                body,
                                Some(Box::new(return_type)),
                                interface,
                                entry,
                            )),
                            span,
                            visibility,
                        )),
                        span,
                    ));
                    Ok(())
                }),
            Classifier::sequence([
                func_predicate,
                Classifier::deferred(Self::literal),
                Self::group(Self::function_param_classifier()),
                Classifier::deferred(Self::expression).into_optional(),
            ])
                .with_transform(|former, classifier| {
                    let form = former.forms.get_mut(classifier.form).unwrap();
                    let sequence = form.as_forms();
                    let keyword = sequence[0].unwrap_input().clone();
                    let name = sequence[1].unwrap_output().clone();
                    let invoke = sequence[2].unwrap_output().clone();

                    let body = if sequence.len() > 3 {
                        Some(Box::new(sequence[3].unwrap_output().clone()))
                    } else {
                        None
                    };

                    let entry = Self::is_entry(&name);
                    let body_elements = Self::get_body(invoke.clone());
                    let (visibility, interface, members) = Self::extract_interface(body_elements);

                    let span = if let Some(ref b) = body {
                        Span::merge(&keyword.borrow_span(), &b.borrow_span())
                    } else {
                        Span::merge(&keyword.borrow_span(), &invoke.borrow_span())
                    };

                    *form = Form::output(Element::new(
                        ElementKind::Symbolize(Symbol::new(
                            SymbolKind::Function(Function::new(
                                Box::new(name),
                                members,
                                body,
                                None::<Box<Element<'parser>>>,
                                interface,
                                entry,
                            )),
                            span,
                            visibility,
                        )),
                        span,
                    ));
                    Ok(())
                }),
        ])
    }

    pub fn module() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(id) = &token.kind {
                    id.as_str() == Some("module")
                } else {
                    false
                }
            }),
            Classifier::deferred(Self::literal).with_panic(|former, classifier| {
                let stack = classifier
                    .stack
                    .iter()
                    .map(|index| former.forms.get(*index).unwrap().clone())
                    .collect::<Vec<_>>();
                let span = stack.span();
                ParseError::new(ErrorKind::ExpectedName, span)
            }),
            Classifier::deferred(Self::expression).with_panic(|former, classifier| {
                let stack = classifier
                    .stack
                    .iter()
                    .map(|index| former.forms.get(*index).unwrap().clone())
                    .collect::<Vec<_>>();
                let span = stack.span();
                ParseError::new(ErrorKind::ExpectedBody, span)
            }),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());
                let symbol = Symbol::new(
                    SymbolKind::Module(Module::new(Box::new(name))),
                    span,
                    Visibility::Private,
                );

                *form = Form::output(Element::new(ElementKind::Symbolize(symbol), span));
                Ok(())
            })
    }
}
