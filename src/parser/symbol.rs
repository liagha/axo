use {
    super::{
        Element, ElementKind,
        ParseError, Parser,
        Symbolic,
    },
    crate::{
        resolver::{
            scope::Scope,
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
            Implementation, Method, Structure, Module,
        },
        internal::hash::{Hash, Hasher},
        data::{memory, string::Str},
        format::{self, Show, Display, Debug, Formatter},
    },
};

pub struct Symbol<'symbol: 'static> {
    pub value: Box<dyn Symbolic<'symbol>>,
    pub span: Span<'symbol>,
    pub scope: Scope<'symbol>,
}

impl<'symbol: 'static> Symbol<'symbol> {
    pub fn new(value: impl Symbolic<'symbol> + 'symbol, span: Span<'symbol>) -> Self {
        Self {
            value: Box::new(value),
            span,
            scope: Scope::new(),
        }
    }

    pub fn with_members(&mut self, members: Vec<Symbol>) {
        self.scope.symbols.extend(members);
    }

    pub fn with_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    pub fn cast<Type: 'symbol>(&self) -> Option<&Type> {
        self.value.as_ref().as_any().downcast_ref::<Type>()
    }
}

impl Clone for Symbol<'_> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            span: self.span.clone(),
            scope: self.scope.clone(),
        }
    }
}

impl Debug for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self.value)?;

        if !self.scope.empty() {
            write!(f, "\n{}", self.scope.symbols.indent())
        } else {
            write!(f, "")
        }
    }
}

impl Display for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self)
    }
}

impl Eq for Symbol<'_> {}

impl Hash for Symbol<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Symbol<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value.clone()
    }
}

impl<'parser: 'static> Parser<'parser> {
    pub fn symbolization() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::implementation(),
            Self::binding(),
            Self::structure(),
            Self::enumeration(),
            Self::method(),
            Self::module(),
        ])
    }

    pub fn implementation() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("impl"))
                }),
                Self::token(),
                Classifier::optional(
                    Classifier::sequence([
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Operator(OperatorKind::Colon))
                        }),
                        Self::token(),
                    ])
                ),
                Classifier::deferred(Self::symbolization),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let keyword = form.collect_inputs()[0].clone();
                let outputs = form.collect_outputs().clone();
                let name = outputs[0].clone();

                if outputs.len() == 2 {
                    let body = outputs[1].clone();
                    let members = body.kind.clone().unwrap_block().items.iter().map(|item| {
                        item.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    Implementation::new(Box::new(name), None::<Box<Element<'static>>>, members),
                                    span
                                ),
                            ),
                            span
                        )
                    ))
                } else if outputs.len() == 3 {
                    let target = outputs[1].clone();
                    let body = outputs[2].clone();
                    let members = <ElementKind as Clone>::clone(&body.kind).unwrap_block().clone().items.iter().map(|item| {
                        item.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    Implementation::new(Box::new(name), Some(Box::new(target)), members),
                                    span,
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
                let mutable = keyword.kind == TokenKind::Identifier(Str::from("var"));
                let body = sequence[1].unwrap_output().clone();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                let symbol = match body.kind {
                    ElementKind::Assign(assign) => {
                        if let ElementKind::Label(label) = assign.get_target().kind.clone() {
                            Binding::new(label.get_label().clone(), Some(assign.get_value().clone()), Some(label.get_element().clone()), mutable)
                        } else {
                            Binding::new(assign.get_target().clone(), Some(assign.get_value().clone()), None, mutable)
                        }
                    }

                    _ => {
                        Binding::new(Box::new(body), None, None, mutable)
                    }
                };

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                symbol,
                                span
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
                Self::token(),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();

                let fields = body.kind.clone().unwrap_bundle().items.iter().map(|item| {
                    item.kind.clone().unwrap_symbolize().clone()
                }).collect::<Vec<_>>();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                Structure::new(Box::new(name), fields),
                                span,
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
                Self::token(),
                Classifier::deferred(Self::symbolization),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());
                let items = body.kind.unwrap_bundle().items;

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                Enumeration::new(Box::new(name), items),
                                span,
                            )
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
                Self::token(),
                Self::group(Classifier::deferred(Self::symbolization)),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            matches!(operator.as_slice(), [OperatorKind::Minus, OperatorKind::RightAngle])
                        } else {
                            false
                        }
                    }).with_ignore(),
                    Self::token(),
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
                    let body = sequence[3].unwrap_output().clone();

                    let parameters = invoke.kind.unwrap_group().items.iter().map(|parameter| {
                        parameter.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    Method::new(Box::new(name), parameters, Box::new(body), None::<Box<Element<'parser>>>),
                                    span,
                                ),
                            ),
                            span,
                        )
                    ))
                } else {
                    let output = sequence[3].unwrap_output().clone();
                    let body = sequence[4].unwrap_output().clone();

                    let parameters = invoke.kind.unwrap_group().items.iter().map(|parameter| {
                        parameter.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol::new(
                                    Method::new(Box::new(name), parameters, Box::new(body), Some(Box::new(output))),
                                    span
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
                Self::token(),
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
                    Module::new(name),
                    span,
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