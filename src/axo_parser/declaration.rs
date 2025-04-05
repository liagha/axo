use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{Error, ErrorKind};
use crate::axo_parser::{Expr, ExprKind, Parser, Primary};
use crate::axo_parser::state::{Position, Context, ContextKind, SyntaxRole};

pub trait Declaration {
    fn parse_let(&mut self) -> Result<Expr, Error>;
    fn parse_function(&mut self) -> Result<Expr, Error>;
    fn parse_enum(&mut self) -> Result<Expr, Error>;
    fn parse_struct_definition(&mut self) -> Result<Expr, Error>;
}

impl Declaration for Parser {
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

    fn parse_function(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Function, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let function = self.parse_basic()?;

        self.pop_context();

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
            _ => {
                let body = self.parse_statement()?;

                let end = body.span.end;
                let kind = ExprKind::Function(function.into(), Vec::new(), body.into());
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
        }
    }

    fn parse_enum(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Enum, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_statement()?;

        self.pop_context();

        let Expr { kind, span: Span { end, .. } } = struct_init;

        if let ExprKind::Struct(name, fields) = kind {
            let kind = ExprKind::Enum(name, fields);
            let expr = Expr { kind, span: self.span(start, end) };

            Ok(expr)
        } else {
            Err(Error::new(ErrorKind::ExpectedSyntax(ContextKind::Enum), self.span(start, end)))
        }
    }

    fn parse_struct_definition(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Struct, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_statement()?;

        self.pop_context();

        let Expr { kind, span: Span { end, .. } } = struct_init;

        if let ExprKind::Struct(name, fields) = kind {
            let kind = ExprKind::StructDef(name, fields);
            let expr = Expr { kind, span: self.span(start, end) };

            Ok(expr)
        } else {
            Err(Error::new(ErrorKind::ExpectedSyntax(ContextKind::Struct), self.span(start, end)))
        }
    }
}
