# Axo Programming Language - API Documentation

## Table of Contents

1. [Overview](#overview)
2. [Core Compiler API](#core-compiler-api)
3. [Scanner Module](#scanner-module)
4. [Parser Module](#parser-module)
5. [Error Handling](#error-handling)
6. [Data Structures](#data-structures)
7. [Text Processing](#text-processing)
8. [Timer and Performance](#timer-and-performance)
9. [Logging](#logging)
10. [Formatting Utilities](#formatting-utilities)
11. [Examples](#examples)
12. [Best Practices](#best-practices)

## Overview

The Axo programming language compiler is designed with modularity and self-healing capabilities in mind. The architecture follows a pipeline approach with distinct phases: scanning (lexical analysis), parsing (syntax analysis), and resolution (semantic analysis).

### Key Features

- **Memory-safe compilation** without sacrificing performance
- **Self-healing error handling** with intelligent recovery mechanisms
- **Flexible type system** combining static and dynamic typing benefits
- **Modern toolchain** with built-in utilities
- **Cross-platform timer support** for performance analysis
- **Rich error diagnostics** with source code highlighting

## Core Compiler API

### `Compiler`

The main entry point for compilation operations.

```rust
pub struct Compiler {
    pub context: Context,
}

impl Compiler {
    /// Creates a new compiler instance for the given file
    pub fn new(path: &'static str, verbose: bool) -> Result<Self, CompilerError>;
    
    /// Compiles the source file through the default pipeline
    pub fn compile(&mut self) -> Result<(), CompilerError>;
    
    /// Compiles with a custom pipeline function
    pub fn compile_with<Function, Type>(&mut self, build_pipeline: Function) -> Result<Type, CompilerError>
    where Function: FnOnce(&mut Context) -> Result<Type, CompilerError>;
}
```

#### Usage Example

```rust
use axo::{Compiler, CompilerError};

fn main() -> Result<(), CompilerError> {
    let mut compiler = Compiler::new("example.axo", true)?;
    compiler.compile()?;
    Ok(())
}
```

### `Context`

Holds the compilation state and environment.

```rust
pub struct Context {
    pub verbose: bool,
    pub resolver: Resolver,
    pub path: &'static str,
    pub content: String,
}

impl Context {
    /// Creates a new compilation context
    pub fn new(file_path: &'static str, content: String) -> Self;
}
```

### `CompilerError`

Represents various compilation errors.

```rust
pub enum CompilerError {
    PathRequired,
    FileReadError(Error),
    ScanningFailed(Vec<ScanError>),
    ParsingFailed(Vec<ParseError>),
    ResolutionFailed(Vec<ResolveError>),
    ArgumentParsing(String),
    HelpRequested,
}
```

### Compilation Stages

The compiler uses a pipeline architecture with stages that implement the `Stage` trait:

```rust
pub trait Stage<Input, Output> {
    fn execute(&mut self, context: &mut Context, input: Input) -> Result<Output, CompilerError>;
}
```

#### Available Stages

- **`ScannerStage`**: Converts source code into tokens
- **`ParserStage`**: Converts tokens into AST elements
- **`ResolverStage`**: Performs semantic analysis

## Scanner Module

The scanner performs lexical analysis, converting source code into a stream of tokens.

### `Scanner`

```rust
pub struct Scanner;

impl Scanner {
    /// Creates a new scanner instance
    pub fn new(context: Context, input: String, file: &'static str) -> Scanner;
    
    /// Scans the input and returns tokens and errors
    pub fn scan(&mut self) -> (Vec<Token>, Vec<ScanError>);
    
    /// Inspects character stream for debugging
    pub fn inspect(start: Position, input: Vec<char>) -> Vec<Character>;
}
```

### `Token`

Represents a lexical token with location information.

```rust
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub enum TokenKind {
    Operator(OperatorKind),
    Punctuation(PunctuationKind),
    Identifier(String),
    Literal(LiteralKind),
    Keyword(KeywordKind),
    Comment(String),
    Whitespace,
    EOF,
}

impl Token {
    /// Creates a new token
    pub fn new(kind: TokenKind, span: Span) -> Self;
    
    /// Creates a token with default span
    pub fn from(kind: TokenKind) -> Self;
    
    /// Parses a token from string representation
    pub fn from_str(s: &str) -> Option<Self>;
}
```

### `Character`

Represents a character with position information.

```rust
pub struct Character {
    pub value: char,
    pub span: Span,
}

impl Character {
    pub fn new(value: char, span: Span) -> Self;
    pub fn is_digit(&self, radix: u32) -> bool;
    pub fn is_numeric(&self) -> bool;
    pub fn is_alphabetic(&self) -> bool;
    pub fn is_alphanumeric(&self) -> bool;
    pub fn is_whitespace(&self) -> bool;
}
```

### Operators

```rust
pub enum OperatorKind {
    // Arithmetic
    Plus, Minus, Multiply, Divide, Modulo, Power,
    
    // Comparison
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    
    // Logical
    And, Or, Not,
    
    // Bitwise
    BitAnd, BitOr, BitXor, BitNot, LeftShift, RightShift,
    
    // Assignment
    Assign, PlusAssign, MinusAssign, /* ... */
    
    // Access
    Dot, Arrow, DoubleColon,
    
    // Special
    Question, Colon, Semicolon,
    
    // Composite operators
    Composite(Vec<OperatorKind>),
}

impl OperatorKind {
    pub fn precedence(&self) -> Option<u8>;
    pub fn is_prefix(&self) -> bool;
    pub fn is_postfix(&self) -> bool;
    pub fn decompound(&self) -> Option<OperatorKind>;
}
```

### Usage Example

```rust
use axo::{Scanner, Context};

let context = Context::new("test.axo", "let x = 42;".to_string());
let mut scanner = Scanner::new(context, "let x = 42;".to_string(), "test.axo");
let (tokens, errors) = scanner.scan();

for token in tokens {
    println!("{:?}", token);
}
```

## Parser Module

The parser converts tokens into an Abstract Syntax Tree (AST).

### `Parser`

```rust
pub struct Parser;

impl Parser {
    /// Creates a new parser instance
    pub fn new(context: Context, tokens: Vec<Token>, file: &'static str) -> Parser;
    
    /// Parses tokens into AST elements
    pub fn parse(&mut self) -> (Vec<Element>, Vec<ParseError>);
}
```

### `Element`

The fundamental AST node representing any syntactic construct.

```rust
pub struct Element {
    pub kind: ElementKind,
    pub span: Span,
}

pub enum ElementKind {
    // Literals and identifiers
    Literal(TokenKind),
    Identifier(String),
    
    // Compound structures
    Group(Vec<Element>),        // (a, b, c)
    Sequence(Vec<Element>),     // (a; b; c)
    Collection(Vec<Element>),   // [a, b, c]
    Series(Vec<Element>),       // [a; b; c]
    Bundle(Vec<Element>),       // {a, b, c}
    Scope(Vec<Element>),        // {a; b; c}
    
    // Operations
    Binary { left: Box<Element>, operator: Token, right: Box<Element> },
    Unary { operator: Token, operand: Box<Element> },
    
    // Object-oriented constructs
    Member { object: Box<Element>, member: Box<Element> },
    Index { element: Box<Element>, index: Box<Element> },
    Invoke { target: Box<Element>, parameters: Box<Element> },
    Constructor { name: Box<Element>, body: Box<Element> },
    
    // Control flow
    Conditional { condition: Box<Element>, then: Box<Element>, alternate: Option<Box<Element>> },
    Cycle { condition: Option<Box<Element>>, body: Box<Element> },
    Iterate { clause: Box<Element>, body: Box<Element> },
    Match { target: Box<Element>, body: Box<Element> },
    
    // Other constructs
    Path { tree: Tree<Box<Element>> },
    Labeled { label: Box<Element>, element: Box<Element> },
    Assignment { target: Box<Element>, value: Box<Element> },
    Item(ItemKind),
    Return(Option<Box<Element>>),
    Break(Option<Box<Element>>),
    Skip(Option<Box<Element>>),
    Procedural(Box<Element>),
}

impl Element {
    /// Creates an empty element with the given span
    pub fn empty(span: Span) -> Element;
    
    /// Creates a new element with automatic transformations
    pub fn new(kind: ElementKind, span: Span) -> Element;
}
```

### `Item`

Represents top-level declarations.

```rust
pub enum ItemKind {
    Variable { target: Box<Element>, value: Option<Box<Element>>, ty: Option<Box<Element>>, mutable: bool },
    Function { signature: Box<Element>, body: Box<Element> },
    Type { name: Box<Element>, definition: Box<Element> },
    Module { name: Box<Element>, body: Box<Element> },
    Import { path: Box<Element>, alias: Option<Box<Element>> },
    Export { item: Box<Element> },
}
```

### Usage Example

```rust
use axo::{Parser, Context, Scanner};

// Scan first
let context = Context::new("test.axo", "fn main() { return 42; }".to_string());
let mut scanner = Scanner::new(context.clone(), context.content.clone(), "test.axo");
let (tokens, _) = scanner.scan();

// Then parse
let mut parser = Parser::new(context, tokens, "test.axo");
let (elements, errors) = parser.parse();

for element in elements {
    println!("{:?}", element);
}
```

## Error Handling

### `Error<K, N, H>`

Generic error type with rich diagnostic information.

```rust
pub struct Error<K, N = String, H = String> 
where 
    K: Display, 
    N: Display, 
    H: Display 
{
    pub kind: K,
    pub span: Span,
    pub note: Option<N>,
    pub hints: Vec<Hint<H>>,
}

impl<K: Display, N: Display, H: Display> Error<K, N, H> {
    /// Creates a new error
    pub fn new(kind: K, span: Span) -> Self;
    
    /// Adds help text to the error
    pub fn with_help(self, note: impl Into<N>) -> Self;
    
    /// Formats the error for display with source code context
    pub fn format(&self) -> (String, String);
}
```

### Error Types

- **`ScanError`**: Lexical analysis errors
- **`ParseError`**: Syntax analysis errors  
- **`ResolveError`**: Semantic analysis errors

### `Hint` and `Action`

Provide suggestions for fixing errors.

```rust
pub struct Hint<H = String> where H: Display {
    pub action: Action,
    pub message: H,
}

pub enum Action {
    Insert,
    Remove,
    Replace,
    Move,
    Suggest,
}
```

### Usage Example

```rust
use axo::{Error, Span, Position, Location};

let span = Span::new(
    Position::new(1, 5, Location::Memory),
    Position::new(1, 10, Location::Memory)
);

let error = Error::new("Unexpected token", span)
    .with_help("Try removing the extra semicolon");

let (message, details) = error.format();
println!("{}\n{}", message, details);
```

## Data Structures

### `Span` and `Position`

Track source code locations for error reporting.

```rust
pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
    pub location: Location,
}

pub enum Location {
    Memory,
    File(&'static str),
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self;
    pub fn mix(left: &Span, right: &Span) -> Span;
}

impl Position {
    pub fn new(line: usize, column: usize, location: Location) -> Self;
}
```

### `Spanned` Trait

Common interface for types that have source locations.

```rust
pub trait Spanned {
    fn span(&self) -> Span;
}
```

### `Tree<T>`

Generic tree data structure for hierarchical data.

```rust
pub struct Tree<T> {
    // Implementation details hidden
}

pub struct Node<T> {
    // Implementation details hidden  
}

impl<T> Tree<T> {
    pub fn with_root_node(root: Node<T>) -> Self;
    pub fn root_mut(&mut self) -> Option<&mut Node<T>>;
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self;
    pub fn with_children(value: T, children: Vec<Node<T>>) -> Self;
    pub fn add_value(&mut self, value: T);
    pub fn has_children(&self) -> bool;
    pub fn child_count(&self) -> usize;
    pub fn get_child_mut(&mut self, index: usize) -> Option<&mut Node<T>>;
}
```

## Text Processing

### Unicode Support

```rust
pub struct CharRange {
    // Implementation details
}

impl CharRange {
    /// Creates a closed range [start, stop]
    pub fn closed(start: char, stop: char) -> CharRange;
    
    /// Creates a half-open range [start, stop)
    pub fn open_right(start: char, stop: char) -> CharRange;
    
    /// Creates a half-open range (start, stop]
    pub fn open_left(start: char, stop: char) -> CharRange;
}
```

### Numeral Processing

The `axo_text::numeral` module provides utilities for parsing and handling numeric literals in various bases.

## Timer and Performance

### `Timer<T>`

High-precision timing for performance analysis.

```rust
pub struct Timer<T: TimeSource> {
    // Implementation details
}

pub trait TimeSource: Sized {
    type Error;
    fn now(&self) -> Result<u64, Self::Error>;
    fn frequency(&self) -> u64;
}

impl<T: TimeSource> Timer<T> {
    pub fn new(time_source: T) -> Self;
    pub fn start(&mut self) -> TimerResult<()>;
    pub fn stop(&mut self) -> TimerResult<u64>;
    pub fn pause(&mut self) -> TimerResult<u64>;
    pub fn resume(&mut self) -> TimerResult<()>;
    pub fn reset(&mut self);
    pub fn elapsed(&self) -> TimerResult<u64>;
    pub fn lap(&mut self) -> TimerResult<u64>;
    pub fn laps(&self) -> &[u64];
    pub fn state(&self) -> TimerState;
    
    // Time conversion utilities
    pub fn to_seconds(&self, time: u64) -> u64;
    pub fn to_milliseconds(&self, time: u64) -> u64;
    pub fn to_microseconds(&self, time: u64) -> u64;
    pub fn to_nanoseconds(&self, time: u64) -> u64;
}
```

### Time Sources

Platform-specific time sources for accurate measurements:

```rust
pub struct CPUCycleSource;        // x86_64 CPU cycles
pub struct ARMGenericTimerSource; // ARM generic timer
pub struct RISCVCycleSource;      // RISC-V cycle counter
pub struct DummyTimeSource;       // For testing
```

### Timer States

```rust
pub enum TimerState {
    Stopped,
    Running,
    Paused,
}
```

### Specialized Timers

```rust
pub struct CallbackTimer<T: TimeSource, C: TimerCallback>;
pub struct CountdownTimer<T: TimeSource>;
```

### Usage Example

```rust
use axo::{Timer, TIMERSOURCE};
use core::time::Duration;

let timer = Timer::new(TIMERSOURCE);
timer.start().unwrap();

// ... perform work ...

let elapsed_ns = timer.elapsed().unwrap();
let duration = Duration::from_nanos(elapsed_ns);
println!("Operation took: {}s", duration.as_secs_f64());
```

## Logging

### `Logger`

Structured logging system with configurable output format.

```rust
pub struct Logger {
    // Implementation details
}

pub struct LogPlan {
    // Implementation details
}

pub enum LogInfo {
    Time,
    Level,
    Message,
    Target,
    Module,
}

impl Logger {
    pub fn new(level: Level, plan: LogPlan) -> Self;
    pub fn init(self) -> Result<(), SetLoggerError>;
}

impl LogPlan {
    pub fn new(components: Vec<LogInfo>) -> Self;
    pub fn with_separator(self, separator: String) -> Self;
    pub fn default() -> Self;
    pub fn simple() -> Self;
    pub fn detailed() -> Self;
}
```

### Usage Example

```rust
use axo::{Logger, LogPlan, LogInfo};
use log::Level;

let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message])
    .with_separator(" | ".to_string());

let logger = Logger::new(Level::Debug, plan);
logger.init().expect("Failed to initialize logger");

log::info!("Compilation started");
```

## Formatting Utilities

### Token Formatting

```rust
/// Formats a slice of tokens for display
pub fn format_tokens(tokens: &[Token]) -> String;

/// Indents each line of a string
pub fn indent(string: &String) -> String;

/// Prints usage information for the CLI
pub fn print_usage(program: &str);
```

### Usage Example

```rust
use axo::{format_tokens, indent};

let formatted = format_tokens(&tokens);
println!("Tokens:\n{}", indent(&formatted));
```

## Examples

### Complete Compilation Example

```rust
use axo::{Compiler, CompilerError, Timer, TIMERSOURCE};
use core::time::Duration;

fn compile_file(path: &'static str, verbose: bool) -> Result<(), CompilerError> {
    let timer = Timer::new(TIMERSOURCE);
    
    let mut compiler = Compiler::new(path, verbose)?;
    compiler.compile()?;
    
    if verbose {
        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        println!("Compilation completed in {}s", duration.as_secs_f64());
    }
    
    Ok(())
}
```

### Custom Pipeline Example

```rust
use axo::{Compiler, Context, ScannerStage, ParserStage, Stage};

fn analyze_only(path: &'static str) -> Result<(), CompilerError> {
    let mut compiler = Compiler::new(path, false)?;
    
    compiler.compile_with(|context| {
        // Custom pipeline: scan and parse only, skip resolution
        let mut scanner_stage = ScannerStage;
        let mut parser_stage = ParserStage;
        
        let tokens = scanner_stage.execute(context, ())?;
        let elements = parser_stage.execute(context, tokens)?;
        
        println!("Found {} top-level elements", elements.len());
        Ok(())
    })
}
```

### Error Handling Example

```rust
use axo::{Compiler, CompilerError};

fn safe_compile(path: &'static str) {
    match Compiler::new(path, true) {
        Ok(mut compiler) => {
            match compiler.compile() {
                Ok(()) => println!("Compilation successful!"),
                Err(CompilerError::ScanningFailed(errors)) => {
                    eprintln!("Lexical errors found:");
                    for error in errors {
                        let (msg, details) = error.format();
                        eprintln!("{}\n{}", msg, details);
                    }
                }
                Err(CompilerError::ParsingFailed(errors)) => {
                    eprintln!("Syntax errors found:");
                    for error in errors {
                        let (msg, details) = error.format();
                        eprintln!("{}\n{}", msg, details);
                    }
                }
                Err(e) => eprintln!("Compilation failed: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to create compiler: {}", e),
    }
}
```

## Best Practices

### 1. Error Handling

- Always handle `CompilerError` appropriately
- Use the `format()` method on errors for user-friendly output
- Consider different error types for different recovery strategies

### 2. Performance Monitoring

- Use `Timer` to measure compilation phases
- Enable verbose mode for development to see detailed timing
- Consider using `CallbackTimer` for long-running operations

### 3. Memory Management

- The compiler uses owned data structures to avoid lifetime issues
- Large ASTs are stored in `Box<Element>` to minimize stack usage
- Clone operations are used sparingly and only when necessary

### 4. Threading and Concurrency

- The current implementation is single-threaded
- All major types implement `Send + Sync` for future parallel processing
- Use `Arc<Mutex<T>>` wrapper if sharing compiler state across threads

### 5. Custom Stages

- Implement the `Stage` trait for custom compilation phases
- Use the `pipeline!` macro for chaining stages
- Handle errors gracefully to maintain the self-healing design philosophy

### 6. Debugging

- Enable verbose mode during development
- Use `Scanner::inspect()` for character-level debugging
- The error system provides rich source context for debugging

### 7. Language Extension

- New token types can be added to `TokenKind`
- New AST nodes can be added to `ElementKind`
- New operators can be added to `OperatorKind`
- Follow the existing patterns for automatic transformations in `Element::new()`

---

This documentation covers the public API surface of the Axo programming language compiler. For implementation details and private APIs, refer to the source code documentation and inline comments.