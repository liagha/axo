use crate::axo_lexer::{Span, Token};
use crate::axo_parser::{Expr, ExprKind, Parser, Primary};
use crate::axo_parser::error::{Error, ErrorKind};
use crate::axo_parser::expression::Expression;
use crate::axo_parser::state::{ContextKind, SyntaxRole};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Use(Box<Expr>),
    Implement(Box<Expr>, Box<Expr>),
    Trait(Box<Expr>, Box<Expr>),
    Struct(Box<Expr>, Box<Expr>),
    Enum(Box<Expr>, Box<Expr>),
    Macro(Box<Expr>, Vec<Expr>, Box<Expr>),
    Function(Box<Expr>, Vec<Expr>, Box<Expr>),
}

pub trait Item {
    fn parse_use(&mut self) -> Result<Expr, Error>;
    fn parse_impl(&mut self) -> Result<Expr, Error>;
    fn parse_trait(&mut self) -> Result<Expr, Error>;
    fn parse_function(&mut self) -> Result<Expr, Error>;
    fn parse_macro(&mut self) -> Result<Expr, Error>;
    fn parse_enum(&mut self) -> Result<Expr, Error>;
    fn parse_struct(&mut self) -> Result<Expr, Error>;
}

impl Item for Parser {
    fn parse_use(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Use, None);

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = {
            self.push_context(ContextKind::Use, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (expr.into(), end)
        };

        self.pop_context();

        let item = ItemKind::Use(value);
        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }
    fn parse_impl(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Implementation, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let implementation = self.parse_basic()?;

        self.pop_context();

        let body = self.parse_statement()?;

        let end = body.span.end;

        let item = ItemKind::Implement(implementation.into(), body.into());
        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_trait(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Trait, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let trait_ = self.parse_basic()?;

        self.pop_context();

        let body = self.parse_statement()?;

        let end = body.span.end;

        let item = ItemKind::Trait(trait_.into(), body.into());
        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
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

                let item = ItemKind::Function(name.into(), parameters, body.into());
                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
            _ => {
                let body = self.parse_statement()?;

                let end = body.span.end;

                let item = ItemKind::Function(function.into(), Vec::new(), body.into());
                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
        }
    }

    fn parse_macro(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Macro, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let macro_ = self.parse_basic()?;

        self.pop_context();

        match macro_ {
            Expr {
                kind: ExprKind::Invoke(name, parameters),
                ..
            } => {
                let body = self.parse_statement()?;

                let end = body.span.end;

                let item = ItemKind::Macro(name.into(), parameters, body.into());
                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
            _ => {
                let body = self.parse_statement()?;

                let end = body.span.end;

                let item = ItemKind::Macro(macro_.into(), Vec::new(), body.into());
                let kind = ExprKind::Item(item);

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

        if let ExprKind::Struct(name, body) = kind {
            let item = ItemKind::Enum(name.into(), body.into());

            let kind = ExprKind::Item(item);

            let expr = Expr {
                kind,
                span: self.span(start, end),
            };

            Ok(expr)
        } else {
            Err(Error::new(ErrorKind::ExpectedSyntax(ContextKind::Enum), self.span(start, end)))
        }
    }

    fn parse_struct(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Struct, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_statement()?;

        self.pop_context();

        let Expr { kind, span: Span { end, .. } } = struct_init;

        if let ExprKind::Struct(name, body) = kind {
            let item = ItemKind::Struct(name.into(), body.into());

            let kind = ExprKind::Item(item);

            let expr = Expr {
                kind,
                span: self.span(start, end),
            };

            Ok(expr)
        } else {
            Err(Error::new(ErrorKind::ExpectedSyntax(ContextKind::Struct), self.span(start, end)))
        }
    }
}