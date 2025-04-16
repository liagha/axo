#![allow(dead_code)]

use crate::{
    axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind},
    axo_parser::{item::ItemKind, ParseError, Parser, Primary},
    axo_data::tree::Tree,
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
    Literal(Token),
    Identifier(String),

    // Composite
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
    Constructor {
        name: Box<Expr>,
        body: Box<Expr>
    },

    // Operations
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Unary {
        operator: Token,
        operand: Box<Expr>,
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
                        // Create a tree for the path
                        // If left is already a Path with a tree, extend it by adding right as a child
                        // Otherwise, create a new tree with left as the root and right as a child
                        let kind = match &left.kind {
                            ExprKind::Path { tree } => {
                                // Clone the existing tree and add the right expression as a new node
                                let mut new_tree = tree.clone();

                                // Find the rightmost leaf node to add the new path segment
                                if let Some(root) = new_tree.root_mut() {
                                    // Simple implementation: add as a child to the first node with no children
                                    // This could be enhanced to be more sophisticated based on actual requirements
                                    let mut current = root;
                                    while current.has_children() {
                                        // Navigate to the last child
                                        let last_idx = current.child_count() - 1;
                                        current = current.get_child_mut(last_idx).unwrap();
                                    }

                                    // Add the right expression as a new node
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