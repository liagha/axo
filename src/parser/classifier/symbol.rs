use {
    super::super::{Element, ElementKind, ParseError, Parser, Symbol, SymbolKind, Visibility},
    crate::{
        data::*,
        formation::{classifier::Classifier, form::Form},
        scanner::{OperatorKind, Token, TokenKind},
        tracker::{Span, Spanned},
    },
};

impl<'parser> Parser<'parser> {
    pub fn symbolization() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::binding(),
            Self::structure(),
            Self::enumeration(),
            Self::method(),
            Self::module(),
        ])
    }

    pub fn binding() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("var"))
                        || token.kind == TokenKind::Identifier(Str::from("const"))
                        || token.kind == TokenKind::Identifier(Str::from("generic"))
                }),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();

                let kind = if let TokenKind::Identifier(identifier) = keyword.kind {
                    match identifier.as_str().unwrap() {
                        "const" => {
                            BindingKind::Constant
                        }

                        "var" => {
                            BindingKind::Variable
                        }

                        "generic" => {
                            BindingKind::Generic
                        }

                        _ => BindingKind::Constant
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

                            if matches!(
                                binary.operator.kind,
                                TokenKind::Operator(OperatorKind::Colon)
                            ) {
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
                                    if let ElementKind::Binary(annotation_pair) = &body.kind.clone()
                                    {
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

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(
                        Symbol::new(
                            0,
                            SymbolKind::Binding(
                                Binding::new(
                                    Box::new(body),
                                    value,
                                    annotation,
                                    kind,
                                )
                            ),
                            span,
                            Visibility::Private,
                        )
                    ),
                    span,
                )))
            },
        )
    }

    pub fn structure() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>
    {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Identifier(Str::from("struct"))
                    }),
                    Self::literal(),
                ]),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
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
                        ElementKind::Literal(
                            Token {
                                kind: TokenKind::Identifier(identifier),
                                ..
                            }
                        ) => {
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
                        },
                        _ => {
                            None
                        }
                    })
                    .collect();

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(
                        Symbol::new(
                            0,
                            SymbolKind::Structure(Structure::new(Box::new(name), members)),
                            span,
                            visibility,
                        ),
                    ),
                    span,
                )))
            },
        )
    }

    pub fn enumeration() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Identifier(Str::from("enum"))
                    }),
                    Self::literal(),
                ]),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
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
                        ElementKind::Literal(
                            Token {
                                kind: TokenKind::Identifier(identifier),
                                ..
                            }
                        ) => {
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
                        },
                        _ => {
                            None
                        }
                    })
                    .collect();

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(
                        Symbol::new(
                            0,
                            SymbolKind::Enumeration(Structure::new(Box::new(name), members)),
                            span,
                            visibility,
                        ),
                    ),
                    span,
                )))
            },
        )
    }

    pub fn method() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("func"))
                }),
                Self::literal(),
                Self::group(
                    Classifier::alternative(
                        [
                            Classifier::deferred(Self::symbolization),
                            Classifier::predicate(|token: &Token| {
                                if let TokenKind::Identifier(_) = &token.kind {
                                    true
                                } else {
                                    false
                                }
                            }).with_transform(
                                |form: Form<
                                    'parser,
                                    Token<'parser>,
                                    Element<'parser>,
                                    ParseError<'parser>,
                                >| {
                                    let input = form.unwrap_input();

                                    Ok(Form::output(Element::new(
                                        ElementKind::literal(input.clone()),
                                        input.span,
                                    )))
                                },
                            ),
                        ]
                    )
                ),
                Classifier::sequence(
                    [
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
                        ]),
                    ]
                ).with_transform(|form| {
                    let output = form.as_forms();

                    Ok(output[0].clone())
                }).as_optional(),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let invoke = sequence[2].unwrap_output().clone();
                
                let entry = if let ElementKind::Literal(token) = &name.kind {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == &Str::from("main")
                    } else { false }
                } else { false };

                let mut visibility = Visibility::Private;
                let mut interface = Interface::Axo;

                let members: Vec<_> = Self::get_body(invoke.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        ElementKind::Literal(
                            Token {
                                kind: TokenKind::Identifier(identifier),
                                ..
                            }
                        ) => {
                            match identifier.as_str().unwrap().to_lowercase().as_str() {
                                "public" => {
                                    visibility = Visibility::Public;
                                }

                                "private" => {
                                    visibility = Visibility::Private;
                                }

                                "c" => {
                                    interface = Interface::C;
                                }

                                "axo" => {
                                    interface = Interface::Axo;
                                }

                                "compiler" => {
                                    interface = Interface::Compiler;
                                }

                                _ => {}
                            }

                            None
                        },
                        _ => {
                            None
                        }
                    })
                    .collect();

                if sequence.len() == 4 {
                    let body = sequence[3].unwrap_output().clone();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                0,
                                SymbolKind::Method(Function::new(
                                    Box::new(name),
                                    members,
                                    Box::new(body),
                                    None::<Box<Element<'parser>>>,
                                    interface,
                                    entry,
                                )),
                                span,
                                visibility,
                            ),
                        ),
                        span,
                    )))
                } else {
                    let output = sequence[3].unwrap_output().clone();

                    let body = sequence[4].unwrap_output().clone();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                0,
                                SymbolKind::Method(Function::new(
                                    Box::new(name),
                                    members,
                                    Box::new(body),
                                    Some(Box::new(output)),
                                    interface,
                                    entry,
                                )),
                                span,
                                visibility,
                            ),
                        ),
                        span,
                    )))
                }
            },
        )
    }

    pub fn module() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("module"))
                }),
                Self::literal(),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
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
                    Symbol::new(0, SymbolKind::Module(Module::new(Box::new(name))), span, Visibility::Private);
                symbol.scope.extend(fields);

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(symbol),
                    span,
                )))
            },
        )
    }
}
