#![allow(dead_code)]

use crate::axo_lexer::TokenKind;
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

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum Context {
    Default,
    Program,
    Statement,
    Expression,
    Binary,
    Unary,

    Definition,
    DefinitionTarget,
    DefinitionValue,

    Assignment,
    AssignmentTarget,
    AssignmentValue,

    Clause,

    Conditional,
    ConditionalThen,
    ConditionalElse,

    While,
    WhileBody,

    For,
    ForBody,

    Match,
    MatchValue,
    MatchPatterns,

    Struct,
    StructName,
    StructFields,
    StructDeclaration,

    Enum,
    EnumName,
    EnumVariants,
    EnumDeclaration,

    Tuple,
    TupleElements,

    Array,
    ArrayElements,

    Closure,
    ClosureParameters,
    ClosureBody,

    Invoke,
    InvokeParameters,

    Index,
    IndexValue,

    Function,
    FunctionName,
    FunctionParameters,
    FunctionBody,
    FunctionDeclaration,

    Return,
    ReturnValue,

    Break,
    BreakValue,

    Continue,
    ContinueValue,

    Macro,
    MacroName,
    MacroParameters,
    MacroBody,

    Block,
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

