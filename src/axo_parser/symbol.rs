use {
    super::{
        Element, ElementKind,
        ParseError, Parser
    },
    crate::{
        axo_cursor::{
            Span, Spanned,
        },
        axo_form::{
            form::Form,
            pattern::Classifier,
        },
        axo_scanner::{
            OperatorKind, Token
            ,
            TokenKind,
        },
        axo_schema::{
            Binding, Enumeration,
            Implementation, Inclusion, Interface, Method, Structure
        },
        hash::Hash,
        operations::{Deref, DerefMut},
    },
    derive_ctor::ctor,
    derive_more::{
        IsVariant, Unwrap,
    },
};
pub struct Symbol {
    pub kind: SymbolKind,
    pub span: Span,
    pub members: Vec<Symbol>,
}

#[derive(ctor, IsVariant, Unwrap)]
pub enum SymbolKind {
    Inclusion(Inclusion<Box<Element>>),
    Implementation(Implementation<Box<Element>, Box<Element>, Symbol>),
    Interface(Interface<Box<Element>, Symbol>),
    Binding(Binding<Box<Element>, Box<Element>, Box<Element>>),
    Structure(Structure<Box<Element>, Symbol>),
    Enumeration(Enumeration<Box<Element>, Element>),
    Method(Method<Box<Element>, Symbol, Box<Element>, Option<Box<Element>>>),
}

impl Symbol {
    pub fn new(kind: SymbolKind, span: Span) -> Symbol {
        Symbol { kind, span, members: vec![] }
    }
}

impl Parser {
    pub fn symbolization() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::implementation(),
            Self::binding(),
            Self::structure(),
            Self::enumeration(),
            Self::method(),
        ])
    }

    pub fn implementation() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "impl"
                    } else {
                        false
                    }
                }),
                Self::token(),
                Classifier::optional(
                    Classifier::sequence([
                        Classifier::predicate(|token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                *operator == OperatorKind::Colon
                            } else {
                                false
                            }
                        }),
                        Self::token(),
                    ])
                ),
                Self::block(Classifier::deferred(Self::symbolization))
            ]),
            |_, form| {
                let keyword = form.collect_inputs()[0].clone();
                let outputs = form.collect_outputs().clone();

                let name = outputs[0].clone();

                if outputs.len() == 2 {
                    let body = outputs[1].clone();
                    let members = body.kind.clone().unwrap_block().items.iter().map(|item| {
                        Symbol {
                            kind: item.kind.clone().unwrap_symbolize().clone().kind,
                            span: item.span,
                            members: vec![]
                        }.into()
                    }).collect::<Vec<_>>();
                    let span = Span::merge(&keyword.span(), &body.span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol {
                                    kind: SymbolKind::Implementation(Implementation::new(name.into(), None, members)),
                                    span,
                                    members: vec![],
                                },
                            ),
                            outputs.span()
                        )
                    ))
                } else if outputs.len() == 3 {
                    let target = outputs[1].clone();
                    let body = outputs[2].clone().kind.unwrap_block();
                    let members = body.items.iter().map(|item| {
                        Symbol {
                            kind: item.kind.clone().unwrap_symbolize().clone().kind,
                            span: item.span,
                            members: vec![]
                        }.into()
                    }).collect::<Vec<_>>();
                    let span = Span::merge(&keyword.span(), &members.span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol {
                                    kind: SymbolKind::Implementation(Implementation::new(name.into(), Some(target.into()), members)),
                                    span,
                                    members: vec![],
                                },
                            ),
                            outputs.span()
                        )
                    ))
                } else {
                    unreachable!()
                }
            },
        )

    }
    
    pub fn binding() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "var" || identifier == "const"
                    } else {
                        false
                    }
                }),
                Classifier::deferred(Self::element),
            ]),
            |_, form| {
                let sequence = form.as_forms();

                let keyword = sequence[0].unwrap_input();
                let mutable = if let TokenKind::Identifier(identifier) = &keyword.kind {
                    identifier == "var"
                } else {
                    false
                };

                let body = sequence[1].unwrap_output().clone();

                let span = Span::merge(&keyword.span(), &body.span());

                let symbol = match body.kind {
                    ElementKind::Assign(assign) => {
                        if let ElementKind::Label(label) = assign.get_target().kind.clone() {
                            Symbol {
                                kind: SymbolKind::Binding(Binding::new(label.get_label().clone(), Some(assign.get_value().clone()), Some(label.get_element().clone()), mutable)),
                                span,
                                members: vec![],
                            }
                        } else {
                            Symbol {
                                kind: SymbolKind::Binding(Binding::new(assign.get_target().clone(), Some(assign.get_value().clone()), None, mutable)),
                                span,
                                members: vec![],
                            }
                        }
                    }

                    _ => {
                        Symbol {
                            kind: SymbolKind::binding(Binding::new(body.into(), None, None, mutable)),
                            span,
                            members: vec![],
                        }
                    }
                };

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(symbol),
                        span,
                    )
                ))
            },
        )
    }

    pub fn structure() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "struct"
                    } else {
                        false
                    }
                }),
                Self::token(),
                Self::bundle(Classifier::deferred(Self::symbolization)),
            ]),
            |_, form| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();

                let fields = body.kind.clone().unwrap_bundle().items.iter().map(|item| {
                    Symbol {
                        kind: item.kind.clone().unwrap_symbolize().clone().kind,
                        span: item.span(),
                        members: vec![]
                    }
                }).collect::<Vec<_>>();
                let span = Span::merge(&keyword.span(), &body.span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol {
                                kind: SymbolKind::Structure(Structure::new(name.into(), fields)),
                                span,
                                members: vec![],
                            },
                        ),
                        span,
                    )
                ))
            }
        )
    }

    pub fn enumeration() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "enum"
                    } else {
                        false
                    }
                }),
                Self::token(),
                Self::bundle(Classifier::deferred(Self::element)),
            ]),
            |_, form| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();
                let span = Span::merge(&keyword.span(), &body.span());
                let items = body.kind.unwrap_bundle().items;

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol {
                                kind: SymbolKind::Enumeration(Enumeration::new(name.into(), items)),
                                span,
                                members: vec![],
                            },
                        ),
                        span,
                    )
                ))
            }
        )
    }

    pub fn method() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "func"
                    } else {
                        false
                    }
                }),
                Self::token(),
                Self::group(Classifier::deferred(Self::symbolization)),
                Self::block(Classifier::deferred(Self::element)),
            ]),
            |_, form| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let invoke = sequence[2].unwrap_output().clone();
                let body = sequence[3].unwrap_output().clone();

                let parameters = invoke.kind.unwrap_group().items.iter().map(|parameter| {
                    Symbol {
                        kind: parameter.kind.clone().unwrap_symbolize().kind,
                        span: parameter.span(),
                        members: vec![]
                    }
                }).collect::<Vec<_>>();

                let span = Span::merge(&keyword.span(), &body.span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol {
                                kind: SymbolKind::Method(Method::new(name.into(), parameters, body.into(), None)),
                                span,
                                members: vec![],
                            }
                        ),
                        span,
                    )
                ))
            }
        )
    }
}

impl Deref for Symbol {
    type Target = SymbolKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl DerefMut for Symbol {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}