use std::cmp::PartialEq;
use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_errors::Error as AxoError;
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::expression::{Expr, ExprKind, Expression};
use crate::axo_parser::{Error, ItemKind, Parser, Primary};
use crate::axo_parser::delimiter::Delimiter;
use crate::axo_parser::state::{Position, Context, ContextKind, SyntaxRole};

pub trait ControlFlow {
    fn parse_let(&mut self) -> Result<Expr, Error>;
    fn parse_match(&mut self) -> Result<Expr, Error>;
    fn parse_conditional(&mut self) -> Result<Expr, Error>;
    fn parse_while(&mut self) -> Result<Expr, Error>;
    fn parse_for(&mut self) -> Result<Expr, Error>;
    fn parse_return(&mut self) -> Result<Expr, Error>;
    fn parse_break(&mut self) -> Result<Expr, Error>;
    fn parse_continue(&mut self) -> Result<Expr, Error>;
}

impl ControlFlow for Parser {
    fn parse_let(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Variable, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let expr = self.parse_complex()?;

        let Expr { kind, span: Span { end, .. } } = expr.clone();

        let span = self.span(start, end);

        self.pop_context();

        match kind {
            ExprKind::Assignment(target, value) => {
                Ok(Expr { kind: ExprKind::Definition(target, Some(value)), span })
            }
            _ => {
                Ok(Expr { kind: ExprKind::Definition(expr.into(), None), span })
            }
        }
    }

    fn parse_match(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Match, Some(SyntaxRole::Clause));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic()?;

        self.pop_context();

        let body = if let Some(Token { kind: TokenKind::Punctuation(PunctuationKind::LeftBrace), .. }) = self.peek() {
            let (exprs, span) = self.parse_delimited(
                ContextKind::Match,
                Some(SyntaxRole::Body),
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                TokenKind::Punctuation(PunctuationKind::RightBrace),
                TokenKind::Punctuation(PunctuationKind::Comma),
                true,
                Parser::parse_complex
            )?;

            Expr { kind: ExprKind::Block(exprs), span }
        } else {
            self.parse_complex()?
        };

        let end = body.span.end;
        let kind = ExprKind::Match(clause.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_conditional(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::If, Some(SyntaxRole::Condition));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic()?;

        self.pop_context();

        self.push_context(ContextKind::If, Some(SyntaxRole::Then));

        let then_branch = self.parse_statement()?;

        self.pop_context();

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            self.push_context(ContextKind::If, Some(SyntaxRole::Else));

            let expr = self.parse_statement()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        } else {
            (None, then_branch.span.end)
        };

        let kind = ExprKind::Conditional(condition.into(), then_branch.into(), else_branch.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_while(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::While, Some(SyntaxRole::Condition));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic()?;

        self.pop_context();

        self.push_context(ContextKind::While, Some(SyntaxRole::Body));

        let body = self.parse_statement()?;

        self.pop_context();

        let end = body.span.end;
        let kind = ExprKind::While(condition.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_for(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::For, Some(SyntaxRole::Clause));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic()?;

        self.pop_context();

        self.push_context(ContextKind::For, Some(SyntaxRole::Body));

        let body = self.parse_statement()?;

        self.pop_context();

        let end = body.span.end;
        let kind = ExprKind::For(clause.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }


    fn parse_return(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Return, None);

        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            self.push_context(ContextKind::Return, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        };

        self.pop_context();

        let kind = ExprKind::Return(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_break(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Break, None);

        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            self.push_context(ContextKind::Break, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        };

        self.pop_context();

        let kind = ExprKind::Break(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_continue(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Continue, None);

        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            self.push_context(ContextKind::Continue, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        };

        self.pop_context();

        let kind = ExprKind::Continue(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }
}