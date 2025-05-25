use crate::axo_data::peekable::Peekable;
use {
    crate::compare::PartialEq,

    crate::{
        axo_lexer::{
            Token, TokenKind,
            OperatorKind, PunctuationKind,
        },
        axo_parser::{
            error::ErrorKind,

            element::{
                Element,
                ElementKind,
            },

            delimiter::Delimiter,
            ItemKind, Parser, Primary, ParseError,
        },
        axo_span::Span,
        axo_error::Error,
    }
};

pub trait ControlFlow {
    fn parse_procedural(&mut self) -> Element;
    fn parse_match(&mut self) -> Element;
    fn parse_conditional(&mut self) -> Element;
    fn parse_loop(&mut self) -> Element;
    fn parse_while(&mut self) -> Element;
    fn parse_for(&mut self) -> Element;
    fn parse_return(&mut self) -> Element;
    fn parse_break(&mut self) -> Element;
    fn parse_continue(&mut self) -> Element;
}

impl ControlFlow for Parser {
    fn parse_procedural(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let element = self.parse_complex();

        let end = element.span.end.clone();

        let kind = ElementKind::Procedural(element.into());

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element 
    }
    fn parse_match(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let target = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let kind = ElementKind::Match {
            target: target.into(),
            body: body.into()
        };

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_conditional(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic();

        let then_branch = self.parse_complex();

        let (else_branch, end) = if self.match_token(&TokenKind::Identifier("else".to_string())) {
            let element = self.parse_complex();
            let end = element.span.end.clone();

            (Some(element.into()), end)
        } else {
            (None, then_branch.span.end.clone())
        };

        let kind = ElementKind::Conditional {
            condition: condition.into(),
            then: then_branch.into(),
            alternate: else_branch.into()
        };

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_loop(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let kind = ElementKind::Loop { condition: None, body: body.into() };

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_while(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let kind = ElementKind::Loop {
            condition: Some(condition.into()),
            body: body.into()
        };

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_for(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end.clone();

        let kind = ElementKind::Iterate {
            clause: clause.into(),
            body: body.into()
        };

        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }


    fn parse_return(&mut self) -> Element {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::SemiColon))
        {
            (None, end)
        } else {
            let element = self.parse_complex();
            let end = element.span.end.clone();

            (Some(element.into()), end)
        };

        let kind = ElementKind::Return(value);
        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_break(&mut self) -> Element {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::SemiColon))
        {
            (None, end)
        } else {
            let element = self.parse_complex();
            let end = element.span.end.clone();

            (Some(element.into()), end)
        };

        let kind = ElementKind::Break(value);
        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }

    fn parse_continue(&mut self) -> Element {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::SemiColon))
        {
            (None, end)
        } else {
            let element = self.parse_complex();
            let end = element.span.end.clone();

            (Some(element.into()), end)
        };

        let kind = ElementKind::Skip(value);
        let element = Element {
            kind,
            span: self.span(start, end),
        };

        element
    }
}