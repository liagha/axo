#![allow(dead_code)]

use crate::axo_lexer::{TokenKind, Token};
use crate::axo_parser::{Expr};
use crate::axo_parser::state::{Position, Context};

pub enum ParseError {
    // Renamed for clarity
    ExpectedTokenNotFound(TokenKind, Position, Context),
    UnexpectedToken(Token, Position, Context),
    UnexpectedExpression(Expr, Position, Context),
    MissingSyntaxElement(Context),
    InvalidSyntaxPattern(String),
    UnimplementedFeature,
    UnexpectedEndOfFile,

    // New error types
    MismatchedDelimiter(TokenKind, TokenKind),
    UnclosedDelimiter(TokenKind, usize, usize), // token, line, column
    EmptyConstruct(Context),
    ConflictingSyntax(Context, Context),
    AmbiguousSyntax(String),
    ExpectedSeparator(TokenKind, Context),
    MissingSemicolon(usize, usize), // line, column
    ExtraSemicolon(usize, usize),   // line, column
    InvalidEscapeSequence(String, usize, usize), // sequence, line, column
    UnterminatedString(usize, usize), // line, column
    UnterminatedCharLiteral(usize, usize), // line, column
    EmptyCharLiteral(usize, usize), // line, column
    MultipleCharsInCharLiteral(usize, usize), // line, column
    InvalidIntegerLiteral(String, usize, usize), // value, line, column
    InvalidFloatLiteral(String, usize, usize), // value, line, column
    OverflowInLiteral(String, usize, usize), // value, line, column
    InvalidOperatorInContext(TokenKind, Context),
    UnexpectedKeyword(TokenKind, Position, Context),
    MisplacedKeyword(TokenKind, Position, Context),
    RedundantTokens(Vec<Token>),
    InvalidStartOfExpression(Token),
    InvalidStartOfStatement(Token),
    NestedBlockInExpressionContext,
    ExpectedIdentifier(Position, Context),
    ReservedIdentifier(String),
    DuplicatePatternBinding(String), // name
    PatternMatchingError(String),
    InvalidRangeBounds,
    IncompleteExpression(Context),
    DisallowedExpression(Context, Position, Context), // expr, position, context
    ExpressionWithoutEffect,
    InvalidPrefix(TokenKind, Context),
    InvalidPostfix(TokenKind, Context),
    InconsistentIndentation(usize, usize, usize), // expected, found, line
    LonelyElseClause,
    MissingArmDelimiter,
    ExpectedPathSeparator,
    ExpectedParameterList,
    ParameterWithoutType,
    IncompletePatternMatch,
    EmptyBlockInNonBlockContext,
    MacroInvocationError(String),
    SyntaxDepthExceeded(usize), // max depth
    InvalidSyntaxRecovery,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // Updated existing error formatters
            ParseError::ExpectedTokenNotFound(token, position, syntax) => {
                write!(f, "Expected {} {} {}", token, position, syntax)
            }
            ParseError::UnexpectedToken(token, position, syntax) => {
                write!(f, "Unexpected token {:?} {} {}", token, position, syntax)
            }
            ParseError::UnexpectedExpression(expr, position, syntax) => {
                write!(f, "Unexpected expression {:?} {} {}", expr, position, syntax)
            }
            ParseError::MissingSyntaxElement(syntax) => {
                write!(f, "Missing {}", syntax)
            }
            ParseError::InvalidSyntaxPattern(m) => {
                write!(f, "Invalid syntax pattern: '{}'", m)
            }
            ParseError::UnimplementedFeature => {
                write!(f, "Unimplemented axo_parser feature")
            }
            ParseError::UnexpectedEndOfFile => {
                write!(f, "Unexpected end of file")
            }

            // Formatters for new error types
            ParseError::MismatchedDelimiter(opening, closing) => {
                write!(f, "Mismatched delimiter: expected closing '{}' to match opening '{}'", closing, opening)
            }
            ParseError::UnclosedDelimiter(token, line, col) => {
                write!(f, "Unclosed delimiter '{}' at line {}, column {}", token, line, col)
            }
            ParseError::EmptyConstruct(syntax) => {
                write!(f, "Empty {} is not allowed in this context", syntax)
            }
            ParseError::ConflictingSyntax(syntax1, syntax2) => {
                write!(f, "Conflicting syntax: {} conflicts with {}", syntax1, syntax2)
            }
            ParseError::AmbiguousSyntax(desc) => {
                write!(f, "Ambiguous syntax: {}", desc)
            }
            ParseError::ExpectedSeparator(token, syntax) => {
                write!(f, "Expected separator '{}' in {}", token, syntax)
            }
            ParseError::MissingSemicolon(line, col) => {
                write!(f, "Missing semicolon at line {}, column {}", line, col)
            }
            ParseError::ExtraSemicolon(line, col) => {
                write!(f, "Unnecessary semicolon at line {}, column {}", line, col)
            }
            ParseError::InvalidEscapeSequence(seq, line, col) => {
                write!(f, "Invalid escape sequence '{}' at line {}, column {}", seq, line, col)
            }
            ParseError::UnterminatedString(line, col) => {
                write!(f, "Unterminated string literal starting at line {}, column {}", line, col)
            }
            ParseError::UnterminatedCharLiteral(line, col) => {
                write!(f, "Unterminated character literal starting at line {}, column {}", line, col)
            }
            ParseError::EmptyCharLiteral(line, col) => {
                write!(f, "Empty character literal at line {}, column {}", line, col)
            }
            ParseError::MultipleCharsInCharLiteral(line, col) => {
                write!(f, "Multiple characters in character literal at line {}, column {}", line, col)
            }
            ParseError::InvalidIntegerLiteral(val, line, col) => {
                write!(f, "Invalid integer literal '{}' at line {}, column {}", val, line, col)
            }
            ParseError::InvalidFloatLiteral(val, line, col) => {
                write!(f, "Invalid float literal '{}' at line {}, column {}", val, line, col)
            }
            ParseError::OverflowInLiteral(val, line, col) => {
                write!(f, "Numeric literal '{}' overflows its type at line {}, column {}", val, line, col)
            }
            ParseError::InvalidOperatorInContext(op, context) => {
                write!(f, "Invalid operator '{}' in {} context", op, context)
            }
            ParseError::UnexpectedKeyword(keyword, position, syntax) => {
                write!(f, "Unexpected keyword '{}' {} {}", keyword, position, syntax)
            }
            ParseError::MisplacedKeyword(keyword, position, syntax) => {
                write!(f, "Misplaced keyword '{}' {} {}", keyword, position, syntax)
            }
            ParseError::RedundantTokens(tokens) => {
                write!(f, "Redundant tokens: {:?}", tokens)
            }
            ParseError::InvalidStartOfExpression(token) => {
                write!(f, "Invalid start of expression: {:?}", token)
            }
            ParseError::InvalidStartOfStatement(token) => {
                write!(f, "Invalid start of statement: {:?}", token)
            }
            ParseError::NestedBlockInExpressionContext => {
                write!(f, "Unexpected nested block in expression context")
            }
            ParseError::ExpectedIdentifier(position, syntax) => {
                write!(f, "Expected identifier {} {}", position, syntax)
            }
            ParseError::ReservedIdentifier(name) => {
                write!(f, "Reserved identifier '{}' cannot be used here", name)
            }
            ParseError::DuplicatePatternBinding(name) => {
                write!(f, "Duplicate binding '{}' in pattern", name)
            }
            ParseError::PatternMatchingError(desc) => {
                write!(f, "Pattern matching error: {}", desc)
            }
            ParseError::InvalidRangeBounds => {
                write!(f, "Invalid range bounds")
            }
            ParseError::IncompleteExpression(syntax) => {
                write!(f, "Incomplete {} expression", syntax)
            }
            ParseError::DisallowedExpression(expr, position, context) => {
                write!(f, "{:?} not allowed {} {}", expr, position, context)
            }
            ParseError::ExpressionWithoutEffect => {
                write!(f, "Expression without effect")
            }
            ParseError::InvalidPrefix(token, syntax) => {
                write!(f, "Invalid prefix '{}' for {}", token, syntax)
            }
            ParseError::InvalidPostfix(token, syntax) => {
                write!(f, "Invalid postfix '{}' for {}", token, syntax)
            }
            ParseError::InconsistentIndentation(expected, found, line) => {
                write!(f, "Inconsistent indentation at line {}: expected {}, found {}", line, expected, found)
            }
            ParseError::LonelyElseClause => {
                write!(f, "Else clause without preceding if statement")
            }
            ParseError::MissingArmDelimiter => {
                write!(f, "Missing delimiter between match arms")
            }
            ParseError::ExpectedPathSeparator => {
                write!(f, "Expected path separator '::'")
            }
            ParseError::ExpectedParameterList => {
                write!(f, "Expected parameter list")
            }
            ParseError::ParameterWithoutType => {
                write!(f, "Parameter missing type annotation")
            }
            ParseError::IncompletePatternMatch => {
                write!(f, "Incomplete pattern match")
            }
            ParseError::EmptyBlockInNonBlockContext => {
                write!(f, "Empty block in non-block context")
            }
            ParseError::MacroInvocationError(desc) => {
                write!(f, "Macro invocation error: {}", desc)
            }
            ParseError::SyntaxDepthExceeded(depth) => {
                write!(f, "Maximum syntax nesting depth exceeded ({})", depth)
            }
            ParseError::InvalidSyntaxRecovery => {
                write!(f, "Failed to recover from previous syntax errors")
            }
        }
    }
}

impl core::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for ParseError {}