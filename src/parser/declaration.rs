use crate::lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::{Expr, ExprKind, Parser, Primary};

pub trait Declaration {
    fn parse_let(&mut self) -> Result<Expr, ParseError>;
    fn parse_function(&mut self) -> Result<Expr, ParseError>;
    fn parse_enum(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError>;
}

impl Declaration for Parser {
    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let identifier = self.parse_expression()?;

        if let Some(token) = self.peek() {
            if token.kind == TokenKind::Operator(OperatorKind::Equal) {
                self.next();

                let value = self.parse_statement()?;
                let span = self.span(start, value.span.end);
                let kind = ExprKind::Definition(identifier.into(), Some(value.into()));

                let expr = Expr { kind, span };

                Ok(expr)
            } else {
                let span = identifier.span.clone();
                let kind = ExprKind::Definition(identifier.into(), None);
                let expr = Expr { kind, span };

                Ok(expr)
            }
        } else {
            Err(ParseError::UnexpectedEndOfFile)
        }
    }

    fn parse_function(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let function = self.parse_primary()?;

        match function {
            Expr {
                kind: ExprKind::Invoke(name, parameters),
                ..
            } => {
                let body = self.parse_statement()?;

                let end = body.span.end;
                let kind = ExprKind::Function(name.into(), parameters, body.into());
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
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
                    span: self.span(start, end),
                };

                Ok(expr)
            }
            expr => {
                let err = ParseError::UnexpectedExpression(
                    expr,
                    SyntaxPosition::As,
                    SyntaxType::FunctionDeclaration,
                );

                Err(err)
            }
        }
    }

    fn parse_enum(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_primary()?;

        if let Expr { kind: ExprKind::Struct(name, fields), span: Span { end, .. } } = struct_init {
            let kind = ExprKind::Enum(name, fields);
            let expr = Expr { kind, span: self.span(start, end) };

            Ok(expr)
        } else {
            let err = ParseError::MissingSyntaxElement(
                SyntaxType::Struct,
            );

            Err(err)
        }
    }
    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_primary()?;

        if let Expr { kind: ExprKind::Struct(name, fields), span: Span { end, .. } } = struct_init {
            let kind = ExprKind::StructDef(name, fields);
            let expr = Expr { kind, span: self.span(start, end) };

            Ok(expr)
        } else {
            let err = ParseError::MissingSyntaxElement(
                SyntaxType::Struct,
            );

            Err(err)
        }
    }
}
