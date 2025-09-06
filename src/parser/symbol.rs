use {
    super::{
        Element, ElementKind,
        ParseError, Parser,
        SymbolKind,
    },
    crate::{
        resolver::{
            scope::Scope,
            Id,
        },
        tracker::{
            Span, Spanned,
        },
        formation::{
            form::Form,
            classifier::Classifier,
        },
        scanner::{
            OperatorKind, Token,
            TokenKind,
        },
        schema::{
            Binding, Enumeration,
            Extension, Method, Structure, Module,
        },
        internal::hash::{Hash, Hasher},
        data::{memory, Str},
        format::{self, Show, Display, Debug, Formatter},
    },
};
use crate::internal::hash::Set;

pub struct Symbol<'symbol> {
    pub id: Id,
    pub kind: SymbolKind<'symbol>,
    pub span: Span<'symbol>,
    pub scope: Scope<'symbol>,
}

impl<'symbol> Symbol<'symbol> {
    pub fn new(value: SymbolKind<'symbol>, span: Span<'symbol>, id: Id) -> Self {
        Self {
            id,
            kind: value,
            span,
            scope: Scope::new(),
        }
    }

    pub fn with_members<I: IntoIterator<Item = Symbol<'symbol>>>(&self, members: I) -> Self {
        Self {
            scope: Scope { symbols: Set::from_iter(members), parent: None },
            id: self.id,
            ..self.clone()
        }
    }

    pub fn set_members(&mut self, members: Vec<Symbol<'symbol>>) {
        self.scope.symbols.extend(members);
    }

    pub fn with_scope(&mut self, scope: Scope<'symbol>) {
        self.scope = scope;
    }

    pub fn brand(&self) -> Option<Token<'symbol>> {
        self.kind.brand()
    }
}

impl<'parser> Parser<'parser> {
    pub fn symbolization() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::extension(),
            Self::binding(),
            Self::structure(),
            Self::enumeration(),
            Self::method(),
            Self::module(),
        ])
    }

    pub fn extension() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("extend"))
                }),
                Self::literal(),
                Classifier::optional(
                    Classifier::sequence([
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Operator(OperatorKind::Colon))
                        }),
                        Self::literal(),
                    ])
                ),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let keyword = form.collect_inputs()[0].clone();
                let outputs = form.collect_outputs().clone();
                let name = outputs[0].clone();

                if outputs.len() == 2 {
                    let body = outputs[1].clone();

                    let members: Vec<_> = Self::get_body(body.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => None,
                        })
                        .collect();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    SymbolKind::Extension(Extension::new(Box::new(name), None::<Box<Element<'parser>>>, members)),
                                    span,
                                    0
                                ),
                            ),
                            span
                        )
                    ))
                } else if outputs.len() == 3 {
                    let target = outputs[1].clone();
                    let body = outputs[2].clone();
                    let members: Vec<_> = Self::get_body(body.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => None,
                        })
                        .collect();
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    SymbolKind::Extension(Extension::new(Box::new(name), Some(Box::new(target)), members)),
                                    span,
                                    0
                                ),
                            ),
                            span
                        )
                    ))
                } else {
                    unreachable!()
                }
            },
        )
    }

    pub fn binding() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("var"))
                        || token.kind == TokenKind::Identifier(Str::from("const"))
                }),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();
                let constant = keyword.kind == TokenKind::Identifier(Str::from("const"));
                let body = sequence[1].unwrap_output().clone();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                let symbol = match body.kind {
                    ElementKind::Assign(assign) => {
                        if let ElementKind::Label(label) = assign.target.kind.clone() {
                            Binding::new(label.label.clone(), Some(assign.value.clone()), Some(label.element.clone()), constant)
                        } else {
                            Binding::new(assign.target.clone(), Some(assign.value.clone()), None, constant)
                        }
                    }

                    _ => {
                        Binding::new(Box::new(body), None, None, constant)
                    }
                };

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Binding(symbol),
                                span,
                                0
                            )
                        ),
                        span,
                    )
                ))
            },
        )
    }

    pub fn structure() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("struct"))
                }),
                Self::literal(),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();

                let members: Vec<_> = Self::get_body(body.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        _ => None,
                    })
                    .collect();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Structure(Structure::new(Box::new(name), members)),
                                span,
                                0
                            ),
                        ),
                        span,
                    )
                ))
            }
        )
    }

    pub fn enumeration() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("enum"))
                }),
                Self::literal(),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();

                let members: Vec<_> = Self::get_body(body.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        _ => None,
                    })
                    .collect();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Enumeration(Enumeration::new(Box::new(name), members)),
                                span,
                                0
                            ),
                        ),
                        span,
                    )
                ))
            }
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
                    Classifier::alternative([
                        Classifier::deferred(Self::symbolization),
                        Classifier::predicate(|token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                matches!(operator.as_slice(), [OperatorKind::Dot, OperatorKind::Dot, OperatorKind::Dot])
                            } else {
                                false
                            }
                        }).with_transform(|form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                            let variadic = form.unwrap_input();

                            Ok(Form::output(
                                Element::new(
                                    ElementKind::literal(
                                        variadic.clone()
                                    ),
                                    variadic.span
                                )
                            ))
                        })
                    ])),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            matches!(operator.as_slice(), [OperatorKind::Minus, OperatorKind::RightAngle])
                        } else {
                            false
                        }
                    }).with_ignore(),
                    Self::literal(),
                ]).with_transform(|form| {
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

                if sequence.len() == 4 {
                    let mut variadic = false;
                    let body = sequence[3].unwrap_output().clone();

                    let members: Vec<_> = Self::get_body(body.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => {
                                variadic = true;
                                
                                None
                            },
                        })
                        .collect();
                    
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    SymbolKind::Method(Method::new(Box::new(name), members, Box::new(body), None::<Box<Element<'parser>>>, variadic)),
                                    span,
                                    0
                                ),
                            ),
                            span,
                        )
                    ))
                } else {
                    let output = sequence[3].unwrap_output().clone();
                    let body = sequence[4].unwrap_output().clone();
                    let mut variadic = false;

                    let members: Vec<_> = Self::get_body(body.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => {
                                variadic = true;

                                None
                            },
                        })
                        .collect();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    SymbolKind::Method(Method::new(Box::new(name), members, Box::new(body), Some(Box::new(output)), variadic)),
                                    span,
                                    0
                                )
                            ),
                            span,
                        )
                    ))
                }
            }
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

                let fields = Self::get_body(body.clone()).iter().filter(|item| item.kind.is_symbolize()).map(|item| {
                    item.kind.clone().unwrap_symbolize().clone()
                }).collect::<Vec<_>>();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());
                let mut symbol = Symbol::new(
                    SymbolKind::Module(Module::new(Box::new(name))),
                    span,
                    0
                );
                symbol.scope.extend(fields);

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            symbol,
                        ),
                        span,
                    )
                ))
            }
        )
    }
}