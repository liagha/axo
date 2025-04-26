use {
    core::cmp::PartialEq,

    crate::{
        axo_lexer::{
            Token, TokenKind,
            KeywordKind, OperatorKind, PunctuationKind,
        },
        axo_parser::{
            error::ErrorKind,

            element::{
                Element,
                ElementKind,
            },

            expression::{
                Expression,
            },
            delimiter::Delimiter,
            ItemKind, Parser, Primary, ParseError,
        },
        axo_span::Span,
        axo_errors::Error,
    }
};

pub trait ControlFlow {
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
    fn parse_match(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let target = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end;

        let kind = ElementKind::Match {
            target: target.into(),
            body: body.into()
        };

        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_conditional(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic();

        let then_branch = self.parse_complex();

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            let expr = self.parse_complex();
            let end = expr.span.end;

            (Some(expr.into()), end)
        } else {
            (None, then_branch.span.end)
        };

        let kind = ElementKind::Conditional {
            condition: condition.into(),
            then: then_branch.into(),
            alternate: else_branch.into()
        };

        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_loop(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let body = self.parse_complex();

        let end = body.span.end;

        let kind = ElementKind::Loop { condition: None, body: body.into() };

        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_while(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end;

        let kind = ElementKind::Loop {
            condition: Some(condition.into()),
            body: body.into()
        };

        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_for(&mut self) -> Element {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic();

        let body = self.parse_complex();

        let end = body.span.end;

        let kind = ElementKind::Iterate {
            clause: clause.into(),
            body: body.into()
        };

        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }


    fn parse_return(&mut self) -> Element {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_complex();
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        

        let kind = ElementKind::Return(value);
        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_break(&mut self) -> Element {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_complex();
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        let kind = ElementKind::Break(value);
        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_continue(&mut self) -> Element {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_complex();
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        let kind = ElementKind::Skip(value);
        let expr = Element {
            kind,
            span: self.span(start, end),
        };

        expr
    }
}