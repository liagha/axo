use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::axo_lexer::{Span, Token};
use crate::axo_lexer::{PunctuationKind, TokenKind};
use crate::axo_parser::{Expr, ExprKind};

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} | {}", self.kind, self.span)
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // Primary Expressions
            ExprKind::Literal(literal) => write!(f, "{:?}", literal),
            ExprKind::Identifier(identifier) => write!(f, "Identifier({})", identifier),
            ExprKind::Binary(first, operator, second) => {
                write!(f, "Binary({:?} {:?} {:?})", first, operator, second)
            }
            ExprKind::Unary(operator, expr) => write!(f, "Unary({:?} {:?})", operator, expr),
            ExprKind::Array(array) => write!(f, "Array({:?})", array),
            ExprKind::Tuple(tuple) => write!(f, "Tuple({:?})", tuple),

            // Composite Expressions
            ExprKind::Typed(expr, ty) => write!(f, "Typed({:?} : {:?})", expr, ty),
            ExprKind::Index(expr, index) => write!(f, "Index({:?}, {:?})", index, expr),
            ExprKind::Invoke(function, params) => write!(f, "Invoke({:?}, {:?})", function, params),
            ExprKind::Member(expr, field) => write!(f, "FieldAccess({:?}, {:?})", expr, field),
            ExprKind::Closure(params, lambda) => write!(f, "Closure(|{:?}| {:?})", params, lambda),

            // Control Flow
            ExprKind::Conditional(cond, then, else_) => {
                write!(f, "If( Condition: {:?} | Then: {:?} | Else: {:?} )", cond, then, else_)
            }
            ExprKind::While(cond, then) => write!(f, "While( Condition: {:?} | Then: {:?} )", cond, then),
            ExprKind::For(clause, body) => write!(f, "For( Clause: {:?} | Body: {:?} )", clause, body),
            ExprKind::Block(stmts) => write!(f, "Block({:#?})", stmts),

            // Declarations & Definitions
            ExprKind::Assignment(name, expr) => write!(f, "Assignment({:?}, {:?})", name, expr),
            ExprKind::Definition(name, expr) => write!(f, "Definition({:?}, {:?})", name, expr),
            ExprKind::Struct(name, fields) => write!(f, "Struct( Name: {:?}, Fields: {:?} )", name, fields),
            ExprKind::StructDef(name, fields) => write!(f, "StructDef( Name: {:?} | Fields: {:?} )", name, fields),
            ExprKind::Enum(name, variants) => write!(f, "Enum( Name: {:?} | Variants: {:?} )", name, variants),
            ExprKind::Function(name, params, body) => {
                write!(f, "Function( Name: {:?} | Params: {:?} | Body: {:?} )", name, params, body)
            }

            // Flow Control Statements
            ExprKind::Return(expr) => write!(f, "Return({:?})", expr),
            ExprKind::Break(expr) => write!(f, "Break({:?})", expr),
            ExprKind::Continue(expr) => write!(f, "Continue({:?})", expr),

            // Patterns
            ExprKind::WildCard => write!(f, "WildCard"),
            ExprKind::Bind(key, value) => write!(f, "Bind({:?}, {:?})", key, value),
            ExprKind::Path(expr, sub) => write!(f,"Path({:?}, {:?})", expr, sub),
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
        if self.kind == TokenKind::EOF {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{:?}", self.kind)

            //write!(f, "{:?} | {}", self.kind, self.span )
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Span {
            start: (start_line, start_column),
            end: (end_line, end_column), ..
        } = self;

        if start_line == end_line && start_column == end_column {
            write!(f, "{}:{}", start_line, start_column )
        } else {
            write!(f, "({}:{} : {}:{})", start_line, start_column, end_line, end_column )
        }
    }
}