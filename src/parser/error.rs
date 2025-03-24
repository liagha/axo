#![allow(dead_code)]
use crate::lexer::TokenKind;
use crate::parser::{Expr, Stmt};

pub enum SyntaxPosition {
    After,
    Before,
    Each,
}

pub enum SyntaxType {
    Token(TokenKind),
    Expr(Expr),
    Stmt(Stmt),
    Expression,
    Function,
    FunctionCall,
    FunctionName,
    FunctionParameter,
    FunctionParameters,
    Lambda,
    LambdaParameter,
    LambdaParameters,
    Tuple,
    TupleElement,
    TupleElements,
    Struct,
    StructName,
    StructField,
    StructFields,
    Enum,
    EnumName,
    EnumVariant,
    EnumVariants,
    EnumVariantName,
    Array,
    ArrayElement,
    ArrayElements,
    For,
    ForClause,
    Condition,
    FieldType,
    ParameterName,
    VariableDeclaration,
    ReturnValue,
    BreakValue,
    Continue,
    UnclosedPipe,
    UnclosedParen,
    UnclosedBracket,
    UnclosedBrace,
}

pub enum ParseError {
    ExpectedToken(TokenKind, SyntaxPosition, SyntaxType),
    ExpectedSyntax(SyntaxType),
    InvalidSyntax(String),
    UnexpectedEOF,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::ExpectedToken(token, position, syntax) => {
                write!(f, "Expected {} {} {}", token, position, syntax)
            }
            ParseError::ExpectedSyntax(syntax) => {
                write!(f, "Expected {}", syntax)
            }
            ParseError::InvalidSyntax(m) => {
                write!(f, "Invalid Syntax '{}'", m)
            }
            ParseError::UnexpectedEOF => {
                write!(f, "Unexpected end of file")
            }
        }
    }
}

impl core::fmt::Display for SyntaxPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SyntaxPosition::After => write!(f, "after"),
            SyntaxPosition::Before => write!(f, "before"),
            SyntaxPosition::Each => write!(f, "each"),
        }
    }
}

impl core::fmt::Display for SyntaxType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxType::Token(token) => write!(f, "{}", token),
            SyntaxType::Expr(expr) => write!(f, "{:?}", expr),
            SyntaxType::Stmt(stmt) => write!(f, "{:?}", stmt),
            SyntaxType::Expression => write!(f, "expression"),
            SyntaxType::Function => write!(f, "function"),
            SyntaxType::FunctionName => write!(f, "function name"),
            SyntaxType::FunctionParameter => write!(f, "function parameter"),
            SyntaxType::FunctionParameters => write!(f, "function parameters"),
            SyntaxType::Lambda => write!(f, "lambda"),
            SyntaxType::LambdaParameter => write!(f, "function parameter"),
            SyntaxType::LambdaParameters => write!(f, "function parameters"),
            SyntaxType::Tuple => write!(f, "tuple"),
            SyntaxType::TupleElement => write!(f, "tuple element"),
            SyntaxType::TupleElements => write!(f, "tuple elements"),
            SyntaxType::Struct => write!(f, "struct"),
            SyntaxType::StructName => write!(f, "struct name"),
            SyntaxType::StructField => write!(f, "struct field name"),
            SyntaxType::StructFields => write!(f, "struct fields"),
            SyntaxType::Enum => write!(f, "enum"),
            SyntaxType::EnumName => write!(f, "enum name"),
            SyntaxType::EnumVariant => write!(f, "enum variant"),
            SyntaxType::EnumVariants => write!(f, "enum variants"),
            SyntaxType::EnumVariantName => write!(f, "enum variant name"),
            SyntaxType::Array => write!(f, "array"),
            SyntaxType::ArrayElement => write!(f, "array element"),
            SyntaxType::ArrayElements => write!(f, "array elements"),
            SyntaxType::For => write!(f, "for"),
            SyntaxType::ForClause => write!(f, "for-clause"),
            SyntaxType::Condition => write!(f, "condition"),
            SyntaxType::FieldType => write!(f, "field type"),
            SyntaxType::ParameterName => write!(f, "parameter name"),
            SyntaxType::VariableDeclaration => write!(f, "variable declaration"),
            SyntaxType::ReturnValue => write!(f, "return value"),
            SyntaxType::BreakValue => write!(f, "break value"),
            SyntaxType::Continue => write!(f, "continue value"),
            SyntaxType::UnclosedPipe => write!(f, "unclosed pipe"),
            SyntaxType::UnclosedParen => write!(f, "unclosed paren"),
            SyntaxType::UnclosedBracket => write!(f, "unclosed bracket"),
            SyntaxType::UnclosedBrace => write!(f, "unclosed brace"),
            SyntaxType::FunctionCall => write!(f, "function call"),
        }
    }
}

impl core::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for ParseError {}
