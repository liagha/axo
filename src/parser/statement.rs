#![allow(dead_code)]

use crate::lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::expression::{Expr, ExprKind};
use crate::parser::{expression::Expression, Parser};

pub trait Statement {
    fn parse_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_let_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_block(&mut self) -> Result<Expr, ParseError>;
    fn parse_function_declaration(&mut self) -> Result<Expr, ParseError>;
    fn parse_if_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_while_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_for_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_return_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_break_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError>;
    fn parse_enum_definition(&mut self) -> Result<Expr, ParseError>;
}

impl Statement for Parser {
    fn parse_statement(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Keyword(kw) => {
                    let expr = match kw {
                        KeywordKind::If => self.parse_if_statement(),
                        KeywordKind::While => self.parse_while_statement(),
                        KeywordKind::For => self.parse_for_statement(),
                        KeywordKind::Fn => self.parse_function_declaration(),
                        KeywordKind::Return => self.parse_return_statement(),
                        KeywordKind::Break => self.parse_break_statement(),
                        KeywordKind::Continue => self.parse_continue_statement(),
                        KeywordKind::Let => self.parse_let_statement(),
                        KeywordKind::Struct => self.parse_struct_definition(),
                        KeywordKind::Enum => self.parse_enum_definition(),
                        KeywordKind::Impl => unimplemented!(),
                        KeywordKind::Trait => unimplemented!(),
                        KeywordKind::Match => unimplemented!(),
                        KeywordKind::Else => unimplemented!(),
                    }?;

                    Ok(expr)
                },
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => self.parse_block(),
                _ => {
                    let left = self.parse_expression()?;

                    if let Some(token) = self.advance() {
                        let expr = if token.kind == TokenKind::Operator(OperatorKind::Equal) {
                            let right = self.parse_statement()?;
                            let start = left.span.start;
                            let end = right.span.end;
                            let kind = ExprKind::Assignment(left.into(), right.into());
                            let expr = Expr {
                                kind,
                                span: Span { start, end },
                            };

                            expr
                        } else if OperatorKind::is_compound_token(&token.kind) {
                            let right = self.parse_statement()?;

                            let start = left.span.start;
                            let end = right.span.end;
                            let span = Span { start, end };
                            let operation_kind = ExprKind::Binary(
                                left.clone().into(),
                                OperatorKind::decompound_token(&token),
                                right.into(),
                            );
                            let operation = Expr {
                                kind: operation_kind,
                                span,
                            };

                            let kind = ExprKind::Assignment(left.into(), operation.into());
                            let expr = Expr { kind, span };

                            expr
                        } else {
                            left
                        };

                        Ok(expr)
                    } else {
                        Err(ParseError::UnexpectedEOF)
                    }
                }
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    fn parse_let_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let identifier = self.parse_expression()?;

        if let Some(token) = self.peek() {
            if token.kind == TokenKind::Operator(OperatorKind::Equal) {
                self.advance();

                let value = self.parse_statement()?;
                let span = Span {
                    start,
                    end: value.span.end,
                };
                let kind = ExprKind::Definition(identifier.into(), Some(value.into()));

                let expr = Expr { kind, span };

                Ok(expr)
            } else {
                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
                    let err = ParseError::ExpectedToken(
                        TokenKind::Punctuation(PunctuationKind::Semicolon),
                        SyntaxPosition::After,
                        SyntaxType::VariableDeclaration,
                    );

                    return Err(err);
                }

                let span = identifier.span;
                let kind = ExprKind::Definition(identifier.into(), None);
                let expr = Expr { kind, span };

                Ok(expr)
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    fn parse_block(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.advance().unwrap();

                    let kind = ExprKind::Block(statements);
                    let expr = Expr {
                        kind,
                        span: Span { start, end },
                    };

                    return Ok(expr);
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.advance();
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt.into());
                }
            }
        }

        let err = ParseError::ExpectedToken(
            TokenKind::Punctuation(PunctuationKind::RightBrace),
            SyntaxPosition::After,
            SyntaxType::Block,
        );

        Err(err)
    }

    fn parse_function_declaration(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();
        let function = self.parse_primary()?;

        match function {
            Expr {
                kind: ExprKind::Call(name, parameters),
                ..
            } => {
                let body = self.parse_statement()?;

                let end = body.span.end;
                let kind = ExprKind::Function(name.into(), parameters, body.into());
                let expr = Expr {
                    kind,
                    span: Span { start, end },
                };

                Ok(expr)
            }
            Expr {
                kind: ExprKind::Identifier(_),
                ..
            } => {
                let body = self.parse_statement()?;

                let end = body.span.end;
                let kind = ExprKind::Function(function.into(), Vec::new(), body.into());
                let expr = Expr {
                    kind,
                    span: Span { start, end },
                };

                Ok(expr)
            }
            expr => {
                let err = ParseError::UnexpectedExpr(
                    expr,
                    SyntaxPosition::As,
                    SyntaxType::FunctionDeclaration,
                );

                Err(err)
            }
        }
    }

    fn parse_if_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let condition = self.parse_expression()?;

        let then_branch = self.parse_statement()?;

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            let expr = self.parse_statement()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        } else {
            (None, then_branch.span.end)
        };

        let kind = ExprKind::If(condition.into(), then_branch.into(), else_branch.into());
        let expr = Expr {
            kind,
            span: Span { start, end },
        };

        Ok(expr)
    }

    fn parse_while_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let condition = self.parse_expression()?;

        let body = self.parse_statement()?;

        let end = body.span.end;
        let kind = ExprKind::While(condition.into(), body.into());
        let expr = Expr {
            kind,
            span: Span { start, end },
        };

        Ok(expr)
    }

    fn parse_for_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let clause = self.parse_expression()?;

        let body = self.parse_statement()?;

        let end = body.span.end;
        let kind = ExprKind::For(clause.into(), body.into());
        let expr = Expr {
            kind,
            span: Span { start, end },
        };

        Ok(expr)
    }

    fn parse_return_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, end },
            ..
        } = self.advance().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedToken(
                TokenKind::Punctuation(PunctuationKind::Semicolon),
                SyntaxPosition::After,
                SyntaxType::ReturnValue,
            );

            return Err(err);
        }

        let kind = ExprKind::Return(value);
        let expr = Expr {
            kind,
            span: Span { start, end },
        };

        Ok(expr)
    }

    fn parse_break_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, end },
            ..
        } = self.advance().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            let expr = self.parse_expression()?;
            let end = expr.span.end;

            (Some(expr.into()), end)
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedToken(
                TokenKind::Punctuation(PunctuationKind::Semicolon),
                SyntaxPosition::After,
                SyntaxType::ReturnValue,
            );

            return Err(err);
        }

        let kind = ExprKind::Break(value);
        let expr = Expr {
            kind,
            span: Span { start, end },
        };

        Ok(expr)
    }

    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        if let Some(Token {
            kind: TokenKind::Punctuation(PunctuationKind::Semicolon),
            span: Span { end, .. },
        }) = self.advance()
        {
            let kind = ExprKind::Continue;
            let expr = Expr {
                kind,
                span: Span { start, end },
            };

            Ok(expr)
        } else {
            let err = ParseError::ExpectedToken(
                TokenKind::Punctuation(PunctuationKind::Semicolon),
                SyntaxPosition::After,
                SyntaxType::Continue,
            );

            Err(err)
        }
    }

    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let struct_init = self.parse_primary()?;

        if let Expr { kind: ExprKind::StructInit(name, fields), span: Span { end, .. } } = struct_init {
            let kind = ExprKind::StructDef(name, fields);
            let expr = Expr { kind, span: Span { start, end } };

            Ok(expr)
        } else {
            let err = ParseError::ExpectedSyntax(
                SyntaxType::Struct,
            );

            Err(err)
        }
    }
    fn parse_enum_definition(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let name = self.parse_primary()?;

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            let err = ParseError::ExpectedToken(
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                SyntaxPosition::After,
                SyntaxType::EnumName,
            );

            return Err(err);
        }

        let mut variants = Vec::new();

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::RightBrace),
                    span: Span { end, .. },
                } => {
                    self.advance();

                    let kind = ExprKind::Enum(name.into(), variants);
                    let expr = Expr {
                        kind,
                        span: Span { start, end },
                    };

                    return Ok(expr);
                }
                Token {
                    kind: TokenKind::Operator(OperatorKind::Comma),
                    ..
                } => {
                    self.advance();

                    continue;
                }
                _ => {
                    let variant = self.parse_expression()?;

                    variants.push(variant);
                }
            }
        }

        let err = ParseError::ExpectedToken(
            TokenKind::Punctuation(PunctuationKind::RightBrace),
            SyntaxPosition::After,
            SyntaxType::EnumVariants,
        );

        Err(err)
    }
}
