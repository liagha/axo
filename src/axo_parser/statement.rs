use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{ParseError};
use crate::axo_parser::expression::{Expr, ExprKind};
use crate::axo_parser::{Parser, Primary};
use crate::axo_parser::state::{Position, Context};

pub trait ControlFlow {
    fn parse_block(&mut self) -> Result<Expr, ParseError>;
    fn parse_if_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_while_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_for_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_return_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_break_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError>;
}

impl ControlFlow for Parser {
    fn parse_block(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::Block);

        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.next().unwrap();

                    let kind = ExprKind::Block(statements);
                    let expr = Expr {
                        kind,
                        span: self.span(start, end),
                    };

                    self.exit();

                    return Ok(expr);
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.next();
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt.into());
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Punctuation(PunctuationKind::RightBrace),
            Position::After,
            Context::Block,
        );

        Err(err)
    }

    fn parse_if_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::Conditional);

        self.enter(Context::Clause);

        let condition = self.parse_expression()?;

        self.exit();

        self.enter(Context::ConditionalThen);

        let then_branch = self.parse_statement()?;

        self.exit();

        self.enter(Context::ConditionalElse);

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            let expr = self.parse_statement()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        } else {
            (None, then_branch.span.end)
        };

        self.exit();

        self.exit();

        let kind = ExprKind::Conditional(condition.into(), then_branch.into(), else_branch.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_while_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::While);

        self.enter(Context::Clause);

        let condition = self.parse_expression()?;

        self.exit();

        self.enter(Context::WhileBody);

        let body = self.parse_statement()?;

        self.exit();

        self.exit();

        let end = body.span.end;
        let kind = ExprKind::While(condition.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_for_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::For);

        self.enter(Context::Clause);

        let clause = self.parse_expression()?;

        self.exit();

        self.enter(Context::ForBody);

        let body = self.parse_statement()?;

        self.exit();

        self.exit();

        let end = body.span.end;
        let kind = ExprKind::For(clause.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_return_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::Return);

        self.enter(Context::ReturnValue);

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        self.exit();

        self.exit();

        let kind = ExprKind::Return(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_break_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::Break);

        self.enter(Context::BreakValue);

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        self.exit();

        self.exit();

        let kind = ExprKind::Break(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::Continue);

        self.enter(Context::ContinueValue);

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        self.exit();

        self.exit();

        let kind = ExprKind::Continue(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }
}