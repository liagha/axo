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
    Use,
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

    pub fn describe_chain(&self) -> String {
        let mut descriptions = Vec::new();
        let mut current = Some(self);

        while let Some(ctx) = current {
            let kind_desc = match ctx.kind {
                ContextKind::Program => "program",
                ContextKind::Module => "module",
                ContextKind::Statement => "statement",
                ContextKind::Expression => "expression",
                ContextKind::Block => "block",
                ContextKind::Binary => "binary operation",
                ContextKind::Unary => "unary operation",
                ContextKind::Literal => "literal",
                ContextKind::Identifier => "identifier",
                ContextKind::Path => "path",
                ContextKind::Function => "function",
                ContextKind::Variable => "variable",
                ContextKind::Constant => "constant",
                ContextKind::Type => "type",
                ContextKind::Struct => "struct",
                ContextKind::Enum => "enum",
                ContextKind::Trait => "trait",
                ContextKind::Implementation => "implementation",
                ContextKind::If => "if statement",
                ContextKind::Else => "else clause",
                ContextKind::Match => "match expression",
                ContextKind::While => "while loop",
                ContextKind::For => "for loop",
                ContextKind::Loop => "loop",
                ContextKind::Tuple => "tuple",
                ContextKind::Array => "array",
                ContextKind::Call => "function call",
                ContextKind::Index => "index access",
                ContextKind::MemberAccess => "member access",
                ContextKind::Closure => "closure",
                ContextKind::Use => "use",
                ContextKind::Return => "return statement",
                ContextKind::Break => "break statement",
                ContextKind::Continue => "continue statement",
                ContextKind::Macro => "macro",
                ContextKind::Attribute => "attribute",
                ContextKind::Generic => "generic",
                ContextKind::Lifetime => "lifetime",
                ContextKind::ErrorRecovery => "error recovery",
            };

            let role_desc = ctx.role.as_ref().map(|role| match role {
                SyntaxRole::Target => "target",
                SyntaxRole::Value => "value",
                SyntaxRole::Name => "name",
                SyntaxRole::Type => "type",
                SyntaxRole::Body => "body",
                SyntaxRole::Then => "then branch",
                SyntaxRole::Else => "else branch",
                SyntaxRole::Clause => "clause",
                SyntaxRole::Condition => "condition",
                SyntaxRole::Pattern => "pattern",
                SyntaxRole::Parameter => "parameter",
                SyntaxRole::Element => "element",
                SyntaxRole::Field => "field",
                SyntaxRole::Variant => "variant",
                SyntaxRole::Initialization => "initialization",
                SyntaxRole::Assignment => "assignment",
                SyntaxRole::Declaration => "declaration",
                SyntaxRole::Definition => "definition",
                SyntaxRole::Implementation => "implementation",
            });

            let full_desc = if let Some(role) = role_desc {
                format!("{} in {}", role, kind_desc)
            } else {
                kind_desc.to_string()
            };

            descriptions.push(full_desc);
            current = ctx.parent.as_ref().map(|p| p.as_ref());
        }

        if descriptions.len() > 1 {
            // For multiple contexts, show the chain of contexts
            descriptions.join(" within ")
        } else {
            // For single context, just return the description
            descriptions.pop().unwrap_or_else(|| "unknown context".to_string())
        }
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

