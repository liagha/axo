use crate::axo_parser::{Expr, ExprKind, ItemKind};

impl core::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ItemKind::Use(expr) => write!(f, "use {}", expr),
            ItemKind::Implement(expr, body) => write!(f, "impl ({}) {}", expr, body),
            ItemKind::Trait(name, body) => write!(f, "trait ({}) {}", name, body),
            ItemKind::Struct { name, body} => write!(f, "struct ({}) {}", name, body),
            ItemKind::Enum { name, body} => write!(f, "enum ({}) {}", name, body),
            ItemKind::Macro { name, parameters, body} => {
                let params = parameters.iter().map(|param| param.to_string()).collect::<Vec<_>>().join(", ");

                write!(f, "macro {}({}) {}", name, params, body)
            },
            ItemKind::Function { name, parameters, body} => {
                let params = parameters.iter().map(|param| param.to_string()).collect::<Vec<_>>().join(", ");

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
            ItemKind::Struct { name, body } => write!(f, "Struct({:?} | {:?})", name, body),
            ItemKind::Enum { name, body } => write!(f, "Enum({:?} | {:?})", name, body),
            ItemKind::Macro { name, parameters, body } => write!(f, "Macro({:?}({:?}) {:?})", name, parameters, body),
            ItemKind::Function { name, parameters, body } => write!(f, "Function({:?}({:?}) {:?})", name, parameters, body),
        }
    }
}

impl core::fmt::Debug for Expr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:?} | [{}]", self.kind, self.span)
        } else {
            write!(f, "{:?}", self.kind)
        }
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

            ExprKind::Array(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", elems.join(", "))
            }
            ExprKind::Tuple(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", elems.join(", "))
            }

            ExprKind::Binary { left, operator, right } => write!(f, "({} {} {})", left, operator, right),
            ExprKind::Unary { operator, operand: expr } => write!(f, "({}{})", operator, expr),

            ExprKind::Labeled { label: expr, expr: ty } => write!(f, "{}: {}", expr, ty),
            ExprKind::Index { expr, index } => write!(f, "{}[{}]", expr, index),
            ExprKind::Invoke { target, parameters } => {
                let args_str: Vec<String> = parameters.iter().map(|e| e.to_string()).collect();
                write!(f, "{}({})", target, args_str.join(", "))
            }
            ExprKind::Member { object, member } => write!(f, "{}.{}", object, member),

            ExprKind::Closure { parameters, body } => {
                let params_str: Vec<String> = parameters.iter().map(|e| e.to_string()).collect();
                write!(f, "|{}| {}", params_str.join(", "), body)
            }

            ExprKind::Block(stmts) => {
                if stmts.is_empty() {
                    write!(f, "{{}}")
                } else {
                    let stmts_str: Vec<String> = stmts.iter().map(|e| indent(e)).collect();
                    write!(f, "{{\n{}\n}}", stmts_str.join("\n"))
                }
            }
            ExprKind::Match { target, body } => write!(f, "match {} {}\n", target, body),
            ExprKind::Conditional { condition, then_branch, else_branch} => {
                write!(f, "if {} {}\n", condition, then_branch)?;

                if let Some(else_expr) = else_branch {
                    write!(f, " else {}\n", else_expr)?;
                }

                Ok(())
            }
            ExprKind::Loop { body } => write!(f, "loop {}", body),
            ExprKind::While { condition, body: then } => write!(f, "while {} {}\n", condition, then),
            ExprKind::For { clause, body} => write!(f, "for {} {}\n", clause, body),

            ExprKind::Item(item) => write!(f, "{}", item),
            ExprKind::Assignment { target, value} => write!(f, "{} = {}", target, value),
            ExprKind::Definition { target, value } => {
                write!(f, "let {}", target)?;

                if let Some(value) = value {
                    write!(f, " = {}", value)?;
                }

                Ok(())
            }
            ExprKind::Struct { name, body } => {
                write!(f, "{} {}", name, body)
            }

            ExprKind::Return(expr) => {
                write!(f, "return")?;

                if let Some(expr) = expr {
                    write!(f, " {}", expr)?;
                }

                Ok(())
            }
            ExprKind::Break(expr) => {
                write!(f, "break")?;

                if let Some(expr) = expr {
                    write!(f, " {}", expr)?;
                }

                Ok(())
            }
            ExprKind::Continue(expr) => {
                write!(f, "continue")?;

                if let Some(expr) = expr {
                    write!(f, " {}", expr)?;
                }

                Ok(())
            }

            ExprKind::Bind { key, value } => write!(f, "{} => {}", key, value),
            ExprKind::Path { left, right } => write!(f, "{}::{}", left, right),

            ExprKind::Error(e) => write!(f, "error: {}", e)
        }
    }
}

impl core::fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExprKind::Literal(literal) => write!(f, "{:?}", literal),
            ExprKind::Identifier(identifier) => write!(f, "Identifier({})", identifier),
            ExprKind::Array(elems) => write!(f, "Array({:?})", elems),
            ExprKind::Tuple(elems) => write!(f, "Tuple({:?})", elems),

            ExprKind::Binary { left, operator, right } => {
                write!(f, "Binary({:?} {:?} {:?})", left, operator, right)
            }
            ExprKind::Unary { operator, operand: expr } => write!(f, "Unary({:?} {:?})", operator, expr),

            ExprKind::Labeled { label: expr, expr: ty } => write!(f, "Typed({:?}: {:?})", expr, ty),
            ExprKind::Index { expr, index } => write!(f, "Index({:?}[{:?}])", expr, index),
            ExprKind::Invoke { target, parameters } => write!(f, "Invoke({:?}({:?}))", target, parameters),
            ExprKind::Member { object, member} => write!(f, "Member({:?}.{:?})", object, member),
            ExprKind::Closure { parameters, body } => write!(f, "Closure(|{:?}| {:?})", parameters, body),

            ExprKind::Match { target, body } => write!(f, "Match({:?} => {:?})", target, body),
            ExprKind::Conditional { condition, then_branch, else_branch } => {
                write!(f, "Conditional({:?} ? {:?}", condition, then_branch)?;
                if let Some(else_expr) = else_branch {
                    write!(f, " : {:?}", else_expr)?;
                }
                write!(f, ")")
            }
            ExprKind::Loop { body } => write!(f, "Loop({:?})", body),
            ExprKind::While { condition, body: then } => write!(f, "While({:?} do {:?})", condition, then),
            ExprKind::For { clause, body } => write!(f, "For({:?} in {:?})", clause, body),
            ExprKind::Block(stmts) => write!(f, "Block({:#?})", stmts),

            ExprKind::Assignment { target, value } => write!(f, "Assignment({:?} = {:?})", target, value),
            ExprKind::Definition { target, value } => write!(f, "Definition({:?} = {:?})", target, value),
            ExprKind::Struct { name, body } => write!(f, "Struct({:?} with {:?})", name, body),

            ExprKind::Item(item) => write!(f, "+ {:?}", item),
            ExprKind::Return(expr) => {
                write!(f, "Return")?;
                if let Some(expr) = expr {
                    write!(f, "({:?})", expr)?;
                }
                Ok(())
            }
            ExprKind::Break(expr) => {
                write!(f, "Break")?;
                if let Some(expr) = expr {
                    write!(f, "({:?})", expr)?;
                }
                Ok(())
            }
            ExprKind::Continue(expr) => {
                write!(f, "Continue")?;
                if let Some(expr) = expr {
                    write!(f, "({:?})", expr)?;
                }
                Ok(())
            }

            ExprKind::Bind { key, value } => write!(f, "Bind({:?} => {:?})", key, value),
            ExprKind::Path { left, right } => write!(f, "Path({:?}::{:?})", left, right),

            ExprKind::Error(e) => write!(f, "Error({:?})", e)
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
