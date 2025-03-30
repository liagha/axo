use crate::lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::{Expr, ExprKind, Parser, Primary};

pub trait Composite {
    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError>;
    fn parse_call(&mut self, name: Expr) -> Result<Expr, ParseError>;
    fn parse_closure(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct(&mut self, struct_name: Expr) -> Result<Expr, ParseError>;
}

impl Composite for Parser {
    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();

        let Expr {
            span: Span { start, .. },
            ..
        } = left;

        let index = self.parse_expression()?;

        if let Some(Token {
            kind: TokenKind::Punctuation(PunctuationKind::RightBracket),
            span: Span { end, .. },
        }) = self.advance()
        {
            let kind = ExprKind::Index(left.into(), index.into());
            let span = Span { start, end };
            let expr = Expr { kind, span };

            Ok(expr)
        } else {
            let err = ParseError::ExpectedTokenNotFound(
                TokenKind::Punctuation(PunctuationKind::RightBracket),
                SyntaxPosition::After,
                SyntaxType::ArrayElements,
            );

            Err(err)
        }
    }

    fn parse_call(&mut self, name: Expr) -> Result<Expr, ParseError> {
        let Expr {
            span: Span { start, .. },
            ..
        } = name;

        let parameters = self.parse_tuple()?;

        if let Expr {
            kind: ExprKind::Tuple(parameters),
            span: Span { end, .. },
        } = parameters
        {
            let kind = ExprKind::Invoke(name.into(), parameters);
            let expr = Expr {
                kind,
                span: Span { start, end },
            };

            Ok(expr)
        } else {
            let err = ParseError::ExpectedTokenNotFound(
                TokenKind::Punctuation(PunctuationKind::RightParen),
                SyntaxPosition::After,
                SyntaxType::FunctionParameters,
            );

            Err(err)
        }
    }

    fn parse_closure(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.advance().unwrap();

        let mut parameters = Vec::new();

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Operator(OperatorKind::Pipe),
                    span: Span { end, .. },
                } => {
                    self.advance();

                    let body = self.parse_primary()?;

                    return Ok(Expr {
                        kind: ExprKind::Closure(parameters, body.into()),
                        span: Span { start, end },
                    });
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    ..
                } => {
                    self.advance();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    parameters.push(expr.into());
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Operator(OperatorKind::Pipe),
            SyntaxPosition::After,
            SyntaxType::ClosureParameters,
        );

        Err(err)
    }

    fn parse_struct(&mut self, struct_name: Expr) -> Result<Expr, ParseError> {
        self.advance();

        let Expr {
            span: Span { start, .. },
            ..
        } = struct_name;

        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    self.advance();
                }
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.advance().unwrap();

                    let kind = ExprKind::Struct(struct_name.into(), statements);
                    let expr = Expr {
                        kind,
                        span: Span { start, end },
                    };

                    return Ok(expr);
                }
                _ => {
                    let stmt = self.parse_statement()?;

                    statements.push(stmt);
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Punctuation(PunctuationKind::RightBrace),
            SyntaxPosition::After,
            SyntaxType::StructFields,
        );

        Err(err)
    }
}
