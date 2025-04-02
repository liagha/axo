use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{ParseError};
use crate::axo_parser::{Expr, ExprKind, Parser, Primary};
use crate::axo_parser::state::{Position, Context};

pub trait Composite {
    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError>;
    fn parse_invoke(&mut self, name: Expr) -> Result<Expr, ParseError>;
    fn parse_closure(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct(&mut self, struct_name: Expr) -> Result<Expr, ParseError>;
}

impl Composite for Parser {
    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.enter(Context::Index);

        self.next();

        let Expr {
            span: Span { start, .. },
            ..
        } = left;

        self.enter(Context::IndexValue);

        let index = self.parse_expression()?;

        let result = if let Some(Token {
            kind: TokenKind::Punctuation(PunctuationKind::RightBracket),
            span: Span { end, .. },
        }) = self.next()
        {
            let kind = ExprKind::Index(left.into(), index.into());
            let span = self.span(start, end);
            let expr = Expr { kind, span };

            self.exit();

            Ok(expr)
        } else {
            let err = ParseError::ExpectedTokenNotFound(
                TokenKind::Punctuation(PunctuationKind::RightBracket),
                Position::After,
                Context::ArrayElements,
            );

            Err(err)
        };

        self.exit();

        result
    }

    fn parse_invoke(&mut self, name: Expr) -> Result<Expr, ParseError> {
        self.enter(Context::Invoke);

        let Expr {
            span: Span { start, .. },
            ..
        } = name;

        self.enter(Context::InvokeParameters);

        let parameters = self.parse_tuple()?;

        let result = match parameters {
            Expr {
                kind: ExprKind::Tuple(parameters),
                span: Span { end, .. },
            } => {
                let kind = ExprKind::Invoke(name.into(), parameters);
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
            Expr {
                span: Span { end, .. },
                ..
            } => {
                let kind = ExprKind::Invoke(name.into(), vec![parameters]);
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
        };

        self.exit();

        self.exit();

        result
    }

    fn parse_closure(&mut self) -> Result<Expr, ParseError> {
        self.enter(Context::Closure);

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::ClosureParameters);

        let mut parameters = Vec::new();

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Operator(OperatorKind::Pipe),
                    span: Span { end, .. },
                } => {
                    self.next();

                    self.enter(Context::ClosureBody);

                    let body = self.parse_statement()?;

                    self.exit();

                    self.exit();

                    return Ok(Expr {
                        kind: ExprKind::Closure(parameters, body.into()),
                        span: self.span(start, end),
                    });
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    ..
                } => {
                    self.next();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    parameters.push(expr.into());
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Operator(OperatorKind::Pipe),
            Position::After,
            Context::ClosureParameters,
        );

        Err(err)
    }

    fn parse_struct(&mut self, struct_name: Expr) -> Result<Expr, ParseError> {
        self.enter(Context::Struct);

        self.next();

        let Expr {
            span: Span { start, .. },
            ..
        } = struct_name;

        self.enter(Context::StructFields);

        let mut fields = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    self.next();
                }
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.next().unwrap();

                    let kind = ExprKind::Struct(struct_name.into(), fields);
                    let expr = Expr {
                        kind,
                        span: self.span(start, end),
                    };

                    self.exit();

                    self.exit();

                    return Ok(expr);
                }
                _ => {
                    let stmt = self.parse_statement()?;

                    fields.push(stmt);
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Punctuation(PunctuationKind::RightBrace),
            Position::After,
            Context::StructFields,
        );

        Err(err)
    }
}
