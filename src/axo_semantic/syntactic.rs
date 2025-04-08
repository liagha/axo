#![allow(dead_code)]

use std::fs::read_to_string;
use broccli::{Color, TextStyle};
use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, ItemKind};

#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (msg, details) = self.format();

        write!(f, "{} \n {}", msg, details)
    }
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
        }
    }

    pub fn format(&self) -> (String, String) {
        let source_code = read_to_string(self.span.file.clone()).unwrap();
        let lines: Vec<&str> = source_code.lines().collect();
        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&format!("error: {}", self.kind.to_string().colorize(Color::Red).bold()));

        let (line_start, column_start) = self.span.start;
        let (line_end, column_end) = self.span.end;

        details.push_str(&format!(" --> {}:{}:{}\n",
                                  self.span.file.display(),
                                  line_start,
                                  column_start
        ).colorize(Color::Blue));

        // Calculate maximum line number width safely
        let max_line_number = line_end.max(line_start).max(1); // Ensure at least 1
        let line_number_width = max_line_number.to_string().len();

        // Calculate bounds safely
        let start_line = line_start.saturating_sub(3).max(1);
        let end_line = (line_end + 3).min(lines.len()).max(1);

        for line_idx in start_line..=end_line {
            let line_content_idx = line_idx.saturating_sub(1);
            if line_content_idx >= lines.len() { break; }

            let line_content = lines[line_content_idx];
            let line_num_str = if line_content.is_empty() {
                " ".repeat(line_number_width)
            } else {
                line_idx.to_string()
            };

            details.push_str(&format!("{:>width$} | {}\n",
                                      line_num_str.colorize(Color::Blue),
                                      line_content,
                                      width = line_number_width
            ));

            if line_idx >= line_start && line_idx <= line_end {
                let line_length = line_content.chars().count();
                let start_col = if line_idx == line_start {
                    column_start.saturating_sub(1).min(line_length)
                } else { 0 };
                let end_col = if line_idx == line_end {
                    column_end.saturating_sub(1).min(line_length)
                } else { line_length };

                let caret_count = if start_col <= end_col {
                    end_col.saturating_sub(start_col) + 1
                } else {
                    1
                };

                let underline = format!("{:width$}{}",
                                        "",
                                        "^".repeat(caret_count),
                                        width = start_col
                );

                details.push_str(&format!("{:>width$} | {}\n",
                                          " ".repeat(line_number_width),
                                          underline.colorize(Color::Red).bold(),
                                          width = line_number_width
                ));
            }
        }

        (messages, details)
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    // Variable and binding errors
    UndefinedVariable(String),
    DuplicateDefinition(String),
    InvalidAssignmentTarget,

    // Type errors
    TypeMismatch,
    InvalidTypeAnnotation,

    // Structural errors
    InvalidArrayElement,
    InvalidStructField,
    MissingRequiredField,

    // Control flow errors
    ReturnOutsideFunction,
    BreakOutsideLoop,
    ContinueOutsideLoop,

    // Expression errors
    InvalidBinaryOperation,
    InvalidUnaryOperation,
    InvalidFunctionCall,
    InvalidPathExpression,
    InvalidMemberAccess,

    // Item errors
    InvalidItemDeclaration,

    // Expression context errors
    ExpectedExpression,
    ExpectedStatement,

    // Other errors
    SyntaxError(String),
    Other(String),
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::UndefinedVariable(var) => {
                write!(f, "Undefined variable: {}", var)
            }
            ErrorKind::DuplicateDefinition(var) => {
                write!(f, "Duplicate definition: {}", var)
            }
            ErrorKind::InvalidAssignmentTarget => {
                write!(f, "Invalid assignment target")
            }
            ErrorKind::TypeMismatch => {
                write!(f, "Type mismatch")
            }
            ErrorKind::InvalidTypeAnnotation => {
                write!(f, "Invalid type annotation")
            }
            ErrorKind::InvalidArrayElement => {
                write!(f, "Invalid array element")
            }
            ErrorKind::InvalidStructField => {
                write!(f, "Invalid struct field")
            }
            ErrorKind::MissingRequiredField => {
                write!(f, "Missing required field")
            }
            ErrorKind::ReturnOutsideFunction => {
                write!(f, "Return-outside function")
            }
            ErrorKind::BreakOutsideLoop => {
                write!(f, "Break-outside loop")
            }
            ErrorKind::ContinueOutsideLoop => {
                write!(f, "Continue-outside loop")
            }
            ErrorKind::InvalidBinaryOperation => {
                write!(f, "Invalid binary operation")
            }
            ErrorKind::InvalidUnaryOperation => {
                write!(f, "Invalid unary operation")
            }
            ErrorKind::InvalidFunctionCall => {
                write!(f, "Invalid function call")
            }
            ErrorKind::InvalidPathExpression => {
                write!(f, "Invalid path expression")
            }
            ErrorKind::InvalidMemberAccess => {
                write!(f, "Invalid member access")
            }
            ErrorKind::InvalidItemDeclaration => {
                write!(f, "Invalid item declaration")
            }
            ErrorKind::ExpectedExpression => {
                write!(f, "Expected expression")
            }
            ErrorKind::ExpectedStatement => {
                write!(f, "Expected statement")
            }
            ErrorKind::SyntaxError(msg) => {
                write!(f, "Syntax error: {}", msg)
            }
            ErrorKind::Other(msg) => {
                write!(f, "Other error: {}", msg)
            }
        }
    }
}

pub struct Validator {
    errors: Vec<Error>,
    loop_depth: usize,
    function_depth: usize,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn validate(&mut self, exprs: Vec<Expr>) {
        for expr in exprs {
            self.validate_expression(expr);
        }
    }

    pub fn validate_expression(&mut self, expr: Expr) {
        let span = expr.span.clone();

        match expr.kind {
            ExprKind::Literal(_token) => {
                // Literals are always valid
            }

            ExprKind::Identifier(_name) => {
                // Simple validation for now
                // In a real implementation, check if identifier is defined in the current scope
            }

            ExprKind::Binary(left, _op, right) => {
                self.validate_expression(*left);
                self.validate_expression(*right);
                // Could add additional validation for operator compatibility here
            }

            ExprKind::Unary(_op, operand) => {
                self.validate_expression(*operand);
                // Could add validation for valid unary operations
            }

            ExprKind::Array(elements) => {
                for element in elements {
                    self.validate_expression(element);
                }
            }

            ExprKind::Tuple(elements) => {
                for element in elements {
                    self.validate_expression(element);
                }
            }

            ExprKind::Struct(name_expr, fields_expr) => {
                // Now takes two boxed expressions instead of name and fields
                self.validate_expression(*name_expr);
                self.validate_expression(*fields_expr);
                // Here you might validate that the struct definition exists
                // and that the fields match what's expected
            }

            ExprKind::Bind(pat, expr) => {
                // Validate binding pattern
                self.validate_expression(*pat);
                self.validate_expression(*expr);
            }

            ExprKind::Typed(expr, type_expr) => {
                self.validate_expression(*expr);
                self.validate_expression(*type_expr);
            }

            ExprKind::Index(target, index) => {
                self.validate_expression(*target);
                self.validate_expression(*index);
                // Validate that target is indexable
            }

            ExprKind::Invoke(callee, args) => {
                self.validate_expression(*callee);
                for arg in args {
                    self.validate_expression(arg);
                }
                // Validate callee is callable
            }

            ExprKind::Path(base, member) => {
                self.validate_expression(*base);
                self.validate_expression(*member);
                // Validate path is accessible
            }

            ExprKind::Member(object, member) => {
                self.validate_expression(*object);
                self.validate_expression(*member);
                // Validate member exists on object
            }

            ExprKind::Closure(params, body) => {
                self.function_depth += 1;

                // Validate params - now it's Vec<Expr> instead of a tuple
                for param in params {
                    self.validate_expression(param);
                }

                self.validate_expression(*body);
                self.function_depth -= 1;
            }

            ExprKind::Match(expr, arms) => {
                self.validate_expression(*expr);
                self.validate_expression(*arms);
                // Now arms is a single boxed Expr instead of Vec of pattern/expr pairs
                // Could validate exhaustiveness of match patterns
            }

            ExprKind::Conditional(condition, then_branch, else_branch) => {
                self.validate_expression(*condition);
                self.validate_expression(*then_branch);
                if let Some(else_expr) = else_branch {
                    self.validate_expression(*else_expr);
                }
            }

            ExprKind::While(condition, body) => {
                self.validate_expression(*condition);

                self.loop_depth += 1;
                self.validate_expression(*body);
                self.loop_depth -= 1;
            }

            ExprKind::For(iterator, body) => {
                self.validate_expression(*iterator);

                self.loop_depth += 1;
                self.validate_expression(*body);
                self.loop_depth -= 1;
            }

            ExprKind::Block(statements) => {
                for stmt in statements {
                    self.validate_expression(stmt);
                }
            }

            ExprKind::Item(item_kind) => {
                // Validate item declaration based on the ItemKind
                self.validate_item(item_kind, span);
            }

            ExprKind::Assignment(target, value) => {
                self.validate_assignment_target(&*target);
                self.validate_expression(*target);
                self.validate_expression(*value);
            }

            ExprKind::Definition(name, value) => {
                self.validate_expression(*name);

                // Value is now Option<Box<Expr>> instead of Box<Expr>
                if let Some(val) = value {
                    self.validate_expression(*val);
                }
                // In a real implementation, register the name in the current scope
            }

            ExprKind::Return(value) => {
                if let Some(expr) = value {
                    self.validate_expression(*expr);
                }

                if self.function_depth == 0 {
                    self.errors.push(Error {
                        kind: ErrorKind::ReturnOutsideFunction,
                        span,
                    });
                }
            }

            ExprKind::Break(value) => {
                if let Some(expr) = value {
                    self.validate_expression(*expr);
                }

                if self.loop_depth == 0 {
                    self.errors.push(Error {
                        kind: ErrorKind::BreakOutsideLoop,
                        span,
                    });
                }
            }

            ExprKind::Continue(label) => {
                if let Some(expr) = label {
                    self.validate_expression(*expr);
                }

                if self.loop_depth == 0 {
                    self.errors.push(Error {
                        kind: ErrorKind::ContinueOutsideLoop,
                        span,
                    });
                }
                // Validate that label exists if provided
            }
        }
    }

    fn validate_assignment_target(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Identifier(_) => {
                // Valid assignment target
            },
            ExprKind::Index(_, _) => {
                // Valid assignment target
            },
            ExprKind::Member(_, _) => {
                // Valid assignment target
            },
            ExprKind::Typed(_, _) => {

            }
            ExprKind::Tuple(_) => {

            }
            _ => {
                self.errors.push(Error {
                    kind: ErrorKind::InvalidAssignmentTarget,
                    span: expr.span.clone(),
                });
            }
        }
    }

    fn validate_item(&mut self, _item_kind: ItemKind, _span: Span) {
        // This would need to be implemented based on your ItemKind enum
        // For now, it's a placeholder function
    }
}