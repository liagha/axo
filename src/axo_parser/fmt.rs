use crate::axo_parser::{Expr, ExprKind, ItemKind};

impl core::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ItemKind::Use(expr) => write!(f, "use {}", expr),
            ItemKind::Implement(expr, body) => write!(f, "impl ({}) {}", expr, body),
            ItemKind::Trait(name, body) => write!(f, "trait ({}) {}", name, body),
            ItemKind::Struct(name, body) => write!(f, "struct ({}) {}", name, body),
            ItemKind::Enum(name, body) => write!(f, "enum ({}) {}", name, body),
            ItemKind::Macro(name, params, body) => {
                let params = params.iter().map(|param| param.to_string()).collect::<Vec<_>>().join(", ");

                write!(f, "macro {}({}) {}", name, params, body)
            },
            ItemKind::Function(name, params, body) => {
                let params = params.iter().map(|param| param.to_string()).collect::<Vec<_>>().join(", ");

                write!(f, "fn {}({}) {}", name, params, body)
            },
        }
    }
}

impl core::fmt::Debug for ItemKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ItemKind::Use(expr) => write!(f, "Use({:?})", expr),
            ItemKind::Implement(expr, body) => write!(f, "Implement({:?} => {:?})", expr, body),
            ItemKind::Trait(name, body) => write!(f, "Trait({:?} {:?})", name, body),
            ItemKind::Struct(name, body) => write!(f, "Struct({:?} | {:?})", name, body),
            ItemKind::Enum(name, body) => write!(f, "Enum({:?} | {:?})", name, body),
            ItemKind::Macro(name, params, body) => write!(f, "Macro({:?}({:?}) {:?})", name, params, body),
            ItemKind::Function(name, params, body) => write!(f, "Function({:?}({:?}) {:?})", name, params, body),
        }
    }
}

impl core::fmt::Debug for Expr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} | [{}]", self.kind, self.span)
    }
}

impl core::fmt::Display for Expr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl core::fmt::Display for ExprKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

            ExprKind::Match(clause, body) => write!(f, "match {} {}\n", clause, body),
            ExprKind::Conditional(cond, then, else_opt) => {
                write!(f, "if {} {}\n", cond, then)?;
                if let Some(else_expr) = else_opt {
                    write!(f, " else {}\n", else_expr)?;
                }
                Ok(())
            }
            ExprKind::While(cond, body) => write!(f, "while {} {}\n", cond, body),
            ExprKind::For(clause, body) => write!(f, "for {} {}\n", clause, body),
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
            ExprKind::Struct(name, fields) => {
                write!(f, "{} {}", name, fields)
            }

            ExprKind::Item(item) => write!(f, "{}", item),

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

            ExprKind::Bind(key, value) => write!(f, "{} => {}", key, value),
            ExprKind::Path(lhs, rhs) => write!(f, "{}::{}", lhs, rhs),
        }
    }
}

impl core::fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
            ExprKind::Struct(name, fields) => write!(f, "Struct({:?} with {:?})", name, fields),

            ExprKind::Item(item) => write!(f, "+ {:?}", item),
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
