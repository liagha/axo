#![allow(dead_code)]
use crate::lexer::{TokenKind, Token};
use crate::parser::{Expr};

pub enum SyntaxPosition {
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
}

#[derive(Debug, Clone)]
pub enum SyntaxType {
    // Existing syntax types
    Token(TokenKind),
    Expr(Expr),
    Expression,
    Function,
    FunctionCall,
    FunctionName,
    FunctionParameter,
    FunctionParameters,
    FunctionDeclaration,
    Block,
    Closure,
    ClosureParameter,
    ClosureParameters,
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

    // New syntax types
    If,
    IfCondition,
    ElseClause,
    ElseIfClause,
    Loop,
    While,
    WhileCondition,
    Match,
    MatchArm,
    MatchArms,
    MatchPattern,
    MatchExpression,
    Range,
    RangeStart,
    RangeEnd,
    RangeStep,
    Path,
    PathSegment,
    Identifier,
    Literal,
    Operator,
    BinaryOperator,
    UnaryOperator,
    AssignmentOperator,
    ComparisonOperator,
    LogicalOperator,
    Type,
    TypeAnnotation,
    GenericType,
    GenericParameters,
    GenericArguments,
    Trait,
    TraitName,
    TraitMethod,
    TraitMethods,
    Implementation,
    Use,
    UseItem,
    UseTree,
    UseAlias,
    Module,
    ModuleName,
    ModuleItem,
    MacroCall,
    MacroRule,
    MacroPattern,
    MacroExpansion,
    Attribute,
    AttributeArgument,
    Comment,
    DocComment,
    Statement,
    Let,
    LetPattern,
    Assignment,
    AssignmentTarget,
    AssignmentValue,
    MatchGuard,
    Lifetime,
    LifetimeParameter,
    TypeConstraint,
    TypeBound,
    Where,
    WhereClause,
    TraitBound,
    ImplTrait,
    DynTrait,
    Reference,
    Dereference,
    MethodCall,
    ChainedMethodCall,
    IndexExpression,
    FieldAccess,
    TupleIndex,
    SlicePattern,
    RestPattern,
    StructPattern,
    TupleStructPattern,
    GroupedExpression,
}

pub enum ParseError {
    // Renamed for clarity
    ExpectedTokenNotFound(TokenKind, SyntaxPosition, SyntaxType),
    UnexpectedToken(Token, SyntaxPosition, SyntaxType),
    UnexpectedExpression(Expr, SyntaxPosition, SyntaxType),
    MissingSyntaxElement(SyntaxType),
    InvalidSyntaxPattern(String),
    UnimplementedFeature,
    UnexpectedEndOfFile,

    // New error types
    MismatchedDelimiter(TokenKind, TokenKind),
    UnclosedDelimiter(TokenKind, usize, usize), // token, line, column
    EmptyConstruct(SyntaxType),
    ConflictingSyntax(SyntaxType, SyntaxType),
    AmbiguousSyntax(String),
    ExpectedSeparator(TokenKind, SyntaxType),
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
    InvalidOperatorInContext(TokenKind, SyntaxType),
    UnexpectedKeyword(TokenKind, SyntaxPosition, SyntaxType),
    MisplacedKeyword(TokenKind, SyntaxPosition, SyntaxType),
    RedundantTokens(Vec<Token>),
    InvalidStartOfExpression(Token),
    InvalidStartOfStatement(Token),
    NestedBlockInExpressionContext,
    ExpectedIdentifier(SyntaxPosition, SyntaxType),
    ReservedIdentifier(String),
    DuplicatePatternBinding(String), // name
    PatternMatchingError(String),
    InvalidRangeBounds,
    IncompleteExpression(SyntaxType),
    DisallowedExpression(SyntaxType, SyntaxPosition, SyntaxType), // expr, position, context
    ExpressionWithoutEffect,
    InvalidPrefix(TokenKind, SyntaxType),
    InvalidPostfix(TokenKind, SyntaxType),
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
                write!(f, "Unimplemented parser feature")
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

impl core::fmt::Display for SyntaxPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SyntaxPosition::After => write!(f, "after"),
            SyntaxPosition::Before => write!(f, "before"),
            SyntaxPosition::Each => write!(f, "each"),
            SyntaxPosition::Between => write!(f, "between"),
            SyntaxPosition::As => write!(f, "as"),
            SyntaxPosition::Inside => write!(f, "inside"),
            SyntaxPosition::Outside => write!(f, "outside"),
            SyntaxPosition::Following => write!(f, "following"),
            SyntaxPosition::Preceding => write!(f, "preceding"),
            SyntaxPosition::Within => write!(f, "within"),
            SyntaxPosition::EndOf => write!(f, "at the end of"),
            SyntaxPosition::StartOf => write!(f, "at the start of"),
        }
    }
}

impl core::fmt::Display for SyntaxType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Existing syntax types
            SyntaxType::Token(token) => write!(f, "{}", token),
            SyntaxType::Expr(expr) => write!(f, "{:?}", expr),
            SyntaxType::Expression => write!(f, "expression"),
            SyntaxType::Function => write!(f, "function"),
            SyntaxType::FunctionCall => write!(f, "function call"),
            SyntaxType::FunctionName => write!(f, "function name"),
            SyntaxType::FunctionParameter => write!(f, "function parameter"),
            SyntaxType::FunctionParameters => write!(f, "function parameters"),
            SyntaxType::FunctionDeclaration => write!(f, "function declaration"),
            SyntaxType::Block => write!(f, "block"),
            SyntaxType::Closure => write!(f, "closure"),
            SyntaxType::ClosureParameter => write!(f, "closure parameter"),
            SyntaxType::ClosureParameters => write!(f, "closure parameters"),
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

            // New syntax types
            SyntaxType::If => write!(f, "if statement"),
            SyntaxType::IfCondition => write!(f, "if condition"),
            SyntaxType::ElseClause => write!(f, "else clause"),
            SyntaxType::ElseIfClause => write!(f, "else if clause"),
            SyntaxType::Loop => write!(f, "loop"),
            SyntaxType::While => write!(f, "while loop"),
            SyntaxType::WhileCondition => write!(f, "while condition"),
            SyntaxType::Match => write!(f, "match expression"),
            SyntaxType::MatchArm => write!(f, "match arm"),
            SyntaxType::MatchArms => write!(f, "match arms"),
            SyntaxType::MatchPattern => write!(f, "match pattern"),
            SyntaxType::MatchExpression => write!(f, "match expression"),
            SyntaxType::Range => write!(f, "range"),
            SyntaxType::RangeStart => write!(f, "range start"),
            SyntaxType::RangeEnd => write!(f, "range end"),
            SyntaxType::RangeStep => write!(f, "range step"),
            SyntaxType::Path => write!(f, "path"),
            SyntaxType::PathSegment => write!(f, "path segment"),
            SyntaxType::Identifier => write!(f, "identifier"),
            SyntaxType::Literal => write!(f, "literal"),
            SyntaxType::Operator => write!(f, "operator"),
            SyntaxType::BinaryOperator => write!(f, "binary operator"),
            SyntaxType::UnaryOperator => write!(f, "unary operator"),
            SyntaxType::AssignmentOperator => write!(f, "assignment operator"),
            SyntaxType::ComparisonOperator => write!(f, "comparison operator"),
            SyntaxType::LogicalOperator => write!(f, "logical operator"),
            SyntaxType::Type => write!(f, "type"),
            SyntaxType::TypeAnnotation => write!(f, "type annotation"),
            SyntaxType::GenericType => write!(f, "generic type"),
            SyntaxType::GenericParameters => write!(f, "generic parameters"),
            SyntaxType::GenericArguments => write!(f, "generic arguments"),
            SyntaxType::Trait => write!(f, "trait"),
            SyntaxType::TraitName => write!(f, "trait name"),
            SyntaxType::TraitMethod => write!(f, "trait method"),
            SyntaxType::TraitMethods => write!(f, "trait methods"),
            SyntaxType::Implementation => write!(f, "implementation"),
            SyntaxType::Use => write!(f, "use declaration"),
            SyntaxType::UseItem => write!(f, "use item"),
            SyntaxType::UseTree => write!(f, "use tree"),
            SyntaxType::UseAlias => write!(f, "use alias"),
            SyntaxType::Module => write!(f, "module"),
            SyntaxType::ModuleName => write!(f, "module name"),
            SyntaxType::ModuleItem => write!(f, "module item"),
            SyntaxType::MacroCall => write!(f, "macro call"),
            SyntaxType::MacroRule => write!(f, "macro rule"),
            SyntaxType::MacroPattern => write!(f, "macro pattern"),
            SyntaxType::MacroExpansion => write!(f, "macro expansion"),
            SyntaxType::Attribute => write!(f, "attribute"),
            SyntaxType::AttributeArgument => write!(f, "attribute argument"),
            SyntaxType::Comment => write!(f, "comment"),
            SyntaxType::DocComment => write!(f, "documentation comment"),
            SyntaxType::Statement => write!(f, "statement"),
            SyntaxType::Let => write!(f, "let binding"),
            SyntaxType::LetPattern => write!(f, "let pattern"),
            SyntaxType::Assignment => write!(f, "assignment"),
            SyntaxType::AssignmentTarget => write!(f, "assignment target"),
            SyntaxType::AssignmentValue => write!(f, "assignment value"),
            SyntaxType::MatchGuard => write!(f, "match guard"),
            SyntaxType::Lifetime => write!(f, "lifetime"),
            SyntaxType::LifetimeParameter => write!(f, "lifetime parameter"),
            SyntaxType::TypeConstraint => write!(f, "type constraint"),
            SyntaxType::TypeBound => write!(f, "type bound"),
            SyntaxType::Where => write!(f, "where clause"),
            SyntaxType::WhereClause => write!(f, "where clause"),
            SyntaxType::TraitBound => write!(f, "trait bound"),
            SyntaxType::ImplTrait => write!(f, "impl trait"),
            SyntaxType::DynTrait => write!(f, "dyn trait"),
            SyntaxType::Reference => write!(f, "reference"),
            SyntaxType::Dereference => write!(f, "dereference"),
            SyntaxType::MethodCall => write!(f, "method call"),
            SyntaxType::ChainedMethodCall => write!(f, "chained method call"),
            SyntaxType::IndexExpression => write!(f, "index expression"),
            SyntaxType::FieldAccess => write!(f, "field access"),
            SyntaxType::TupleIndex => write!(f, "tuple index"),
            SyntaxType::SlicePattern => write!(f, "slice pattern"),
            SyntaxType::RestPattern => write!(f, "rest pattern"),
            SyntaxType::StructPattern => write!(f, "struct pattern"),
            SyntaxType::TupleStructPattern => write!(f, "tuple struct pattern"),
            SyntaxType::GroupedExpression => write!(f, "grouped expression"),
        }
    }
}

impl core::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for ParseError {}