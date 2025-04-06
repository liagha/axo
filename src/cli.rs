use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::axo_lexer::{Span, Token};
use crate::axo_lexer::{PunctuationKind, TokenKind};
use crate::axo_parser::{Expr, ExprKind};

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} | [{}]", self.kind, self.span)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Literal(token) => write!(f, "{}", token),
            ExprKind::Identifier(ident) => write!(f, "{}", ident),
            ExprKind::Binary(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            ExprKind::Unary(op, expr) => write!(f, "({}{})", op, expr),
            ExprKind::Array(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", elems.join(", "))
            }
            ExprKind::Tuple(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", elems.join(", "))
            }

            ExprKind::Typed(expr, ty) => write!(f, "{}: {}", expr, ty),
            ExprKind::Index(expr, index) => write!(f, "{}[{}]", expr, index),
            ExprKind::Invoke(function, args) => {
                let args_str: Vec<String> = args.iter().map(|e| e.to_string()).collect();
                write!(f, "{}({})", function, args_str.join(", "))
            }
            ExprKind::Member(expr, field) => write!(f, "{}.{}", expr, field),
            ExprKind::Closure(params, body) => {
                let params_str: Vec<String> = params.iter().map(|e| e.to_string()).collect();
                write!(f, "|{}| {}", params_str.join(", "), body)
            }

            ExprKind::Match(clause, body) => write!(f, "match {} \n{}\n", clause, body),
            ExprKind::Conditional(cond, then, else_opt) => {
                write!(f, "if {} {}\n", cond, then)?;
                if let Some(else_expr) = else_opt {
                    write!(f, " else {}\n", else_expr)?;
                }
                Ok(())
            }
            ExprKind::While(cond, body) => write!(f, "while {} \n{}\n", cond, body),
            ExprKind::For(clause, body) => write!(f, "for {} \n{}\n", clause, body),
            ExprKind::Block(stmts) => {
                if stmts.is_empty() {
                    write!(f, "{{}}")
                } else {
                    let stmts_str: Vec<String> = stmts.iter().map(|e| indent(e)).collect();
                    write!(f, "{{\n{}\n}}", stmts_str.join("\n"))
                }
            }

            ExprKind::Assignment(lhs, rhs) => write!(f, "{} = {}", lhs, rhs),
            ExprKind::Definition(name, value_opt) => {
                write!(f, "let {}", name)?;
                if let Some(value) = value_opt {
                    write!(f, " = {}", value)?;
                }
                Ok(())
            }
            ExprKind::Implement(name, body) => {
                write!(f, "impl {} \n{}\n", name, body)
            }
            ExprKind::Trait(name, body) => {
                write!(f, "trait {} \n{}\n", name, body)
            }
            ExprKind::Struct(name, fields) => {
                let fields_str: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
                write!(f, "{} {{ {} }}", name, fields_str.join(", "))
            }
            ExprKind::StructDef(name, fields) => {
                let fields_str: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
                write!(f, "struct {} {{\n{}\n}}", name, fields_str.join("\n"))
            }
            ExprKind::Enum(name, variants) => {
                let variants_str: Vec<String> = variants.iter().map(|v| v.to_string()).collect();
                write!(f, "enum {} {{\n{}\n}}", name, variants_str.join("\n"))
            }
            ExprKind::Function(name, params, body) => {
                let params_str: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "fn {}({}) {}\n", name, params_str.join(", "), body)
            }
            ExprKind::Macro(name, params, body) => {
                let params_str: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "macro {}({}) {}\n", name, params_str.join(", "), body)
            }

            ExprKind::Return(expr_opt) => {
                write!(f, "return")?;
                if let Some(expr) = expr_opt {
                    write!(f, " {}", expr)?;
                }
                Ok(())
            }
            ExprKind::Break(expr_opt) => {
                write!(f, "break")?;
                if let Some(expr) = expr_opt {
                    write!(f, " {}", expr)?;
                }
                Ok(())
            }
            ExprKind::Continue(expr_opt) => {
                write!(f, "continue")?;
                if let Some(expr) = expr_opt {
                    write!(f, " {}", expr)?;
                }
                Ok(())
            }

            ExprKind::WildCard => write!(f, "_"),
            ExprKind::Bind(key, value) => write!(f, "{} => {}", key, value),
            ExprKind::Path(lhs, rhs) => write!(f, "{}::{}", lhs, rhs),
        }
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Literal(literal) => write!(f, "{:?}", literal),
            ExprKind::Identifier(identifier) => write!(f, "Identifier({})", identifier),
            ExprKind::Binary(lhs, op, rhs) => {
                write!(f, "Binary({:?} {:?} {:?})", lhs, op, rhs)
            }
            ExprKind::Unary(op, expr) => write!(f, "Unary({:?} {:?})", op, expr),
            ExprKind::Array(elems) => write!(f, "Array({:?})", elems),
            ExprKind::Tuple(elems) => write!(f, "Tuple({:?})", elems),

            ExprKind::Typed(expr, ty) => write!(f, "Typed({:?}: {:?})", expr, ty),
            ExprKind::Index(expr, index) => write!(f, "Index({:?}[{:?}])", expr, index),
            ExprKind::Invoke(func, args) => write!(f, "Invoke({:?}({:?}))", func, args),
            ExprKind::Member(expr, field) => write!(f, "Member({:?}.{:?})", expr, field),
            ExprKind::Closure(params, body) => write!(f, "Closure(|{:?}| {:?})", params, body),

            ExprKind::Match(clause, body) => write!(f, "Match({:?} => {:?})", clause, body),
            ExprKind::Conditional(cond, then, else_opt) => {
                write!(f, "Conditional({:?} ? {:?}", cond, then)?;
                if let Some(else_expr) = else_opt {
                    write!(f, " : {:?}", else_expr)?;
                }
                write!(f, ")")
            }
            ExprKind::While(cond, body) => write!(f, "While({:?} do {:?})", cond, body),
            ExprKind::For(clause, body) => write!(f, "For({:?} in {:?})", clause, body),
            ExprKind::Block(stmts) => write!(f, "Block({:#?})", stmts),

            ExprKind::Assignment(lhs, rhs) => write!(f, "Assignment({:?} = {:?})", lhs, rhs),
            ExprKind::Definition(name, value) => write!(f, "Definition({:?} = {:?})", name, value),
            ExprKind::Implement(name, body) => write!(f, "Implement({:?} : {:?})", name, body),
            ExprKind::Trait(name, body) => write!(f, "Trait({:?} with {:?})", name, body),
            ExprKind::Struct(name, fields) => write!(f, "Struct({:?} with {:?})", name, fields),
            ExprKind::StructDef(name, fields) => write!(f, "StructDef({:?} with {:?})", name, fields),
            ExprKind::Enum(name, variants) => write!(f, "Enum({:?} with {:?})", name, variants),
            ExprKind::Function(name, params, body) => {
                write!(f, "Function({:?}({:?}) => {:?})", name, params, body)
            }
            ExprKind::Macro(name, params, body) => {
                write!(f, "Macro({:?}({:?}) => {:?})", name, params, body)
            }

            ExprKind::Return(expr) => {
                write!(f, "Return")?;
                if expr.is_some() {
                    write!(f, "({:?})", expr)?;
                }
                Ok(())
            }
            ExprKind::Break(expr) => {
                write!(f, "Break")?;
                if expr.is_some() {
                    write!(f, "({:?})", expr)?;
                }
                Ok(())
            }
            ExprKind::Continue(expr) => {
                write!(f, "Continue")?;
                if expr.is_some() {
                    write!(f, "({:?})", expr)?;
                }
                Ok(())
            }

            ExprKind::WildCard => write!(f, "WildCard"),
            ExprKind::Bind(key, value) => write!(f, "Bind({:?} => {:?})", key, value),
            ExprKind::Path(lhs, rhs) => write!(f, "Path({:?}::{:?})", lhs, rhs),
        }
    }
}

fn indent(expr: &Expr) -> String {
    let s = expr.to_string();
    s.lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n")
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
            TokenKind::EOF => write!(f, "EOF"),
            TokenKind::Keyword(keyword) => write!(f, "Keyword({:?})", keyword),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.kind != TokenKind::EOF {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "")
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.kind == TokenKind::EOF {
            write!(f, "EOF")
        } else {
            write!(f, "{:?}", self.kind)
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
            write!(f, "{}:{}", start_line, start_column)
        } else {
            write!(f, "{}:{}-{}:{}", start_line, start_column, end_line, end_column)
        }
    }
}