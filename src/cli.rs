use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::lexer::{Span, Token};
use crate::lexer::{PunctuationKind, TokenKind};
use crate::parser::{Expr};

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(num) => {
                write!(f, "Number({:?})", num)
            }
            Expr::Boolean(bool) => {
                write!(f, "Boolean({})", bool)
            }
            Expr::Char(char) => {
                write!(f, "Char({:?})", char)
            }
            Expr::String(string) => {
                write!(f, "String({:?})", string)
            }
            Expr::Identifier(identifier) => {
                write!(f, "Identifier({})", identifier)
            }
            Expr::Typed(expr, ty) => {
                write!(f, "Typed({:?} : {:?})", expr, ty)
            }
            Expr::Binary(first, operator, second) => {
                write!(f, "Binary({:?} {:?} {:?})", first, operator, second)
            }
            Expr::Unary(operator, expr) => {
                write!(f, "Unary({:?} {:?})", operator, expr)
            }
            Expr::Array(array) => {
                write!(f, "Array({:?})", array)
            }
            Expr::Index(expr, index) => {
                write!(f, "Index({:?}, {:?})", index, expr)
            }
            Expr::Call(function, params) => {
                write!(f, "Call({:?}, {:?})", function, params)
            }
            Expr::Lambda(params, lambda) => {
                write!(f, "Lambda(|{:?}| {:?})", params, lambda)
            }
            Expr::StructInit(name, fields) => {
                write!(f, "Struct( Name: {:?}, Fields: {:?} )", name, fields)
            }
            Expr::FieldAccess(expr, field) => {
                write!(f, "FieldAccess({:?}, {:?})", expr, field)
            }
            Expr::Tuple(tuple) => {
                write!(f, "Tuple({:?})", tuple)
            }
            Expr::Assignment(name, expr) => write!(f, "Assignment({:?}, {:?})", name, expr),
            Expr::CompoundAssignment(name, op, expr) => write!(f, "Compound | Assignment({:?}, {:?}, {:?})", name, op, expr),
            Expr::If(cond, then, else_) => { write!(f, "If( Condition: {:?} | Then: {:?} | Else: {:?} )", cond, then, else_) }
            Expr::While(cond, then) => write!(f, "While( Condition: {:?} | Then: {:?} )", cond, then),
            Expr::Block(stmts) => { write!(f, "Block({:#?})", stmts) }
            Expr::Return(expr) => write!(f, "Return({:?})", expr),
            Expr::Definition(name, expr) => write!(f, "Definition({:?}, {:?})", name, expr),
            Expr::Continue => write!(f, "Continue"),
            Expr::Break(expr) => write!(f, "Break({:?})", expr),
            Expr::For(clause, body) => {
                write!(f, "For( Clause: {:?} | Body: {:?} )", clause, body)
            }
            Expr::Function(name, params, body) => {
                write!(f, "Function( Name: {:?} | Params: {:?} | Body: {:?} )", name, params, body)
            }
            Expr::StructDef(name, fields) => {
                write!(f, "Struct( Name: {:?} | Fields: {:?} )", name, fields)
            }
            Expr::EnumDef(name, variants) => {
                let format = variants.iter().map(
                    |(variant, data)| format!("Variant({:?}({:?}))", variant, data)).collect::<Vec<_>>().join(", ");

                write!(f, "Enum( Name: {:?} | Variants: {} )", name, format)
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
