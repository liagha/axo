use {
    crate::{
        axo_lexer::{
            OperatorKind, PunctuationKind,
            Token, TokenKind
        },
        axo_parser::{
            error::ErrorKind,
            delimiter::Delimiter,
            ParseError, Element, ElementKind,
            Parser, Primary, ControlFlow,
        },
        axo_span::Span,
    },
};

pub trait Composite {
    fn parse_index(&mut self, left: Element) -> Element;
    fn parse_invoke(&mut self, name: Element) -> Element;
    fn parse_constructor(&mut self, struct_name: Element) -> Element;
}

impl Composite for Parser {
    fn parse_index(&mut self, left: Element) -> Element {
        let index = self.parse_complex();

        let Element {
            span: Span { start, .. },
            ..
        } = left;

        let Element {
            span: Span { end, .. },
            ..
        } = index;

        let result = {
            let kind = ElementKind::Index {
                element: left.into(),
                index: index.into()
            };

            let span = self.span(start, end);
            let expr = Element::new(kind, span);

            expr
        };

        result
    }

    fn parse_invoke(&mut self, name: Element) -> Element {
        let Element {
            span: Span { start, .. },
            ..
        } = name;

        let parameters = self.parse_parenthesized();

        let result = match parameters {
            Element {
                kind: ElementKind::Group(parameters),
                span: Span { end, .. },
            } => {
                let kind = ElementKind::Invoke {
                    target: name.into(),
                    parameters
                };

                let expr = Element::new(
                    kind,
                    self.span(start, end),
                );

                expr
            }
            Element {
                span: Span { end, .. },
                ..
            } => {
                let kind = ElementKind::Invoke {
                    target: name.into(),
                    parameters: vec![parameters],
                };

                let expr = Element {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
        };

        result
    }

    fn parse_constructor(&mut self, struct_name: Element) -> Element {
        let Element {
            span: Span { start, .. },
            ..
        } = struct_name;

        let body = self.parse_complex();

        let end = body.span.end;

        let kind = ElementKind::Constructor {
            name: struct_name.into(),
            body: body.into()
        };

        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }
}
