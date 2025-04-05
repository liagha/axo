use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{Error, ErrorKind};
use crate::axo_parser::expression::{Expr, ExprKind};
use crate::axo_parser::{Parser, Primary};
use crate::axo_parser::state::{Position, Context, ContextKind, SyntaxRole};

pub trait ControlFlow {
    fn parse_block(&mut self) -> Result<Expr, Error>;
    fn parse_conditional(&mut self) -> Result<Expr, Error>;
    fn parse_while(&mut self) -> Result<Expr, Error>;
    fn parse_for(&mut self) -> Result<Expr, Error>;
    fn parse_return(&mut self) -> Result<Expr, Error>;
    fn parse_break(&mut self) -> Result<Expr, Error>;
    fn parse_continue(&mut self) -> Result<Expr, Error>;
}

impl ControlFlow for Parser {
    fn parse_block(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Block, Some(SyntaxRole::Body));

        let brace = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = brace;

        let mut statements = Vec::new();

        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token { kind: TokenKind::Punctuation(PunctuationKind::RightBrace), .. } => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.next().unwrap();

                    self.pop_context();

                    let kind = ExprKind::Block(statements);
                    let expr = Expr {
                        kind,
                        span: self.span(start, end),
                    };

                    return Ok(expr);
                }
                Token { kind: TokenKind::Punctuation(PunctuationKind::Semicolon), span: Span { end, .. } } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt.into());
                }
            }
        }

        Err(Error::new(ErrorKind::UnclosedDelimiter(brace), self.span(start, err_end)))
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

        let body = self.parse_basic()?;

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