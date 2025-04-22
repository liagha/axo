use {
    core::cmp::PartialEq,

    crate::{
        axo_lexer::{
            Token, TokenKind,
            KeywordKind, OperatorKind, PunctuationKind,
        },
        axo_parser::{
            error::ErrorKind,
            expression::{
                Expr, ExprKind, Expression,
            },
            delimiter::Delimiter,
            ItemKind, Parser, Primary, ParseError,
        },
        axo_span::Span,
        axo_errors::Error,
    }
};

pub trait ControlFlow {
    fn parse_match(&mut self) -> Expr;
    fn parse_conditional(&mut self) -> Expr;
    fn parse_loop(&mut self) -> Expr;
    fn parse_while(&mut self) -> Expr;
    fn parse_for(&mut self) -> Expr;
    fn parse_return(&mut self) -> Expr;
    fn parse_break(&mut self) -> Expr;
    fn parse_continue(&mut self) -> Expr;
}

impl ControlFlow for Parser {
    fn parse_match(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let target = self.parse_basic();

        let body = if let Some(Token { kind: TokenKind::Punctuation(PunctuationKind::LeftBrace), .. }) = self.peek() {
            let (exprs, span) = self.parse_delimited(
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                TokenKind::Punctuation(PunctuationKind::RightBrace),
                TokenKind::Punctuation(PunctuationKind::Comma),
                true,
                Parser::parse_complex
            );

            Expr { kind: ExprKind::Block(exprs), span }
        } else {
            self.parse_complex()
        };

        let end = body.span.end;

        let kind = ExprKind::Match {
            target: target.into(),
            body: body.into()
        };

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_conditional(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic();

        let then_branch = self.parse_statement();

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            let expr = self.parse_statement();
            let end = expr.span.end;

            (Some(expr.into()), end)
        } else {
            (None, then_branch.span.end)
        };

        let kind = ExprKind::Conditional {
            condition: condition.into(),
            then_branch: then_branch.into(),
            else_branch: else_branch.into()
        };

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_loop(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let body = self.parse_statement();

        let end = body.span.end;

        let kind = ExprKind::Loop { body: body.into() };

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_while(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic();

        let body = self.parse_statement();

        let end = body.span.end;

        let kind = ExprKind::While {
            condition: condition.into(),
            body: body.into()
        };

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_for(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic();

        let body = self.parse_statement();

        let end = body.span.end;

        let kind = ExprKind::For {
            clause: clause.into(),
            body: body.into()
        };

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }


    fn parse_return(&mut self) -> Expr {
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

        

        let kind = ExprKind::Return(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_break(&mut self) -> Expr {
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

        let kind = ExprKind::Break(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_continue(&mut self) -> Expr {
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

        let kind = ExprKind::Continue(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }
}