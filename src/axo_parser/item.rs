use crate::axo_data::peekable::Peekable;
use {
    crate::hash::{
        Hash, Hasher
    },
    crate::{
        axo_lexer::{
            Token, TokenKind,
            PunctuationKind,
            OperatorKind,
        },
        axo_parser::{
            delimiter::Delimiter,
            error::ErrorKind,
            Element, ElementKind,
            ParseError, Parser, Primary
        },
        axo_span::Span,
    }
};

#[derive(Eq, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Eq, Clone)]
pub enum ItemKind {
    Use(Box<Element>),
    Implement {
        element: Box<Element>,
        body: Box<Element>
    },
    Trait {
        name: Box<Element>,
        body: Box<Element>
    },
    Variable {
        target: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
        mutable: bool,
    },
    Field {
        name: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
    },
    Structure {
        name: Box<Element>,
        fields: Vec<Item>
    },
    Enum {
        name: Box<Element>,
        body: Box<Element>,
    },
    Macro {
        name: Box<Element>,
        parameters: Vec<Element>,
        body: Box<Element>
    },
    Function {
        name: Box<Element>,
        parameters: Vec<Element>,
        body: Box<Element>
    },
    Unit,
}

pub trait ItemParser {
    fn parse_field(&mut self) -> Item;
    fn parse_use(&mut self) -> Element;
    fn parse_variable(&mut self) -> Element;
    fn parse_impl(&mut self) -> Element;
    fn parse_trait(&mut self) -> Element;
    fn parse_function(&mut self) -> Element;
    fn parse_macro(&mut self) -> Element;
    fn parse_enum(&mut self) -> Element;
    fn parse_struct(&mut self) -> Element;
}

impl ItemParser for Parser {
    fn parse_field(&mut self) -> Item {
        let Element { kind, span } = self.parse_complex();

        match kind {
            ElementKind::Assignment {
                target,
                value
            } => {
                let kind = if let Element {
                    kind: ElementKind::Labeled { label, element }, ..
                } = *target {
                    ItemKind::Field { name: label, value: Some(value), ty: Some(element) }
                } else {
                    ItemKind::Field { name: target.into(), value: Some(value), ty: None }
                };

                Item {
                    kind,
                    span,
                }
            }

            ElementKind::Labeled {
                label, element
            } => {
                let kind = ItemKind::Field { name: label, value: None, ty: Some(element) };
                Item {
                    kind,
                    span,
                }
            }

            _ => {
                let element = Element {
                    kind,
                    span: span.clone(),
                };

                Item {
                    kind: ItemKind::Field { name: element.into(), value: None, ty: None },
                    span,
                }
            }
        }
    }

    fn parse_use(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = {
            let element = self.parse_complex();
            let end = element.span.end.clone();

            (element.into(), end)
        };

        let item = ItemKind::Use(value);
        let kind = ElementKind::Item(item);

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_variable(&mut self) -> Element {
        let Token {
            kind: def_kind,
            span: Span { start, .. },
        } = self.next().unwrap();

        let mutable = def_kind == TokenKind::Identifier("let".to_string());

        let element = self.parse_complex();

        let Element { kind, span: Span { end, .. } } = element.clone();

        let span = self.span(start, end);

        let item = match kind {
            ElementKind::Assignment { target, value } => {
                if let Element { kind: ElementKind::Labeled { label, element }, .. } = *target {
                    ItemKind::Variable {
                        target: label,
                        value: Some(value),
                        ty: Some(element),
                        mutable,
                    }
                } else {
                    ItemKind::Variable {
                        target,
                        value: Some(value),
                        ty: None,
                        mutable,
                    }
                }
            }
            _ => {
                ItemKind::Variable {
                    target: element.into(),
                    value: None,
                    ty: None,
                    mutable,
                }
            }
        };

        Element {
            kind: ElementKind::Item(item),
            span,
        }
    }

    fn parse_impl(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let implementation = self.parse_basic();

        

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let item = ItemKind::Implement { element: implementation.into(), body: body.into() };
        let kind = ElementKind::Item(item);

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_trait(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let trait_ = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let item = ItemKind::Trait {
            name: trait_.into(),
            body: body.into()
        };

        let kind = ElementKind::Item(item);

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_function(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let function = self.parse_basic();

        match function {
            Element {
                kind: ElementKind::Invoke { target, parameters },
                ..
            } => {
                let body = self.parse_complex();

                let end = body.span.end.clone();

                let item = ItemKind::Function {
                    name: target.into(),
                    parameters,
                    body: body.into()
                };

                let kind = ElementKind::Item(item);

                let element = Element {
                    kind,
                    span: self.span(start, end),
                };

                element
            }
            _ => {
                let body = self.parse_complex();

                let end = body.span.end.clone();

                let item = ItemKind::Function {
                    name: function.into(),
                    parameters: Vec::new(),
                    body: body.into()
                };

                let kind = ElementKind::Item(item);

                let element = Element {
                    kind,
                    span: self.span(start, end),
                };

                element
            }
        }
    }

    fn parse_macro(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let macro_ = self.parse_basic();

        match macro_ {
            Element {
                kind: ElementKind::Invoke { target, parameters},
                ..
            } => {
                let body = self.parse_complex();

                let end = body.span.end.clone();

                let item = ItemKind::Macro {
                    name: target.into(),
                    parameters,
                    body: body.into()
                };

                let kind = ElementKind::Item(item);

                let element = Element {
                    kind,
                    span: self.span(start, end),
                };

                element
            }
            _ => {
                let body = self.parse_complex();

                let end = body.span.end.clone();

                let item = ItemKind::Macro {
                    name: macro_.into(),
                    parameters: Vec::new(),
                    body: body.into()
                };

                let kind = ElementKind::Item(item);

                let element = Element {
                    kind,
                    span: self.span(start, end),
                };

                element
            }
        }
    }

    fn parse_enum(&mut self) -> Element {
        let enum_name = self.parse_basic();

        let Element {
            span: Span { start, .. },
            ..
        } = enum_name.clone();

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let kind = ElementKind::Constructor {
            name: enum_name.into(),
            body: body.into()
        };

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_struct(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let name = self.parse_token();

        let (fields, span) = self.parse_delimited(
              TokenKind::Punctuation(PunctuationKind::LeftBrace),
              TokenKind::Punctuation(PunctuationKind::RightBrace),
              &[TokenKind::Punctuation(PunctuationKind::Comma)],
              true,
              Parser::parse_field
        );

        let end = span.end;

        let item = ItemKind::Structure {
            name: name.into(),
            fields
        };

        let kind = ElementKind::Item(item);

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }
}