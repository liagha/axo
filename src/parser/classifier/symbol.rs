use {
    super::super::{Element, ElementKind, ParseError, Parser, Specifier, Symbol, SymbolKind},
    crate::{
        data::Str,
        formation::{classifier::Classifier, form::Form},
        resolver::scope::Scope,
        scanner::{OperatorKind, Token, TokenKind},
        tracker::{Span, Spanned},
    },
};
use crate::data::*;

impl<'parser> Parser<'parser> {
    fn is_type_annotation(element: &Element<'parser>) -> bool {
        matches!(
            element.kind,
            ElementKind::Literal(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) if name == Str::from("Type")
        )
    }

    fn split_generic_members(
        members: Vec<Symbol<'parser>>,
    ) -> (Vec<Symbol<'parser>>, Scope<'parser>) {
        let mut runtime = Vec::new();
        let mut generic = Scope::new();
        for member in members {
            let is_generic = match &member.kind {
                SymbolKind::Binding(binding) => {
                    binding.constant
                        && binding
                            .annotation
                            .as_ref()
                            .is_some_and(|annotation| Self::is_type_annotation(annotation))
                }
                _ => false,
            };

            if is_generic {
                generic.add(member);
            } else {
                runtime.push(member);
            }
        }

        (runtime, generic)
    }

    pub fn symbolization(
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::inclusion(),
            Self::extension(),
            Self::binding(),
            Self::structure(),
            Self::enumeration(),
            Self::method(),
            Self::module(),
        ])
    }

    pub fn inclusion() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>
    {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                token.kind == TokenKind::Identifier(Str::from("use"))
            }),
            Classifier::deferred(Self::element),
        ])
        .with_transform(
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let keyword = form.collect_inputs()[0].clone();
                let inclusion = form.collect_outputs();

                let span = Span::merge(&keyword.span, &inclusion.clone().span());

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(Symbol::new(
                        SymbolKind::Inclusion(Inclusion::new(Box::new(inclusion[0].clone()), 0)),
                        span,
                        0,
                    )),
                    span,
                )))
            },
        )
    }

    pub fn extension() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>
    {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier(Str::from("extend"))
                }),
                Self::literal(),
                Classifier::optional(Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        matches!(token.kind, TokenKind::Operator(OperatorKind::Colon))
                    }),
                    Self::literal(),
                ])),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let keyword = form.collect_inputs()[0].clone();
                let outputs = form.collect_outputs().clone();
                let name = outputs[0].clone();

                if outputs.len() == 2 {
                    let body = outputs[1].clone();

                    let parsed_members: Vec<_> = Self::get_body(body.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => None,
                        })
                        .collect();
                    let (members, generic) = Self::split_generic_members(parsed_members);

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Extension(Extension::new(
                                    Box::new(name),
                                    None::<Box<Element<'parser>>>,
                                    members,
                                )),
                                span,
                                0,
                            )
                            .with_generic(generic),
                        ),
                        span,
                    )))
                } else if outputs.len() == 3 {
                    let target = outputs[1].clone();
                    let body = outputs[2].clone();
                    let parsed_members: Vec<_> = Self::get_body(body.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => None,
                        })
                        .collect();
                    let (members, generic) = Self::split_generic_members(parsed_members);
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Extension(Extension::new(
                                    Box::new(name),
                                    Some(Box::new(target)),
                                    members,
                                )),
                                span,
                                0,
                            )
                            .with_generic(generic),
                        ),
                        span,
                    )))
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
                            // Normalize cases like `x: T = a || b` where precedence can produce
                            // `((x: T = a) || b)` so bindings still capture target/annotation/value.
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
                    ElementKind::Symbolize(Symbol::new(
                        SymbolKind::Binding(Binding::new(
                            Box::new(body),
                            value,
                            annotation,
                            constant,
                        )),
                        span,
                        0,
                    )),
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
                let mut specifier = Specifier::default();

                let keyword = head[0].unwrap_input();
                let name = head[1].unwrap_output().clone();

                let body = sequence[1].unwrap_output().clone();

                let parsed_members: Vec<_> = Self::get_body(body.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        _ => {
                            specifier.apply(element.clone());

                            None
                        }
                    })
                    .collect();
                let (members, generic) = Self::split_generic_members(parsed_members);
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(
                        Symbol::new(
                            SymbolKind::Structure(Structure::new(Box::new(name), members)),
                            span,
                            0,
                        )
                        .with_specifier(specifier)
                        .with_generic(generic),
                    ),
                    span,
                )))
            },
        )
    }

    pub fn enumeration(
    ) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
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
                let generic = Scope::new();
                let mut specifier = Specifier::default();

                let keyword = head[0].unwrap_input();
                let name = head[1].unwrap_output().clone();

                let body = sequence[1].unwrap_output().clone();

                let members: Vec<_> = Self::get_body(body.clone())
                    .into_iter()
                    .filter_map(|element| match element.kind {
                        ElementKind::Symbolize(symbol) => Some(symbol),
                        _ => {
                            specifier.apply(element.clone());

                            None
                        }
                    })
                    .collect();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(
                        Symbol::new(
                            SymbolKind::Enumeration(Structure::new(Box::new(name), members)),
                            span,
                            0,
                        )
                        .with_specifier(specifier)
                        .with_generic(generic),
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
                Self::group(Classifier::alternative([
                    Classifier::deferred(Self::symbolization),
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            matches!(
                                operator.as_slice(),
                                [OperatorKind::Dot, OperatorKind::Dot, OperatorKind::Dot]
                            )
                        } else {
                            false
                        }
                    })
                    .with_transform(
                        |form: Form<
                            'parser,
                            Token<'parser>,
                            Element<'parser>,
                            ParseError<'parser>,
                        >| {
                            let variadic = form.unwrap_input();

                            Ok(Form::output(Element::new(
                                ElementKind::literal(variadic.clone()),
                                variadic.span,
                            )))
                        },
                    ),
                ])),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            matches!(operator, OperatorKind::Colon)
                        } else {
                            false
                        }
                    })
                    .with_ignore(),
                    Self::literal(),
                ])
                .with_transform(|form| {
                    let output = form.as_forms();

                    Ok(output[0].clone())
                })
                .as_optional(),
                Classifier::deferred(Self::element),
            ]),
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let invoke = sequence[2].unwrap_output().clone();

                if sequence.len() == 4 {
                    let mut variadic = false;

                    let parsed_members: Vec<_> = Self::get_body(invoke.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => {
                                variadic = true;

                                None
                            }
                        })
                        .collect();
                    let (members, generic) = Self::split_generic_members(parsed_members);
                    let body = sequence[3].unwrap_output().clone();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Method(Method::new(
                                    Box::new(name),
                                    members,
                                    Box::new(body),
                                    None::<Box<Element<'parser>>>,
                                    variadic,
                                )),
                                span,
                                0,
                            )
                            .with_generic(generic),
                        ),
                        span,
                    )))
                } else {
                    let output = sequence[3].unwrap_output().clone();
                    let mut variadic = false;

                    let parsed_members: Vec<_> = Self::get_body(invoke.clone())
                        .into_iter()
                        .filter_map(|element| match element.kind {
                            ElementKind::Symbolize(symbol) => Some(symbol),
                            _ => {
                                variadic = true;

                                None
                            }
                        })
                        .collect();
                    let (members, generic) = Self::split_generic_members(parsed_members);
                    let body = sequence[4].unwrap_output().clone();

                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(
                                SymbolKind::Method(Method::new(
                                    Box::new(name),
                                    members,
                                    Box::new(body),
                                    Some(Box::new(output)),
                                    variadic,
                                )),
                                span,
                                0,
                            )
                            .with_generic(generic),
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
                    Symbol::new(SymbolKind::Module(Module::new(Box::new(name))), span, 0);
                symbol.scope.extend(fields);

                Ok(Form::output(Element::new(
                    ElementKind::Symbolize(symbol),
                    span,
                )))
            },
        )
    }
}
