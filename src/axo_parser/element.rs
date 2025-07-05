use {
    super::{
        SymbolKind, ParseError,
    },

    crate::{
        axo_data::tree::{
            Node, Tree
        },

        axo_scanner::{
            Token, TokenKind,
            OperatorKind,
        },

        axo_cursor::Span,
    }
};

/// Represents a single element in the Abstract Syntax Tree (AST).
///
/// An `Element` is the fundamental building block of the AST, containing
/// both the semantic information about what the element represents (`kind`)
/// and its location in the source code (`span`).
#[derive(Eq, Clone)]
pub struct Element {
    /// The semantic type and data of this element
    pub kind: ElementKind,
    /// The source code location where this element was parsed from
    pub span: Span,
}

/// Defines the various types of elements that can exist in the AST.
///
/// This enum encompasses all possible syntactic constructs in the language,
/// from primitive literals to complex control flow structures.
#[derive(Eq, Clone)]
pub enum ElementKind {
    /// A literal value such as a string, number, or character
    ///
    /// # Examples
    /// - `"hello"` (string literal)
    /// - `42` (integer literal)
    /// - `'a'` (character literal)
    Literal(TokenKind),

    /// A named identifier or variable reference
    ///
    /// # Examples
    /// - `variable_name`
    /// - `function_name`
    /// - `MY_CONSTANT`
    Identifier(String),

    /// A procedural macro or compile-time code generation construct
    ///
    /// Contains the element that defines the procedural behavior.
    Procedural(Box<Element>),

    /// Comma-separated elements in parentheses: `(a, b, c)`
    ///
    /// Used for function parameters, tuple construction, and grouping expressions.
    Group(Vec<Element>),

    /// Semicolon-separated elements in parentheses: `(a; b; c)`
    ///
    /// Used for sequential execution or statement blocks.
    Sequence(Vec<Element>),

    /// Comma-separated elements in square brackets: `[a, b, c]`
    ///
    /// Typically used for array literals or list construction.
    Collection(Vec<Element>),

    /// Semicolon-separated elements in square brackets: `[a; b; c]`
    ///
    /// Used for array initialization with repeated values or sequential operations.
    Series(Vec<Element>),

    /// Comma-separated elements in curly braces: `{a, b, c}`
    ///
    /// Used for set literals, object construction, or unordered collections.
    Bundle(Vec<Element>),

    /// Semicolon-separated elements in curly braces: `{a; b; c}`
    ///
    /// Used for code blocks, scoped execution, or structured control flow.
    Scope(Vec<Element>),

    /// Binary operation between two expressions: `left op right`
    ///
    /// # Examples
    /// - `a + b` (addition)
    /// - `x == y` (equality comparison)
    /// - `p && q` (logical AND)
    Binary {
        /// The left-hand side operand
        left: Box<Element>,
        /// The operator token (contains the operator type and source location)
        operator: Token,
        /// The right-hand side operand
        right: Box<Element>,
    },

    /// Unary operation on a single expression: `op operand` or `operand op`
    ///
    /// # Examples
    /// - `-x` (negation)
    /// - `!flag` (logical NOT)
    /// - `++counter` (pre-increment)
    /// - `value++` (post-increment)
    Unary {
        /// The operator token
        operator: Token,
        /// The operand being operated on
        operand: Box<Element>,
    },

    /// Associates a label with an element for naming or documentation
    ///
    /// # Examples
    /// - `label: expression`
    /// - `name: value` (in struct literals)
    Labeled {
        /// The label or name
        label: Box<Element>,
        /// The element being labeled
        element: Box<Element>,
    },

    /// Member access operation: `object.member`
    ///
    /// Used to access fields, methods, or properties of an object.
    Member {
        /// The object being accessed
        object: Box<Element>,
        /// The member name or expression
        member: Box<Element>,
    },

    /// Index access operation: `element[index]`
    ///
    /// Used to access elements by position in arrays, maps, or other collections.
    Index {
        /// The collection or object being indexed
        element: Box<Element>,
        /// The index expression
        index: Box<Element>,
    },

    /// Function or method invocation: `target(parameters...)`
    ///
    /// Represents calling a function with zero or more arguments.
    Invoke {
        /// The function or method being called
        target: Box<Element>,
        /// The list of arguments passed to the function
        parameters: Box<Element>,
    },

    /// Constructor call for creating instances: `Type { fields... }`
    ///
    /// Used for struct initialization, object construction, or similar patterns.
    Constructor {
        /// The type or constructor name
        name: Box<Element>,
        /// The constructor body (typically field initializations)
        body: Box<Element>,
    },

    /// Namespace or module path: `module::submodule::item`
    ///
    /// Represents hierarchical access to items across module boundaries.
    Path {
        /// Tree structure representing the path hierarchy
        tree: Tree<Box<Element>>,
    },

    /// Conditional branching: `if condition then branch else alternate`
    ///
    /// Represents if-then-else logic with optional else clause.
    Conditional {
        /// The boolean condition to evaluate
        condition: Box<Element>,
        /// The code to execute if condition is true
        then: Box<Element>,
        /// Optional code to execute if condition is false
        alternate: Option<Box<Element>>,
    },

    /// Basic loop construct
    ///
    /// Can represent both infinite loops and conditional loops depending
    /// on whether a condition is provided.
    Cycle {
        /// Optional condition for while-style loops (None for infinite loops)
        condition: Option<Box<Element>>,
        /// The loop body to execute repeatedly
        body: Box<Element>,
    },

    /// Iterator-based loop: `for item in collection`
    ///
    /// Used for iterating over collections, ranges, or other iterable objects.
    Iterate {
        /// The iteration clause (typically variable binding and collection)
        clause: Box<Element>,
        /// The loop body executed for each iteration
        body: Box<Element>,
    },

    /// Pattern matching construct: `match target { patterns... }`
    ///
    /// Allows branching based on pattern matching against a target value.
    Match {
        /// The expression being matched against
        target: Box<Element>,
        /// The match arms containing patterns and associated code
        body: Box<Element>,
    },

    /// Module-level symbol definition (function, type, constant, etc.)
    ///
    /// Represents top-level declarations that define reusable components.
    Symbolization(SymbolKind),

    /// Variable assignment: `target = value`
    ///
    /// Assigns a value to a variable, field, or other assignable location.
    Assignment {
        /// The target being assigned to (variable, field, etc.)
        target: Box<Element>,
        /// The value being assigned
        value: Box<Element>,
    },

    /// Early return from a function: `return [value]`
    ///
    /// Exits the current function, optionally returning a value.
    Return(Option<Box<Element>>),

    /// Break out of a loop: `break [value]`
    ///
    /// Exits the nearest enclosing loop, optionally with a break value.
    Break(Option<Box<Element>>),

    /// Skip to next iteration: `continue [value]` or `skip [value]`
    ///
    /// Jumps to the next iteration of the nearest enclosing loop.
    Skip(Option<Box<Element>>),
}

impl Element {
    /// Creates an empty element (empty group) with the given span.
    ///
    /// This is useful for representing missing or optional elements
    /// in the AST where a placeholder is needed.
    ///
    /// # Arguments
    /// * `span` - The source location for this empty element
    ///
    /// # Returns
    /// An `Element` containing an empty group
    ///
    /// # Examples
    /// ```rust
    /// let empty = Element::empty(span);
    /// // Represents: ()
    /// ```
    pub fn empty(span: Span) -> Element {
        Element {
            kind: ElementKind::Group(Vec::new()),
            span,
        }
    }

    /// Creates a new element with the given kind and span.
    ///
    /// This constructor applies semantic transformations based on the element kind,
    /// automatically converting certain binary operations into more specific
    /// element types (e.g., dot notation becomes member access).
    ///
    /// # Arguments
    /// * `kind` - The semantic type of the element
    /// * `span` - The source location of the element
    ///
    /// # Returns
    /// A new `Element`, potentially transformed based on the input kind
    ///
    /// # Transformations
    /// The following binary operators receive special treatment:
    /// - `.` (dot) → `Member` access
    /// - `:` (colon) → `Labeled` element  
    /// - `=` (equal) → `Assignment`
    /// - `:=` (colon-equal) → Variable declaration (`Item`)
    /// - `::` (double-colon) → Path extension
    /// - Compound assignment ops (e.g., `+=`) → Assignment with binary operation
    /// - Arrow operators → `Bind` operations
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
                // Member access: object.member
                [OperatorKind::Dot] => {
                    let kind = ElementKind::Member {
                        object: left.clone(),
                        member: right.clone(),
                    };

                    Element { kind, span }
                }
                // Labeled element: label: element
                [OperatorKind::Colon] => {
                    let kind = ElementKind::Labeled {
                        label: left.clone(),
                        element: right.clone(),
                    };

                    Element { kind, span }
                }
                // Assignment: target = value
                [OperatorKind::Equal] => {
                    let kind = ElementKind::Assignment {
                        target: left.clone(),
                        value: right.clone(),
                    };

                    Element { kind, span }
                }
                // Variable declaration: target := value
                [OperatorKind::Colon, OperatorKind::Equal] => {
                    let symbol = SymbolKind::Variable {
                        target: left.clone(),
                        value: Some(right.clone()),
                        ty: None,
                        mutable: false,
                    };

                    let kind = ElementKind::Symbolization(symbol);

                    Element { kind, span }
                }
                // Path extension: namespace::item
                [OperatorKind::Colon, OperatorKind::Colon] => {
                    let kind = match &left.kind {
                        ElementKind::Path { tree } => {
                            // Extend existing path
                            let mut new_tree = tree.clone();

                            if let Some(root) = new_tree.root_mut() {
                                let mut current = root;

                                // Navigate to the deepest node
                                while current.has_children() {
                                    let last_idx = current.child_count() - 1;
                                    current = current.get_child_mut(last_idx).unwrap();
                                }

                                // Add the new path segment
                                current.add_value(right.as_ref().clone().into());
                            }

                            ElementKind::Path { tree: new_tree }
                        }
                        _ => {
                            // Create new path from two elements
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
                // Handle compound operators and arrows
                op => {
                    let op = OperatorKind::Composite(op.into());

                    if let Some(op) = op.decompound() {
                        // Compound assignment: target op= value becomes target = target op value
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
                    } else {
                        // No transformation needed
                        Element { kind, span }
                    }
                }
            },
            // No transformation needed for non-binary elements
            _ => Element { kind, span },
        }
    }
}