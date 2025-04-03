#![allow(dead_code)]

use crate::axo_lexer::{Span, TokenKind};
use crate::axo_parser::Expr;

#[derive(Debug, Clone)]
pub enum Position {
    After,
    Before,
    Each,
    Between,
    As,
    Inside,
    Outside,
    Following,
    Preceding,
    Within,
    EndOf,
    StartOf,
    Around,
    Near,
    Throughout,
    AtMiddleOf,
    Adjacent,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyntaxRole {
    Target,
    Value,
    Name,
    Type,
    Body,
    Then,
    Else,
    Clause,
    Condition,
    Pattern,
    Parameter,
    Element,
    Field,
    Variant,

    Initialization,
    Assignment,
    Declaration,
    Definition,
    Implementation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextKind {
    // Top-level contexts
    Program,
    Module,

    // Statement contexts
    Statement,
    Expression,
    Block,

    // Expression-related contexts
    Binary,
    Unary,
    Literal,
    Identifier,
    Path,

    // Declaration/definition contexts
    Function,
    Variable,
    Constant,
    Type,
    Struct,
    Enum,
    Trait,
    Implementation,

    // Control flow contexts
    If,
    Else,
    Match,
    While,
    For,
    Loop,

    // Expression constructs
    Tuple,
    Array,
    Call,
    Index,
    MemberAccess,
    Closure,

    // Special statements
    Return,
    Break,
    Continue,

    // Other specialized contexts
    Macro,
    Attribute,
    Generic,
    Lifetime,

    // Error recovery context
    ErrorRecovery,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub kind: ContextKind,
    pub role: Option<SyntaxRole>,
    pub span: Span,
    pub parent: Option<Box<Context>>,
}

impl Context {
    pub fn new(kind: ContextKind, span: Span) -> Self {
        Self {
            kind,
            role: None,
            span,
            parent: None,
        }
    }

    pub fn with_role(kind: ContextKind, role: SyntaxRole, span: Span) -> Self {
        Self {
            kind,
            role: Some(role),
            span,
            parent: None,
        }
    }

    pub fn with_parent(mut self, parent: Context) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }

    /// Build a context chain description for error messages
    pub fn describe_chain(&self) -> String {
        let mut descriptions = Vec::new();

        let mut current = Some(self);
        while let Some(ctx) = current {
            let mut desc = format!("{:?}", ctx.kind);

            if let Some(role) = &ctx.role {
                desc = format!("{} ({:?})", desc, role);
            }

            descriptions.push(desc);
            current = ctx.parent.as_ref().map(|p| p.as_ref());
        }

        descriptions.join(" → ")
    }
}

impl core::fmt::Display for Position {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl core::fmt::Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

