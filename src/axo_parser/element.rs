use crate::axo_data::tree::{Node, Tree};
use crate::axo_lexer::OperatorKind::{Colon, Composite, Dot, Equal};
use crate::axo_parser::{ItemKind, ParseError};
use crate::axo_span::Span;
use crate::{Token, TokenKind};

#[derive(Eq, Clone)]
pub struct Element {
    pub kind: ElementKind,
    pub span: Span,
}

#[derive(Eq, Clone)]
pub enum ElementKind {
    // Primitives
    Literal(TokenKind), // Basic values (string, char, number, etc.)
    Identifier(String), // Named reference
    Error(ParseError),  // Error representation

    // Groupings
    Group(Vec<Element>),      // Comma-separated in parentheses: (a, b)
    Sequence(Vec<Element>),   // Semicolon-separated in parentheses: (a; b)
    Collection(Vec<Element>), // Comma-separated in brackets: [a, b]
    Series(Vec<Element>),     // Semicolon-separated in brackets: [a; b]
    Bundle(Vec<Element>),     // Comma-separated in braces: {a, b}
    Scope(Vec<Element>),      // Semicolon-separated in braces: {a; b}

    // Operations
    Binary {
        // Expression op Expression
        left: Box<Element>,
        operator: Token,
        right: Box<Element>,
    },
    Unary {
        // op Expression or Expression op
        operator: Token,
        operand: Box<Element>,
    },

    // Associations
    Chain {
        left: Box<Element>,
        right: Box<Element>,
    },
    Bind {
        // Connects key to value
        key: Box<Element>,
        value: Box<Element>,
    },
    Labeled {
        // Names an expression
        label: Box<Element>,
        element: Box<Element>,
    },

    // Access
    Member {
        // Access object property
        object: Box<Element>,
        member: Box<Element>,
    },
    Index {
        // Access by position
        element: Box<Element>,
        index: Box<Element>,
    },
    Invoke {
        // Function call
        target: Box<Element>,
        parameters: Vec<Element>,
    },
    Constructor {
        // Creates named structure
        name: Box<Element>,
        body: Box<Element>,
    },
    Path {
        // Namespace traversal
        tree: Tree<Box<Element>>,
    },
    Closure {
        // Anonymous function
        parameters: Vec<Element>,
        body: Box<Element>,
    },

    // Control structures
    Conditional {
        // Branching logic
        condition: Box<Element>,
        then: Box<Element>,
        alternate: Option<Box<Element>>,
    },
    Loop {
        // loop
        condition: Option<Box<Element>>,
        body: Box<Element>,
    },
    Iterate {
        // Iterative loop
        clause: Box<Element>,
        body: Box<Element>,
    },
    Match {
        // Pattern matching
        target: Box<Element>,
        body: Box<Element>,
    },

    // Declarations
    Item(ItemKind), // Module-level definition
    Assignment {
        // Value binding
        target: Box<Element>,
        value: Box<Element>,
    },

    // Control flow
    Return(Option<Box<Element>>),   // Exit function with value
    Break(Option<Box<Element>>),    // Exit loop with value
    Skip(Option<Box<Element>>), // Skip to next iteration with value
}

impl Element {
    pub fn empty(span: Span) -> Element {
        Element {
            kind: ElementKind::Group(Vec::new()),
            span,
        }
    }

    pub fn new(kind: ElementKind, span: Span) -> Element {
        match kind.clone() {
            ElementKind::Binary {
                left,
                operator:
                    Token {
                        kind: TokenKind::Operator(op),
                        ..
                    },
                right,
            } => match op.as_slice() {
                [Dot] => {
                    let kind = ElementKind::Member {
                        object: left.clone(),
                        member: right.clone(),
                    };

                    Element { kind, span }
                }
                [Colon] => {
                    let kind = ElementKind::Labeled {
                        label: left.clone(),
                        element: right.clone(),
                    };

                    Element { kind, span }
                }
                [Equal] => {
                    let kind = ElementKind::Assignment {
                        target: left.clone(),
                        value: right.clone(),
                    };

                    Element { kind, span }
                }
                [Colon, Equal] => {
                    let item = ItemKind::Variable {
                        target: left.clone(),
                        value: Some(right.clone()),
                        ty: None,
                        mutable: false,
                    };

                    let kind = ElementKind::Item(item);

                    Element { kind, span }
                }
                [Colon, Colon] => {
                    let kind = match &left.kind {
                        ElementKind::Path { tree } => {
                            let mut new_tree = tree.clone();

                            if let Some(root) = new_tree.root_mut() {
                                let mut current = root;

                                while current.has_children() {
                                    let last_idx = current.child_count() - 1;
                                    current = current.get_child_mut(last_idx).unwrap();
                                }

                                current.add_value(right.as_ref().clone().into());
                            }

                            ElementKind::Path { tree: new_tree }
                        }
                        _ => {
                            let node = Node::with_children(
                                left.as_ref().clone().into(),
                                vec![Node::new(right.as_ref().clone().into())],
                            );

                            let tree = Tree::with_root_node(node);
                            ElementKind::Path { tree }
                        }
                    };

                    Element { kind, span }
                }
                op => {
                    let op = Composite(op.into());

                    if let Some(op) = op.decompound() {
                        let operator = Token {
                            kind: TokenKind::Operator(op),
                            span: span.clone(),
                        };

                        let operation = Element {
                            kind: ElementKind::Binary {
                                left: left.clone().into(),
                                operator,
                                right: right.into(),
                            },

                            span: span.clone(),
                        };

                        let kind = ElementKind::Assignment {
                            target: left.into(),
                            value: operation.into(),
                        };

                        Element { kind, span }
                    } else if op.is_arrow() {
                        let kind = ElementKind::Bind {
                            key: left.clone(),
                            value: right.clone(),
                        };

                        Element { kind, span }
                    } else if op.is_left_arrow() {
                        let kind = ElementKind::Bind {
                            key: right.clone(),
                            value: left.clone(),
                        };

                        Element { kind, span }
                    } else {
                        Element { kind, span }
                    }
                }
            },
            _ => Element { kind, span },
        }
    }
}
