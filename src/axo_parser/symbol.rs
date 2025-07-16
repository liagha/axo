use {
    derive_ctor::ctor,
    derive_more::{
        IsVariant, Unwrap,
    },
    super::{
        error::ErrorKind,
        Element, ElementKind,
        ParseError, Parser
    },
    crate::{
        artifact::Artifact,
        operations::{Deref, DerefMut},
        hash::{
            Hash, Hasher
        },
        axo_form::{
            pattern::Classifier,
            form::Form,
        },
        axo_schema::{
            Group, Sequence,
            Collection, Series,
            Bundle, Block,
            Binary, Unary,
            Index, Invoke, Construct,
            Structure, Enumeration,
            Binding, Method, Interface, Implementation, Formation, Inclusion,
            Conditional, Repeat, Iterate,
            Label, Access, Assign,
        },
        axo_scanner::{
            Token, TokenKind,
            PunctuationKind,
            OperatorKind,
        },
        axo_cursor::{
            Span, Spanned,
        },
    },
};

pub struct Symbol {
    pub kind: SymbolKind,
    pub span: Span,
    pub members: Vec<Symbol>,
}


#[derive(ctor, IsVariant, Unwrap)]
pub enum SymbolKind {
    Formation(Formation),
    Inclusion(Inclusion<Box<Element>>),
    Implementation(Implementation<Box<Element>, Box<Element>, Box<Symbol>>),
    Interface(Interface<Box<Element>, Box<Element>>),
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
                Self::block(Classifier::lazy(Self::symbolization))
            ]),
            |_, form| {
                let outputs = form.outputs().clone();

                let name = outputs[0].clone();

                if outputs.len() == 2 {
                    let body = outputs[1].clone().kind.unwrap_block();
                    let members = body.items.iter().map(|item| {
                        Symbol {
                            kind: item.kind.clone().unwrap_symbolize().clone().kind,
                            span: item.span,
                            members: vec![]
                        }.into()
                    }).collect::<Vec<_>>();

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol {
                                    kind: SymbolKind::Implementation(Implementation::new(name.into(), None, members)),
                                    span: form.span,
                                    members: vec![],
                                },
                            ),
                            outputs.span()
                        )
                    ))
                } else {
                    let target = outputs[2].clone();
                    let body = outputs[3].clone().kind.unwrap_bundle();
                    let members = body.items.iter().map(|item| {
                        Symbol {
                            kind: item.kind.clone().unwrap_symbolize().clone().kind,
                            span: item.span,
                            members: vec![]
                        }.into()
                    }).collect::<Vec<_>>();

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Symbol {
                                    kind: SymbolKind::Implementation(Implementation::new(name.into(), Some(target.into()), members)),
                                    span: form.span,
                                    members: vec![],
                                },
                            ),
                            outputs.span()
                        )
                    ))
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
                Classifier::lazy(|| Self::primary()),
                Classifier::repetition(
                    Classifier::alternative([
                        Classifier::sequence([
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Operator(ref op) if op == &OperatorKind::Colon)
                            }),
                            Classifier::lazy(|| Self::element()),
                        ]),
                        Classifier::sequence([
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Operator(ref op) if op == &OperatorKind::Equal)
                            }),
                            Classifier::lazy(|| Self::element()),
                        ]),
                    ]),
                    0,
                    None,
                ),
            ]),
            |_, form| {
                let sequence = form.unwrap();

                let keyword = sequence[0].unwrap_input();
                let mutable = if let TokenKind::Identifier(identifier) = &keyword.kind {
                    identifier == "var"
                } else {
                    false
                };

                let target = sequence[1].unwrap_output();
                let operations = sequence[2].unwrap();

                let mut ty : Option<Box<Element>> = None;
                let mut value : Option<Box<Element>> = None;

                for operation in operations {
                    let op_sequence = operation.unwrap();
                    if op_sequence.len() >= 2 {
                        let operator = op_sequence[0].unwrap_input();
                        let operand = op_sequence[1].unwrap_output();

                        if let TokenKind::Operator(op) = &operator.kind {
                            match op {
                                OperatorKind::Colon => {
                                    ty = Some(operand.into());
                                }
                                OperatorKind::Equal => {
                                    value = Some(operand.into());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let symbol = Symbol {
                    kind: SymbolKind::Binding(Binding::new(target.into(), value, ty, mutable)),
                    span: form.span.clone(),
                    members: vec![],
                };

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(symbol),
                        form.span,
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
                Self::bundle(Classifier::lazy(Self::symbolization)),
            ]),
            |_, form| {
                let outputs = form.outputs().clone();

                let name = outputs[0].clone();
                let body = outputs[1].clone().kind.unwrap_bundle();
                let fields = body.items.iter().map(|item| {
                    Symbol {
                        kind: item.kind.clone().unwrap_symbolize().clone().kind,
                        span: item.span,
                        members: vec![]
                    }
                }).collect::<Vec<_>>();

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol {
                                kind: SymbolKind::Structure(Structure::new(name.into(), fields)),
                                span: form.span,
                                members: vec![],
                            },
                        ),
                        outputs.span()
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
                Self::bundle(Classifier::lazy(Self::element)),
            ]),
            |_, form| {
                let outputs = form.outputs().clone();

                let name = outputs[0].clone();

                let body = outputs[1].clone().kind.unwrap_bundle();

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol {
                                kind: SymbolKind::Enumeration(Enumeration::new(name.into(), body.items)),
                                span: form.span,
                                members: vec![],
                            },
                        ),
                        outputs.span()
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
                Self::group(Classifier::lazy(Self::symbolization)),
                Self::block(Classifier::lazy(Self::element)),
            ]),
            |_, form| {
                let outputs = form.outputs().clone();

                let name = outputs[0].clone();
                let invoke = outputs[1].clone().kind.unwrap_group().items;
                let parameters = invoke.iter().map(|parameter| {
                    Symbol {
                        kind: parameter.kind.clone().unwrap_symbolize().kind,
                        span: parameter.span,
                        members: vec![]
                    }
                }).collect::<Vec<_>>();

                let body = outputs[2].clone();

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol {
                                kind: SymbolKind::Method(Method::new(name.into(), parameters, body.into(), None)),
                                span: form.span,
                                members: vec![],
                            }
                        ),
                        outputs.span()
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