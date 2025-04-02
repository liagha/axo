use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{ParseError};
use crate::axo_parser::{Expr, ExprKind, Parser, Primary};
use crate::axo_parser::state::{Position, Context};

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

        self.enter(Context::Definition);

        self.enter(Context::DefinitionTarget);

        let expr = self.parse_expression()?;

        let Expr { kind, span: Span { end, .. } } = expr.clone();

        let span = self.span(start, end);

        match kind {
            ExprKind::Assignment(target, value) => {
                Ok(Expr { kind: ExprKind::Definition(target, Some(value)), span })
            }
            _ => {
                Ok(Expr { kind: ExprKind::Definition(expr.into(), None), span })
            }
        }
    }

    fn parse_function(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::FunctionDeclaration);

        let function = self.parse_primary()?;

        match function {
            Expr {
                kind: ExprKind::Invoke(name, parameters),
                ..
            } => {
                self.enter(Context::FunctionBody);

                let body = self.parse_statement()?;

                let end = body.span.end;
                let kind = ExprKind::Function(name.into(), parameters, body.into());
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                self.exit();

                self.exit();

                Ok(expr)
            }
            Expr {
                kind: ExprKind::Identifier(_),
                ..
            } => {
                self.enter(Context::FunctionBody);

                let body = self.parse_statement()?;

                let end = body.span.end;
                let kind = ExprKind::Function(function.into(), Vec::new(), body.into());
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                self.exit();

                self.exit();

                Ok(expr)
            }
            expr => {
                let err = ParseError::UnexpectedExpression(
                    expr,
                    Position::As,
                    Context::FunctionDeclaration,
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

        self.enter(Context::EnumDeclaration);

        let struct_init = self.parse_primary()?;

        if let Expr { kind: ExprKind::Struct(name, fields), span: Span { end, .. } } = struct_init {
            let kind = ExprKind::Enum(name, fields);
            let expr = Expr { kind, span: self.span(start, end) };

            self.exit();

            Ok(expr)
        } else {
            let err = ParseError::MissingSyntaxElement(
                Context::StructDeclaration,
            );

            Err(err)
        }
    }
    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        self.enter(Context::StructDeclaration);

        let struct_init = self.parse_statement()?;

        if let Expr { kind: ExprKind::Struct(name, fields), span: Span { end, .. } } = struct_init {
            let kind = ExprKind::StructDef(name, fields);
            let expr = Expr { kind, span: self.span(start, end) };

            self.exit();

            Ok(expr)
        } else {
            let err = ParseError::MissingSyntaxElement(
                Context::StructDeclaration,
            );

            Err(err)
        }
    }
}
