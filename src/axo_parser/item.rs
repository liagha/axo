use crate::axo_lexer::{PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::{ParseError, Expr, ExprKind, Parser, Primary};
use crate::axo_parser::delimiter::Delimiter;
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
        fields: Vec<Item>
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
    fn parse_field(&mut self) -> Item;
    fn parse_use(&mut self) -> Expr;
    fn parse_impl(&mut self) -> Expr;
    fn parse_trait(&mut self) -> Expr;
    fn parse_function(&mut self) -> Expr;
    fn parse_macro(&mut self) -> Expr;
    fn parse_enum(&mut self) -> Expr;
    fn parse_struct(&mut self) -> Expr;
}

impl ItemParser for Parser {
    fn parse_field(&mut self) -> Item {
        let Expr { kind, span } = self.parse_statement();

        match kind {
            ExprKind::Assignment {
                target: box Expr {
                    kind: ExprKind::Labeled { label, expr }, ..
                },
                value
            } => {
                let kind = ItemKind::Field { name: label, value: Some(value), ty: Some(expr) };

                Item {
                    kind,
                    span,
                }
            }

            ExprKind::Assignment {
                target,
                value
            } => {
                let kind = ItemKind::Field { name: target.into(), value: Some(value), ty: None };

                Item {
                    kind,
                    span,
                }
            }

            ExprKind::Labeled {
                label, expr
            } => {
                let kind = ItemKind::Field { name: label, value: None, ty: Some(expr) };
                Item {
                    kind,
                    span,
                }
            }

            _ => {
                let expr = Expr {
                    kind,
                    span: span.clone(),
                };

                Item {
                    kind: ItemKind::Field { name: expr.into(), value: None, ty: None },
                    span,
                }
            }
        }
    }

    fn parse_use(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = {
            let expr = self.parse_complex();
            let end = expr.span.end;

            (expr.into(), end)
        };

        let item = ItemKind::Use(value);
        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }
    fn parse_impl(&mut self) -> Expr {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let implementation = self.parse_basic();

        

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
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let trait_ = self.parse_basic();

        

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
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let function = self.parse_basic();

        

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
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let macro_ = self.parse_basic();

        

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
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let struct_init = self.parse_statement();

        

        let Expr { kind, span: Span { end, .. } } = struct_init;

        if let ExprKind::Constructor { name, body } = kind {
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
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let name = self.parse_atom();

        let (fields, span) = self.parse_delimited(
              TokenKind::Punctuation(PunctuationKind::LeftBrace),
              TokenKind::Punctuation(PunctuationKind::RightBrace),
              TokenKind::Punctuation(PunctuationKind::Comma),
              true,
              Parser::parse_field
        );

        let end = span.end;

        let item = ItemKind::Struct {
            name: name.into(),
            fields
        };

        let kind = ExprKind::Item(item);

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }
}