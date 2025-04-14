use crate::axo_lexer::{Span, Token};
use crate::axo_parser::{ParseError, Expr, ExprKind, Parser, Primary};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::expression::Expression;
use crate::axo_parser::state::{ContextKind, SyntaxRole};

#[derive(Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Clone)]
pub enum ItemKind {
    Use(Box<Expr>),
    Expression(Box<Expr>),
    Implement {
        expr: Box<Expr>,
        body: Box<Expr>
    },
    Trait {
        name: Box<Expr>,
        body: Box<Expr>
    },
    Variable {
        target: Box<Expr>,
        value: Option<Box<Expr>>,
        ty: Option<Box<Expr>>,
        mutable: bool,
    },
    Field {
        name: Box<Expr>,
        value: Option<Box<Expr>>,
        ty: Option<Box<Expr>>,
    },
    Struct {
        name: Box<Expr>,
        body: Box<Expr>
    },
    Enum {
        name: Box<Expr>,
        body: Box<Expr>,
    },
    Macro {
        name: Box<Expr>,
        parameters: Vec<Expr>,
        body: Box<Expr>
    },
    Function {
        name: Box<Expr>,
        parameters: Vec<Expr>,
        body: Box<Expr>
    },
    Unit,
}

pub trait ItemParser {
    fn parse_use(&mut self) -> Expr;
    fn parse_impl(&mut self) -> Expr;
    fn parse_trait(&mut self) -> Expr;
    fn parse_function(&mut self) -> Expr;
    fn parse_macro(&mut self) -> Expr;
    fn parse_enum(&mut self) -> Expr;
    fn parse_struct(&mut self) -> Expr;
}

impl ItemParser for Parser {
    fn parse_use(&mut self) -> Expr {
        self.push_context(ContextKind::Use, None);

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = {
            self.push_context(ContextKind::Use, Some(SyntaxRole::Value));

            let expr = self.parse_complex();
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

        expr
    }
    fn parse_impl(&mut self) -> Expr {
        self.push_context(ContextKind::Implementation, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let implementation = self.parse_basic();

        self.pop_context();

        let body = self.parse_statement();

        let end = body.span.end;

        let item = ItemKind::Implement { expr: implementation.into(), body: body.into() };
        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_trait(&mut self) -> Expr {
        self.push_context(ContextKind::Trait, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let trait_ = self.parse_basic();

        self.pop_context();

        let body = self.parse_statement();

        let end = body.span.end;

        let item = ItemKind::Trait {
            name: trait_.into(),
            body: body.into()
        };

        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_function(&mut self) -> Expr {
        self.push_context(ContextKind::Function, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let function = self.parse_basic();

        self.pop_context();

        match function {
            Expr {
                kind: ExprKind::Invoke { target, parameters },
                ..
            } => {
                let body = self.parse_statement();

                let end = body.span.end;

                let item = ItemKind::Function {
                    name: target.into(),
                    parameters,
                    body: body.into()
                };

                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
            _ => {
                let body = self.parse_statement();

                let end = body.span.end;

                let item = ItemKind::Function {
                    name: function.into(),
                    parameters: Vec::new(),
                    body: body.into()
                };

                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
        }
    }

    fn parse_macro(&mut self) -> Expr {
        self.push_context(ContextKind::Macro, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let macro_ = self.parse_basic();

        self.pop_context();

        match macro_ {
            Expr {
                kind: ExprKind::Invoke { target, parameters},
                ..
            } => {
                let body = self.parse_statement();

                let end = body.span.end;

                let item = ItemKind::Macro {
                    name: target.into(),
                    parameters,
                    body: body.into()
                };

                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
            _ => {
                let body = self.parse_statement();

                let end = body.span.end;

                let item = ItemKind::Macro {
                    name: macro_.into(),
                    parameters: Vec::new(),
                    body: body.into()
                };

                let kind = ExprKind::Item(item);

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
        }
    }

    fn parse_enum(&mut self) -> Expr {
        self.push_context(ContextKind::Enum, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_statement();

        self.pop_context();

        let Expr { kind, span: Span { end, .. } } = struct_init;

        if let ExprKind::Struct { name, body } = kind {
            let item = ItemKind::Enum {
                name: name.into(),
                body: body.into()
            };

            let kind = ExprKind::Item(item);

            let expr = Expr {
                kind,
                span: self.span(start, end),
            };

            expr
        } else {
            self.error(&ParseError::new(ErrorKind::ExpectedSyntax(ContextKind::Enum), self.span(start, end)))
        }
    }

    fn parse_struct(&mut self) -> Expr {
        self.push_context(ContextKind::Struct, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_statement();

        self.pop_context();

        let Expr { kind, span: Span { end, .. } } = struct_init;

        if let ExprKind::Struct { name, body } = kind {
            let item = ItemKind::Struct {
                name: name.into(),
                body: body.into()
            };

            let kind = ExprKind::Item(item);

            let expr = Expr {
                kind,
                span: self.span(start, end),
            };

            expr
        } else {
            self.error(&ParseError::new(ErrorKind::ExpectedSyntax(ContextKind::Struct), self.span(start, end)))
        }
    }
}