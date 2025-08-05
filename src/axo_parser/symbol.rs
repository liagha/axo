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
            classifier::Classifier,
        },
        axo_scanner::{
            OperatorKind, Token,
            TokenKind,
        },
        axo_schema::{
            Binding, Enumeration,
            Implementation, Inclusion, Interface, Method, Structure
        },
        hash::{Hash, Hasher},
        format::{Debug, Formatter},
        operations::{Deref, DerefMut},
    },
    derive_ctor::ctor,
    derive_more::{
        IsVariant, Unwrap,
    },
};

pub trait Symbolic: DynClone + DynEq + DynHash + Debug + Send + Sync {
    fn brand(&self) -> Option<Token<'_>>;
}

clone_trait_object!(Symbolic);
eq_trait_object!(Symbolic);
hash_trait_object!(Symbolic);

pub struct Symbol<'symbol> {
    pub value: Box<dyn Symbolic>,
    pub span: Span<'symbol>,
    pub members: Vec<Symbol<'symbol>>,
}

impl<'symbol> Symbol<'symbol> {
    pub fn new(value: impl Symbolic, span: Span<'symbol>) -> Self {
        Self {
            value: Box::new(value),
            span,
            members: Vec::new(),
        }
    }

    pub fn cast<Type: 'symbol + 'static>(&self) -> Option<&Type> {
        (*self.value).as_any().downcast_ref::<Type>()
    }
}

impl<'symbol> Clone for Symbol<'symbol> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            span: self.span.clone(),
            members: self.members.clone(),
        }
    }
}

impl<'symbol> Debug for Symbol<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> crate::format::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<'symbol> Eq for Symbol<'symbol> {}

impl<'symbol> Hash for Symbol<'symbol> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<'symbol> PartialEq for Symbol<'symbol> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Symbolic for Symbol<'static> {
    fn brand(&self) -> Option<Token<'_>> {
        self.value.brand()
    }
}

impl Symbolic for Inclusion<Box<Element<'static>>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl Symbolic for Implementation<Box<Element<'static>>, Box<Element<'static>>, Symbol<'static>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl Symbolic for Interface<Box<Element<'static>>, Symbol<'static>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl Symbolic for Binding<Box<Element<'static>>, Box<Element<'static>>, Box<Element<'static>>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl Symbolic for Structure<Box<Element<'static>>, Symbol<'static>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl Symbolic for Enumeration<Box<Element<'static>>, Element<'static>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl Symbolic for Method<Box<Element<'static>>, Symbol<'static>, Box<Element<'static>>, Option<Box<Element<'static>>>> {
    fn brand(&self) -> Option<Token<'_>> {
        self.get_target().clone().brand()
    }
}

impl<'parser> Parser<'parser> {
    pub fn symbolization() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::implementation(),
            Self::binding(),
            Self::structure(),
            Self::enumeration(),
            Self::method(),
        ])
    }

    pub fn implementation() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Identifier("impl".to_string())
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
                Self::block(Classifier::deferred(Self::symbolization))
            ]),
            |_, form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
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
                            ElementKind::symbolize(
                                Symbol::new(Implementation::new(Box::new(name), None, members), span),
                            ),
                            span
                        )
                    ))
                } else if outputs.len() == 3 {
                    let target = outputs[1].clone();
                    let body = outputs[2].clone();
                    let members = <ElementKind as Clone>::clone(&body).unwrap_block().clone().items.iter().map(|item| {
                        item.kind.clone().unwrap_symbolize()
                    }).collect::<Vec<_>>();
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::symbolize(
                                Symbol::new(Implementation::new(Box::new(name), Some(target.into()), members), span),
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
                    token.kind == TokenKind::Identifier("var".to_string())
                        || token.kind == TokenKind::Identifier("const".to_string())
                }),
                Classifier::deferred(Self::element),
            ]),
            |_, form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();
                let mutable = keyword.kind == TokenKind::Identifier("var".to_string());
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
                        ElementKind::symbolize(Symbol::new(symbol, span)),
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
                    token.kind == TokenKind::Identifier("struct".to_string())
                }),
                Self::token(),
                Self::bundle(Classifier::deferred(Self::symbolization)),
            ]),
            |_, form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
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
                        ElementKind::symbolize(
                            Symbol::new(Structure::new(Box::new(name), fields), span),
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
                    token.kind == TokenKind::Identifier("enum".to_string())
                }),
                Self::token(),
                Self::bundle(Classifier::deferred(Self::element)),
            ]),
            |_, form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let body = sequence[2].unwrap_output().clone();
                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());
                let items = body.kind.unwrap_bundle().items;

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(Enumeration::new(Box::new(name), items), span)
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
                    token.kind == TokenKind::Identifier("func".to_string())
                }),
                Self::token(),
                Self::group(Classifier::deferred(Self::symbolization)),
                Self::block(Classifier::deferred(Self::element)),
            ]),
            |_, form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input().clone();
                let name = sequence[1].unwrap_output().clone();
                let invoke = sequence[2].unwrap_output().clone();
                let body = sequence[3].unwrap_output().clone();

                let parameters = invoke.kind.unwrap_group().items.iter().map(|parameter| {
                    parameter.kind.clone().unwrap_symbolize()
                }).collect::<Vec<_>>();

                let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                Ok(Form::output(
                    Element::new(
                        ElementKind::Symbolize(
                            Symbol::new(Method::new(Box::new(name), parameters, Box::new(body), None), span)
                        ),
                        span,
                    )
                ))
            }
        )
    }
}