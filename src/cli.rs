use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::lexer::{Span, Token};
use crate::lexer::{PunctuationKind, TokenKind};
use crate::parser::{Expr, ExprKind};

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Literal(literal) => {
                write!(f, "{:?}", literal)
            }
            ExprKind::Identifier(identifier) => {
                write!(f, "Identifier({})", identifier)
            }
            ExprKind::Typed(expr, ty) => {
                write!(f, "Typed({:?} : {:?})", expr, ty)
            }
            ExprKind::Binary(first, operator, second) => {
                write!(f, "Binary({:?} {:?} {:?})", first, operator, second)
            }
            ExprKind::Unary(operator, expr) => {
                write!(f, "Unary({:?} {:?})", operator, expr)
            }
            ExprKind::Array(array) => {
                write!(f, "Array({:?})", array)
            }
            ExprKind::Index(expr, index) => {
                write!(f, "Index({:?}, {:?})", index, expr)
            }
            ExprKind::Call(function, params) => {
                write!(f, "Call({:?}, {:?})", function, params)
            }
            ExprKind::Closure(params, lambda) => {
                write!(f, "Closure(|{:?}| {:?})", params, lambda)
            }
            ExprKind::StructInit(name, fields) => {
                write!(f, "Struct( Name: {:?}, Fields: {:?} )", name, fields)
            }
            ExprKind::FieldAccess(expr, field) => {
                write!(f, "FieldAccess({:?}, {:?})", expr, field)
            }
            ExprKind::Tuple(tuple) => {
                write!(f, "Tuple({:?})", tuple)
            }
            ExprKind::Assignment(name, expr) => write!(f, "Assignment({:?}, {:?})", name, expr),
            ExprKind::If(cond, then, else_) => { write!(f, "If( Condition: {:?} | Then: {:?} | Else: {:?} )", cond, then, else_) }
            ExprKind::While(cond, then) => write!(f, "While( Condition: {:?} | Then: {:?} )", cond, then),
            ExprKind::Block(stmts) => { write!(f, "Block({:#?})", stmts) }
            ExprKind::Return(expr) => write!(f, "Return({:?})", expr),
            ExprKind::Definition(name, expr) => write!(f, "Definition({:?}, {:?})", name, expr),
            ExprKind::Continue => write!(f, "Continue"),
            ExprKind::Break(expr) => write!(f, "Break({:?})", expr),
            ExprKind::For(clause, body) => {
                write!(f, "For( Clause: {:?} | Body: {:?} )", clause, body)
            }
            ExprKind::Function(name, params, body) => {
                write!(f, "Function( Name: {:?} | Params: {:?} | Body: {:?} )", name, params, body)
            }
            ExprKind::StructDef(name, fields) => {
                write!(f, "StructDef( Name: {:?} | Fields: {:?} )", name, fields)
            }
            ExprKind::Enum(name, variants) => {
                write!(f, "Enum( Name: {:?} | Variants: {:?} )", name, variants)
            }
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "{}", b),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Punctuation(c) => {
                if c == &PunctuationKind::Newline {
                    return write!(f, "\n")
                }

                write!(f, "{}", c)
            },
            TokenKind::Operator(c) => write!(f, "{}", c),
            TokenKind::Str(str) => write!(f, "{}", str),
            TokenKind::Identifier(str) => write!(f, "{}", str),
            TokenKind::Char(char) => write!(f, "'{}'", char),
            TokenKind::Keyword(keyword) => write!(f, "{}", keyword),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
            TokenKind::Invalid(invalid) => write!(f, "{}", invalid.colorize(Color::Red)),
            TokenKind::EOF => write!(f, "{}", "End Of File"),
        }
    }
}

impl fmt::Debug for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "Boolean({})", b),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Integer(n) => write!(f, "Integer({})", n),
            TokenKind::Operator(op) => write!(f, "Operator({:?})", op),
            TokenKind::Punctuation(pun) => write!(f, "Punctuation({:?})", pun),
            TokenKind::Invalid(err) => write!(f, "Invalid({})", err),
            TokenKind::Identifier(var) => write!(f, "Identifier({})", var),
            TokenKind::Str(str) => write!(f, "String({})", str),
            TokenKind::Char(char) => write!(f, "Char('{}')", char),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
            TokenKind::EOF => write!(f, "{}", "End Of File"),
            TokenKind::Keyword(keyword) => write!(f, "Keyword({:?})", keyword),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Span {
            start: (start_line, start_column),
            end: (end_line, end_column)
        } = self.span;

        if self.kind == TokenKind::EOF {
            write!(f, "[{:?}]", self.kind)
        } else if start_line == end_line && start_column == end_column {
            write!(f, "[{:?} | {}:{}]", self.kind, start_line, start_column )
        } else {
            write!(f, "[{:?} | {}:{} : {}:{}]", self.kind, start_line, start_column, end_line, end_column )
        }
    }
}
