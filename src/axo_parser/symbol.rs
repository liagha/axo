use {
    dynemit::{
        clone::DynClone,
        eq::DynEq,
        hash::DynHash,
        clone_trait_object, 
        eq_trait_object, 
        hash_trait_object,
    },
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
        format::Debug,
        operations::{Deref, DerefMut},
    },
    derive_ctor::ctor,
    derive_more::{
        IsVariant, Unwrap,
    },
};
use crate::axo_resolver::Branded;

pub trait Symbol: Branded<Token> + DynClone + DynEq + DynHash + Debug + Send + Sync {}

clone_trait_object!(Symbol);
eq_trait_object!(Symbol);
hash_trait_object!(Symbol);

pub type DynSymbol = Box<dyn Symbol>;

impl Symbol for Inclusion<Box<Element>> {}
impl Symbol for Implementation<Box<Element>, Box<Element>, DynSymbol> {}
impl Symbol for Interface<Box<Element>, DynSymbol> {}
impl Symbol for Binding<Box<Element>, Box<Element>, Box<Element>> {}
impl Symbol for Structure<Box<Element>, DynSymbol> {}
impl Symbol for Enumeration<Box<Element>, Element> {}
impl Symbol for Method<Box<Element>, DynSymbol, Box<Element>, Option<Box<Element>>> {}


impl Branded<Token> for Inclusion<Box<Element>> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
    }
}

impl Branded<Token> for Implementation<Box<Element>, Box<Element>, DynSymbol> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
    }
}

impl Branded<Token> for Interface<Box<Element>, DynSymbol> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
    }
}

impl Branded<Token> for Binding<Box<Element>, Box<Element>, Box<Element>> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
    }
}

impl Branded<Token> for Structure<Box<Element>, DynSymbol> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
    }
}

impl Branded<Token> for Enumeration<Box<Element>, Element> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
    }
}

impl Branded<Token> for Method<Box<Element>, DynSymbol, Box<Element>, Option<Box<Element>>> {
    fn brand(&self) -> Option<Token> {
        self.get_target().clone().brand()
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
                        item.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Box::new(Implementation::new(name.into(), None, members)),
                            ),
                            outputs.span()
                        )
                    ))
                } else if outputs.len() == 3 {
                    let target = outputs[1].clone();
                    let body = outputs[2].clone();
                    let members = <ElementKind as Clone>::clone(&body).unwrap_block().clone().items.iter().map(|item| {
                        item.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();
                    let span = Span::merge(&keyword.span(), &body.span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Symbolize(
                                Box::new(Implementation::new(name.into(), Some(target.into()), members)),
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
                            Binding::new(label.get_label().clone(), Some(assign.get_value().clone()), Some(label.get_element().clone()), mutable)
                        } else {
                            Binding::new(assign.get_target().clone(), Some(assign.get_value().clone()), None, mutable)
                        }
                    }

                    _ => {
                        Binding::new(body.into(), None, None, mutable)
                    }
                };

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(Box::new(symbol)),
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
                    item.kind.clone().unwrap_symbolize().clone()
                }).collect::<Vec<_>>();
                let span = Span::merge(&keyword.span(), &body.span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Box::new(Structure::new(name.into(), fields))
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
                            Box::new(Enumeration::new(name.into(), items))
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
                    parameter.kind.clone().unwrap_symbolize()
                }).collect::<Vec<_>>();

                let span = Span::merge(&keyword.span(), &body.span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Box::new(Method::new(name.into(), parameters, body.into(), None))
                        ),
                        span,
                    )
                ))
            }
        )
    }
}