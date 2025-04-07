#![allow(dead_code)]

use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{Error};
use crate::axo_parser::{Parser, Primary};
use crate::axo_parser::item::ItemKind;

#[derive(Hash, Eq, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
    // Primary Expressions
    Literal(Token),
    Identifier(String),
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
    Struct(Box<Expr>, Box<Expr>),

    // Composite Expressions
    Bind(Box<Expr>, Box<Expr>),
    Typed(Box<Expr>, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Invoke(Box<Expr>, Vec<Expr>),
    Path(Box<Expr>, Box<Expr>),
    Member(Box<Expr>, Box<Expr>),
    Closure(Vec<Expr>, Box<Expr>),

    // Control Flow
    Match(Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    While(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),

    // Declarations & Definitions
    Item(ItemKind),
    Assignment(Box<Expr>, Box<Expr>),
    Definition(Box<Expr>, Option<Box<Expr>>),

    // Flow Control Statements
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue(Option<Box<Expr>>),
}

impl Expr {
    pub fn transform(&self) -> Expr {
        let Expr { kind, span } = self.clone();

        match kind {
            ExprKind::Binary(left, Token { kind: TokenKind::Operator(op), .. }, right) => {
                match op {
                    OperatorKind::Dot => {
                        let kind = ExprKind::Member(left.clone(), right.clone());

                        Expr { kind, span }
                    }
                    OperatorKind::Colon => {
                        let kind = ExprKind::Typed(left.clone(), right.clone());

                        Expr { kind, span }
                    }
                    OperatorKind::Equal => {
                        let kind = ExprKind::Assignment(left.clone(), right.clone());

                        Expr { kind, span }
                    }
                    OperatorKind::ColonEqual => {
                        let kind = ExprKind::Definition(left.clone(), Some(right.clone()));

                        Expr { kind, span }
                    }
                    OperatorKind::DoubleColon => {
                        let kind = ExprKind::Path(left.clone(), right.clone());

                        Expr { kind, span }
                    }
                    op => {
                        if let Some(op) = op.decompound() {
                            let operator = Token { kind: TokenKind::Operator(op), span: span.clone() };

                            let operation = Expr {
                                kind: ExprKind::Binary(
                                    left.clone().into(),
                                    operator,
                                    right.into(),
                                ),
                                span: span.clone(),
                            };

                            let kind = ExprKind::Assignment(left.into(), operation.into());

                            Expr { kind, span }
                        } else if op.is_arrow() {
                            let kind = ExprKind::Bind(left.clone(), right.clone());

                            Expr { kind, span }
                        } else if op.is_left_arrow() {
                            let kind = ExprKind::Bind(right.clone(), left.clone());

                            Expr { kind, span }
                        } else {
                            self.clone()
                        }
                    }
                }
            }
            _ => self.clone(),
        }
    }
}

pub trait Expression {
    fn parse_basic(&mut self) -> Result<Expr, Error>;
    fn parse_complex(&mut self) -> Result<Expr, Error>;
}

impl Expression for Parser {
    fn parse_basic(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_term(Parser::parse_primary)?;

        while let Some(Token {
                           kind: TokenKind::Operator(op),
                           ..
                       }) = self.peek().cloned()
        {
            if op.is_expression() {
                let op = self.next().unwrap();

                let right = self.parse_term(Parser::parse_primary)?;

                let start = left.span.start;
                let end = right.span.end;
                let span = self.span(start, end);

                let kind = ExprKind::Binary(left.into(), op, right.into());

                left = Expr { kind, span }.transform();
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_complex(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_term(Parser::parse_leaf)?;

        while let Some(Token {
                           kind: TokenKind::Operator(op),
                           ..
                       }) = self.peek().cloned()
        {
            if op.is_expression() {
                let op = self.next().unwrap();

                let right = self.parse_term(Parser::parse_leaf)?;

                let start = left.span.start;
                let end = right.span.end;
                let span = self.span(start, end);

                let kind = ExprKind::Binary(left.into(), op, right.into());

                left = Expr { kind, span }.transform();
            } else {
                break;
            }
        }

        Ok(left)
    }
}