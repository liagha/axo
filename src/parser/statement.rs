use crate::lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::expression::{Expr, ExprKind};
use crate::parser::{Parser, Primary};

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
            SyntaxPosition::After,
            SyntaxType::Block,
        );

        Err(err)
    }

    fn parse_if_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_expression()?;

        let then_branch = self.parse_statement()?;

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            let expr = self.parse_statement()?;
            let end = expr.span.end;

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

    fn parse_while_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_expression()?;

        let body = self.parse_statement()?;

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

        let clause = self.parse_expression()?;

        let body = self.parse_statement()?;

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

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedTokenNotFound(
                TokenKind::Punctuation(PunctuationKind::Semicolon),
                SyntaxPosition::After,
                SyntaxType::ReturnValue,
            );

            return Err(err);
        }

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

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedTokenNotFound(
                TokenKind::Punctuation(PunctuationKind::Semicolon),
                SyntaxPosition::After,
                SyntaxType::ReturnValue,
            );

            return Err(err);
        }

        let kind = ExprKind::Break(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        if let Some(Token {
                        kind: TokenKind::Punctuation(PunctuationKind::Semicolon),
                        span: Span { end, .. },
                    }) = self.next()
        {
            let kind = ExprKind::Continue;
            let expr = Expr {
                kind,
                span: self.span(start, end),
            };

            Ok(expr)
        } else {
            let err = ParseError::ExpectedTokenNotFound(
                TokenKind::Punctuation(PunctuationKind::Semicolon),
                SyntaxPosition::After,
                SyntaxType::Continue,
            );

            Err(err)
        }
    }
}