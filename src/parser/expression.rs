#![allow(dead_code)]

use crate::lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::Parser;

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone)]
pub enum ExprKind {
    // Primary Expressions
    Literal(Token),
    Identifier(String),
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),

    // Composite Expressions
    Typed(Box<Expr>, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Invoke(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, Box<Expr>),
    Closure(Vec<Expr>, Box<Expr>),

    // Control Flow
    Conditional(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    While(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),

    // Declarations & Definitions
    Assignment(Box<Expr>, Box<Expr>),
    Definition(Box<Expr>, Option<Box<Expr>>),
    Struct(Box<Expr>, Vec<Expr>),
    StructDef(Box<Expr>, Vec<Expr>),
    Enum(Box<Expr>, Vec<Expr>),
    Function(Box<Expr>, Vec<Expr>, Box<Expr>),

    // Flow Control Statements
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,

    // Patterns
    Any, // _
}