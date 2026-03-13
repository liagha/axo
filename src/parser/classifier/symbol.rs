use {
    super::super::{Element, ElementKind, ParseError, Parser, Symbol, SymbolKind, Visibility},
    crate::{
        data::*,
        formation::{classifier::Classifier, form::Form},
        scanner::{OperatorKind, Token, TokenKind},
        parser::error::ErrorKind,
        tracker::{Span, Spanned},
    },
};

impl<'parser> Parser<'parser> {
    pub fn symbolization() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::binding(),
            Self::structure(),
            Self::union(),
            Self::function(),
            Self::module(),
        ])
    }

    pub fn binding() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(id) = &token.kind {
                    matches!(id.as_str().unwrap(), "var" | "const" | "meta")
                } else {
                    false
                }
            }),
            Self::expression().with_panic(
                |former, classifier| {
                    let consumed = classifier.consumed.iter().map(|index| former.consumed.get(*index).unwrap().clone()).collect::<Vec<_>>();
                    let span = consumed.span();

                    ParseError::new(ErrorKind::ExpectedBody, span)
                }
            ),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();

                let kind = if let TokenKind::Identifier(identifier) = keyword.kind {
                    match identifier.as_str().unwrap() {
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

                if let ElementKind::Binary(binary) = &body.kind.clone() {
                    match (&*binary.left, &binary.operator, &*binary.right) {
                        (
                            Element {
                                kind: ElementKind::Binary(binary),
                                ..
                            },
                            Token {
                                kind: TokenKind::Operator(OperatorKind::Equal),
                                ..
                            },
                            right,
                        ) => {
                            value = Some(Box::new(right.clone()));

                            if matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Colon)) {
                                body = *binary.left.clone();
                                annotation = Some(binary.right.clone());
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
                            if let ElementKind::Binary(assigned) = &binary.left.kind {
                                if matches!(assigned.operator.kind, TokenKind::Operator(OperatorKind::Equal)) {
                                    let merged_span =
                                        Span::merge(&assigned.right.borrow_span(), &binary.right.borrow_span());
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
                                    if let ElementKind::Binary(annotation_pair) = &body.kind.clone() {
                                        if matches!(
                                        annotation_pair.operator.kind,
                                        TokenKind::Operator(OperatorKind::Colon)
                                    ) {
                                            body = *annotation_pair.left.clone();
                                            annotation = Some(annotation_pair.right.clone());
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

    pub fn structure() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(id) = &token.kind {
                        id.as_str() == Some("struct")
                    } else {
                        false
                    }
                }),
                Self::literal().with_panic(
                    |former, classifier| {
                        let consumed = classifier.consumed.iter().map(|index| former.consumed.get(*index).unwrap().clone()).collect::<Vec<_>>();
                        let span = consumed.span();

                        ParseError::new(ErrorKind::ExpectedHead, span)
                    }
                ),
            ]),
            Self::expression().with_panic(
                |former, classifier| {
                    let consumed = classifier.consumed.iter().map(|index| former.consumed.get(*index).unwrap().clone()).collect::<Vec<_>>();
                    let span = consumed.span();

                    ParseError::new(ErrorKind::ExpectedBody, span)
                }
            ),
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
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        ElementKind::Literal(Token {
                                                 kind: TokenKind::Identifier(identifier),
                                                 ..
                                             }) => {
                            match identifier.as_str().unwrap().to_lowercase().as_str() {
                                "public" => {
                                    visibility = Visibility::Public;
                                }

                                "private" => {
                                    visibility = Visibility::Private;
                                }

                                _ => {}
                            }

                            None
                        }
                        _ => None,
                    })
                    .collect();

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Symbol::new(
                        SymbolKind::Structure(Structure::new(Box::new(name), members)),
                        span,
                        visibility,
                    )),
                    span,
                ));

                Ok(())
            })
    }

    pub fn union() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::sequence([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(id) = &token.kind {
                        id.as_str() == Some("union")
                    } else {
                        false
                    }
                }),
                Self::literal().with_panic(
                    |former, classifier| {
                        let consumed = classifier.consumed.iter().map(|index| former.consumed.get(*index).unwrap().clone()).collect::<Vec<_>>();
                        let span = consumed.span();

                        ParseError::new(ErrorKind::ExpectedHead, span)
                    }
                ),
            ]),
            Self::expression().with_panic(
                |former, classifier| {
                    let consumed = classifier.consumed.iter().map(|index| former.consumed.get(*index).unwrap().clone()).collect::<Vec<_>>();
                    let span = consumed.span();

                    ParseError::new(ErrorKind::ExpectedBody, span)
                }
            ),
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
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        ElementKind::Literal(Token {
                                                 kind: TokenKind::Identifier(identifier),
                                                 ..
                                             }) => {
                            match identifier.as_str().unwrap().to_lowercase().as_str() {
                                "public" => {
                                    visibility = Visibility::Public;
                                }

                                "private" => {
                                    visibility = Visibility::Private;
                                }

                                _ => {}
                            }

                            None
                        }
                        _ => None,
                    })
                    .collect();

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                *form = Form::output(Element::new(
                    ElementKind::Symbolize(Symbol::new(
                        SymbolKind::Union(Structure::new(Box::new(name), members)),
                        span,
                        visibility,
                    )),
                    span,
                ));

                Ok(())
            })
    }

    pub fn function() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("func"))
                }),
                Self::literal().with_panic(
                    |former, classifier| {
                        let consumed = classifier.consumed.iter().map(|index| former.consumed.get(*index).unwrap().clone()).collect::<Vec<_>>();
                        let span = consumed.span();

                        ParseError::new(ErrorKind::ExpectedName, span)
                    }
                ),
                Self::group(
                    Classifier::alternative([
                        Classifier::deferred(Self::symbolization),
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Identifier(_))
                        }).with_transform(|former, classifier| {
                            let form = former.forms.get_mut(classifier.form).unwrap();
                            let input = form.unwrap_input();
                            *form = Form::output(Element::new(ElementKind::literal(input.clone()), input.span));
                            Ok(())
                        }),
                ])).with_panic(
                    |former, classifier| {
                        let stack = classifier.stack.iter().map(|index| former.forms.get(*index).unwrap().clone()).collect::<Vec<_>>();
                        let span = stack.span();

                        ParseError::new(ErrorKind::ExpectedHead, span)
                    }
                ),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            matches!(operator, OperatorKind::Colon)
                        } else {
                            false
                        }
                    })
                        .with_ignore(),
                    Classifier::alternative([
                        Self::prefixed(),
                        Self::literal(),
                    ]).with_panic(
                        |former, classifier| {
                            let stack = classifier.stack.iter().map(|index| former.forms.get(*index).unwrap().clone()).collect::<Vec<_>>();
                            let span = stack.span();

                            ParseError::new(ErrorKind::ExpectedAnnotation, span)
                        }
                    ),
                ])
                    .with_transform(|former, classifier| {
                        let form = former.forms.get_mut(classifier.form).unwrap();
                        let output = form.as_forms();
                        *form = output[0].clone();
                        
                        Ok(())
                    }),
                Self::expression().into_optional(),
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

                    let entry = if let ElementKind::Literal(token) = &name.kind {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == &Str::from("main")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let mut visibility = Visibility::Private;
                    let mut interface = Interface::Axo;

                    let members: Vec<_> = Self::get_body(invoke.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            ElementKind::Literal(Token {
                                                     kind: TokenKind::Identifier(identifier),
                                                     ..
                                                 }) => {
                                match identifier.as_str().unwrap() {
                                    "public" => visibility = Visibility::Public,
                                    "private" => visibility = Visibility::Private,
                                    "C" => interface = Interface::C,
                                    "Axo" => interface = Interface::Axo,
                                    "Compiler" => interface = Interface::Compiler,
                                    _ => {}
                                }
                                None
                            }
                            _ => None,
                        })
                        .collect();

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
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("func"))
                }),
                Self::literal(),
                Self::group(Classifier::alternative([
                    Classifier::deferred(Self::symbolization),
                    Classifier::predicate(|token: &Token| {
                        matches!(token.kind, TokenKind::Identifier(_))
                    })
                        .with_transform(|former, classifier| {
                            let form = former.forms.get_mut(classifier.form).unwrap();
                            let input = form.unwrap_input();

                            *form = Form::output(Element::new(ElementKind::literal(input.clone()), input.span));
                            
                            Ok(())
                        }),
                ])),
                Self::expression().into_optional(),
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

                    let entry = if let ElementKind::Literal(token) = &name.kind {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == &Str::from("main")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let mut visibility = Visibility::Private;
                    let mut interface = Interface::Axo;

                    let members: Vec<_> = Self::get_body(invoke.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            ElementKind::Literal(Token {
                                                     kind: TokenKind::Identifier(identifier),
                                                     ..
                                                 }) => {
                                match identifier.as_str().unwrap() {
                                    "public" => visibility = Visibility::Public,
                                    "private" => visibility = Visibility::Private,
                                    "C" => interface = Interface::C,
                                    "Axo" => interface = Interface::Axo,
                                    "Compiler" => interface = Interface::Compiler,
                                    _ => {}
                                }
                                None
                            }
                            _ => None,
                        })
                        .collect();

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
            Self::literal().with_panic(
                |former, classifier| {
                    let stack = classifier.stack.iter().map(|index| former.forms.get(*index).unwrap().clone()).collect::<Vec<_>>();
                    let span = stack.span();

                    ParseError::new(ErrorKind::ExpectedName, span)
                }
            ),
            Self::expression().with_panic(
                |former, classifier| {
                    let stack = classifier.stack.iter().map(|index| former.forms.get(*index).unwrap().clone()).collect::<Vec<_>>();
                    let span = stack.span();

                    ParseError::new(ErrorKind::ExpectedBody, span)
                }
            ),
        ])
            .with_transform(|former, classifier| {
                let form = former.forms.get_mut(classifier.form).unwrap();
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();

                let fields = Self::get_body(body.clone())
                    .iter()
                    .filter(|item| item.kind.is_symbolize())
                    .map(|item| item.kind.clone().unwrap_symbolize().clone())
                    .collect::<Vec<_>>();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());
                let mut symbol =
                    Symbol::new(SymbolKind::Module(Module::new(Box::new(name))), span, Visibility::Private);
                symbol.scope.extend(fields);

                *form = Form::output(Element::new(ElementKind::Symbolize(symbol), span));

                Ok(())
            })
    }
}
