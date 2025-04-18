#![allow(dead_code)]

use crate::{
    axo_lexer::{OperatorKind, PunctuationKind, Token, TokenKind},
    axo_parser::{item::ItemKind, ParseError, Parser, Primary},
    axo_data::tree::Tree,
    axo_span::Span,
};
use crate::axo_data::tree::Node;

#[derive(Hash, Eq, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Hash, Eq, Clone, PartialEq)]
pub enum ExprKind {
    // Primary Expressions
    Literal(Token),        // Strings, Characters, Floats and Integers
    Identifier(String),    // Identifiers for functions, structs, etc

    // Composite
    Group(Vec<Expr>),      // (exprs, ...)
    Collection(Vec<Expr>), // [exprs, ...]
    Bundle(Vec<Expr>),     // { exprs, ... }
    Constructor {          // $Name { fields, ... }
        name: Box<Expr>,
        body: Box<Expr>
    },

    // Operations
    Binary {               // First_Expr | Operator | Second_Expr
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Unary {                // Prefix or Suffix, Before or After an Expr
        operator: Token,
        operand: Box<Expr>,
    },

    // Access Expressions
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
        tree: Tree<Box<Expr>>,
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
    pub fn empty(span: Span) -> Expr {
        Expr {
            kind: ExprKind::Group(Vec::new()),
            span,
        }
    }

    pub fn transform(&self) -> Expr {
        use OperatorKind::*;

        let Expr { kind, span } = self.clone();

        match kind {
            ExprKind::Binary { left, operator: Token { kind: TokenKind::Operator(op), .. }, right} => {
                match op.as_slice() {
                    [Dot] => {
                        let kind = ExprKind::Member { object: left.clone(), member: right.clone() };

                        Expr { kind, span }
                    }
                    [Colon] => {
                        let kind = ExprKind::Labeled {
                            label: left.clone(),
                            expr: right.clone()
                        };

                        Expr { kind, span }
                    }
                    [Equal] => {
                        let kind = ExprKind::Assignment {
                            target: left.clone(),
                            value: right.clone()
                        };

                        Expr { kind, span }
                    }
                    [Colon, Equal] => {
                        let item = ItemKind::Variable {
                            target: left.clone(),
                            value: Some(right.clone()),
                            ty: None,
                            mutable: false,
                        };

                        let kind = ExprKind::Item(item);

                        Expr { kind, span }
                    }
                    [Colon, Colon] => {
                        let kind = match &left.kind {
                            ExprKind::Path { tree } => {
                                let mut new_tree = tree.clone();

                                if let Some(root) = new_tree.root_mut() {
                                    let mut current = root;

                                    while current.has_children() {
                                        let last_idx = current.child_count() - 1;
                                        current = current.get_child_mut(last_idx).unwrap();
                                    }

                                    current.add_value(right.as_ref().clone().into());
                                }

                                ExprKind::Path { tree: new_tree }
                            },
                            _ => {
                                let node = Node::with_children(
                                    left.as_ref().clone().into(),
                                    vec![Node::new(right.as_ref().clone().into())]
                                );

                                let tree = Tree::with_root_node(node);
                                ExprKind::Path { tree }
                            }
                        };

                        Expr { kind, span }
                    }
                    op => {
                        let op = Composite(op.into());

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