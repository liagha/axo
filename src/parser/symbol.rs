use {
    super::{
        Element, ElementKind,
        ParseError, Parser,
        Symbolic,
    },
    crate::{
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
            Implementation, Method, Structure
        },
        internal::hash::{Hash, Hasher},
        data::memory,
        format::{self, Debug, Formatter},
    },
};

pub struct Symbol {
    pub value: Box<dyn Symbolic>,
    pub span: Span<'static>,
    pub members: Vec<Symbol>,
}

impl Symbol {
    pub fn new(value: impl Symbolic + 'static, span: Span<'static>) -> Self {
        Self {
            value: Box::new(value),
            span,
            members: Vec::new(),
        }
    }

    pub fn cast<Type: 'static>(&self) -> Option<&Type> {
        self.value.as_ref().as_any().downcast_ref::<Type>()
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            span: self.span.clone(),
            members: self.members.clone(),
        }
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self.value)
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value.clone()
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
                            ElementKind::Symbolize(
                                Symbol::new(unsafe { memory::transmute::<_, Implementation<Box<Element<'static>>, Box<Element<'static>>, Symbol>>(Implementation::new(Box::new(name), None::<Box<Element<'static>>>, members)) }, unsafe { memory::transmute(span) }),
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
                            ElementKind::Symbolize(
                                Symbol::new(unsafe { memory::transmute::<_, Implementation<Box<Element<'static>>, Box<Element<'static>>, Symbol>>(Implementation::new(Box::new(name), Some(Box::new(target)), members)) }, unsafe { memory::transmute(span) }),
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
                        ElementKind::Symbolize(Symbol::new(unsafe { memory::transmute::<_, Binding<Box<Element<'static>>, Box<Element<'static>>, Box<Element<'static>>>>(symbol) }, unsafe { memory::transmute(span) })),
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
                        ElementKind::Symbolize(
                            Symbol::new(unsafe { memory::transmute::<_, Structure<Box<Element<'static>>, Symbol>>(Structure::new(Box::new(name), fields)) }, unsafe { memory::transmute(span) }),
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
                            Symbol::new(unsafe { memory::transmute::<_, Enumeration<Box<Element<'static>>, Element<'static>>>(Enumeration::new(Box::new(name), items)) }, unsafe { memory::transmute(span) })
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
                            Symbol::new(unsafe { memory::transmute::<_, Method<Box<Element<'static>>, Symbol, Box<Element<'static>>, Option<Box<Element<'static>>>>>(Method::new(Box::new(name), parameters, Box::new(body), None::<Box<Element<'static>>>)) }, unsafe { memory::transmute(span) })
                        ),
                        span,
                    )
                ))
            }
        )
    }
}