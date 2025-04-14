#![allow(dead_code)]

use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::{ParseError, Parser, Primary};
use crate::axo_parser::item::ItemKind;

#[derive(Hash, Eq, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Hash, Eq, Clone, PartialEq)]
pub enum ExprKind {
    // Primary Expressions
    Literal(Token),
    Identifier(String),
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Unary {
        operator: Token,
        operand: Box<Expr>,
    },
    Struct {
        name: Box<Expr>,
        body: Box<Expr>
    },

    // Composite Expressions
    Bind {
        key: Box<Expr>,
        value: Box<Expr>
    },
    Labeled {
        label: Box<Expr>,
        expr: Box<Expr>
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>
    },
    Invoke {
        target: Box<Expr>,
        parameters: Vec<Expr>
    },
    Path {
        left: Box<Expr>,
        right: Box<Expr>
    },
    Member {
        object: Box<Expr>,
        member: Box<Expr>
    },

    Closure {
        parameters: Vec<Expr>,
        body: Box<Expr>
    },

    // Control Flow
    Block(Vec<Expr>),
    Match {
        target: Box<Expr>,
        body: Box<Expr>
    },
    Conditional {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Loop {
        body: Box<Expr>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Expr>
    },
    For {
        clause: Box<Expr>,
        body: Box<Expr>
    },

    // Declarations & Definitions
    Item(ItemKind),
    Assignment {
        target: Box<Expr>,
        value: Box<Expr>
    },

    // Flow Control Statements
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue(Option<Box<Expr>>),

    Error(ParseError),
}

impl Expr {
    pub fn dummy() -> Expr {
        Expr {
            kind: ExprKind::Tuple(Vec::new()),
            span: Span::zero(),
        }
    }

    pub fn empty(span: Span) -> Expr {
        Expr {
            kind: ExprKind::Tuple(Vec::new()),
            span,
        }
    }

    pub fn transform(&self) -> Expr {
        let Expr { kind, span } = self.clone();

        match kind {
            ExprKind::Binary { left, operator: Token { kind: TokenKind::Operator(op), .. }, right} => {
                match op {
                    OperatorKind::Dot => {
                        let kind = ExprKind::Member { object: left.clone(), member: right.clone() };

                        Expr { kind, span }
                    }
                    OperatorKind::Colon => {
                        let kind = ExprKind::Labeled {
                            label: left.clone(),
                            expr: right.clone()
                        };

                        Expr { kind, span }
                    }
                    OperatorKind::Equal => {
                        let kind = ExprKind::Assignment {
                            target: left.clone(),
                            value: right.clone()
                        };

                        Expr { kind, span }
                    }
                    OperatorKind::ColonEqual => {
                        let item = ItemKind::Variable {
                            target: left.clone(),
                            value: Some(right.clone()),
                            ty: None,
                            mutable: false,
                        };

                        let kind = ExprKind::Item(item);

                        Expr { kind, span }
                    }
                    OperatorKind::DoubleColon => {
                        let kind = ExprKind::Path { left: left.clone(), right: right.clone() };

                        Expr { kind, span }
                    }
                    op => {
                        if let Some(op) = op.decompound() {
                            let operator = Token { kind: TokenKind::Operator(op), span: span.clone() };

                            let operation = Expr {
                                kind: ExprKind::Binary {
                                    left: left.clone().into(),
                                    operator,
                                    right: right.into(),
                                },

                                span: span.clone(),
                            };

                            let kind = ExprKind::Assignment {
                                target: left.into(),
                                value: operation.into()
                            };

                            Expr { kind, span }
                        } else if op.is_arrow() {
                            let kind = ExprKind::Bind {
                                key: left.clone(),
                                value: right.clone(),
                            };

                            Expr { kind, span }
                        } else if op.is_left_arrow() {
                            let kind = ExprKind::Bind {
                                key: right.clone(),
                                value: left.clone()
                            };

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
    fn parse_basic(&mut self) -> Expr;
    fn parse_complex(&mut self) -> Expr;
}

impl Expression for Parser {
    fn parse_basic(&mut self) -> Expr {
        self.parse_binary(Parser::parse_primary, 0)
    }

    fn parse_complex(&mut self) -> Expr {
        self.parse_binary(Parser::parse_leaf, 0)
    }
}