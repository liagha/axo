use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::parser::parser::{Expr, Stmt};
use crate::tokens::{Punctuation, Token};

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
            Expr::Assign(left, right) => {
                write!(f, "Assign({:?} to {:?})", right, left)
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
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{:?}", expr),
            Stmt::Assignment(name, expr) => write!(f, "Assignment({:?}, {:?})", name, expr),
            Stmt::CompoundAssignment(name, op, expr) => write!(f, "Compound({:?}, {:?}, {:?})", name, op, expr),
            Stmt::If(cond, then, else_) => { write!(f, "If( Condition: {:?} | Then: {:?} | Else: {:?} )", cond, then, else_) }
            Stmt::While(cond, then) => write!(f, "While( Condition: {:?} | Then: {:?} )", cond, then),
            Stmt::Block(stmts) => { write!(f, "Block({:?})", stmts) }
            Stmt::Return(expr) => write!(f, "Return({:?})", expr),
            Stmt::Definition(name, expr) => write!(f, "Definition({:?}, {:?})", name, expr),
            Stmt::Continue => write!(f, "Continue"),
            Stmt::Break(expr) => write!(f, "Break({:?})", expr),
            Stmt::For(init, cond, increment, body) => {
                write!(f, "For( Init: {:?} | Condition: {:?} | Increment: {:?} | Body: {:?} )", init, cond, increment, body)
            }
            Stmt::Function(name, params, body) => {
                write!(f, "Function( Name: {:?} | Params: {:?} | Body: {:?} )", name, params, body)
            }
            Stmt::StructDef(name, fields) => {
                write!(f, "Struct( Name: {:?} | Fields: {:?} )", name, fields)
            }
            Stmt::EnumDef(name, variants) => {
                let format = variants.iter().map(
                    |(variant, data)| format!("Variant({:?}({:?}))", variant, data)).collect::<Vec<_>>().join(", ");

                write!(f, "Enum( Name: {:?} | Variants: {} )", name, format)
            }
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Boolean(b) => write!(f, "{}", b),
            Token::Float(n) => write!(f, "{}", n),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Punctuation(c) => {
                if c == &Punctuation::Newline {
                    return write!(f, "\n")
                }

                write!(f, "{}", c)
            },
            Token::Operator(c) => write!(f, "{}", c),
            Token::Str(str) => write!(f, "{}", str),
            Token::Identifier(str) => write!(f, "{}", str),
            Token::Char(char) => write!(f, "'{}'", char),
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Invalid(invalid) => write!(f, "{}", invalid.colorize(Color::Red)),
            Token::EOF => write!(f, "{}", "End Of File"),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Boolean(b) => write!(f, "Boolean({})", b),
            Token::Float(n) => write!(f, "Float({})", n),
            Token::Integer(n) => write!(f, "Integer({})", n),
            Token::Operator(op) => write!(f, "Operator({:?})", op),
            Token::Punctuation(pun) => write!(f, "Punctuation({:?})", pun),
            Token::Invalid(err) => write!(f, "Invalid({})", err),
            Token::Identifier(var) => write!(f, "Identifier({})", var),
            Token::Str(str) => write!(f, "String({})", str),
            Token::Char(char) => write!(f, "Char('{}')", char),
            Token::EOF => write!(f, "{}", "End Of File"),
            Token::Keyword(keyword) => write!(f, "Keyword({:?})", keyword),
        }
    }
}