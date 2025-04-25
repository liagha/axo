#![allow(dead_code)]

use {
    crate::{
        axo_lexer::{OperatorKind, PunctuationKind, Token, TokenKind},
        axo_parser::{item::ItemKind, ParseError, Parser, Primary},
        axo_data::tree::Tree,
        axo_span::Span,
        axo_data::tree::Node
    },
};

#[derive(Eq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Eq, Clone)]
pub enum ExprKind {
    // Primitives
    Literal(TokenKind),        // Basic values (string, char, number, etc.)
    Identifier(String),        // Named reference
    Error(ParseError),         // Error representation

    // Groupings
    Group(Vec<Expr>),          // Comma-separated in parentheses: (a, b)
    Sequence(Vec<Expr>),       // Semicolon-separated in parentheses: (a; b)
    Collection(Vec<Expr>),     // Comma-separated in brackets: [a, b]
    Series(Vec<Expr>),         // Semicolon-separated in brackets: [a; b]
    Bundle(Vec<Expr>),         // Comma-separated in braces: {a, b}
    Block(Vec<Expr>),          // Semicolon-separated in braces: {a; b}

    // Operations
    Binary {                   // Expression op Expression
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Unary {                    // op Expression or Expression op
        operator: Token,
        operand: Box<Expr>,
    },

    // Associations
    Bind {                     // Connects key to value
        key: Box<Expr>,
        value: Box<Expr>
    },
    Labeled {                  // Names an expression
        label: Box<Expr>,
        expr: Box<Expr>
    },
    Constructor {              // Creates named structure
        name: Box<Expr>,
        body: Box<Expr>
    },

    // Access
    Member {                   // Access object property
        object: Box<Expr>,
        member: Box<Expr>
    },
    Index {                    // Access by position
        expr: Box<Expr>,
        index: Box<Expr>
    },
    Path {                     // Namespace traversal
        tree: Tree<Box<Expr>>,
    },

    // Functions
    Invoke {                   // Function call
        target: Box<Expr>,
        parameters: Vec<Expr>
    },
    Closure {                  // Anonymous function
        parameters: Vec<Expr>,
        body: Box<Expr>
    },

    // Control structures
    Conditional {              // Branching logic
        condition: Box<Expr>,
        then: Box<Expr>,
        alternate: Option<Box<Expr>>,
    },
    Loop {                     // loop
        condition: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    Iterate {                      // Iterative loop
        clause: Box<Expr>,
        body: Box<Expr>
    },
    Match {                    // Pattern matching
        target: Box<Expr>,
        body: Box<Expr>
    },

    // Declarations
    Item(ItemKind),            // Module-level definition
    Assignment {               // Value binding
        target: Box<Expr>,
        value: Box<Expr>
    },

    // Control flow
    Return(Option<Box<Expr>>), // Exit function with value
    Break(Option<Box<Expr>>),  // Exit loop with value
    Continue(Option<Box<Expr>>), // Skip to next iteration with value
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