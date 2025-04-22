use {
    crate::{
        axo_parser::{
            item::Item,
            Expr, ExprKind, ItemKind
        }
    },
    core::fmt::{
        Debug, Display,
        Formatter, Result
    }
};

impl Display for ItemKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ItemKind::Expression(expr) => write!(f, "{}", expr),
            ItemKind::Use(expr) => write!(f, "use {}", expr),
            ItemKind::Implement { expr, body} => write!(f, "impl ({}) {}", expr, body),
            ItemKind::Trait{ name, body } => write!(f, "trait ({}) {}", name, body),
            ItemKind::Variable { target, value, .. } => {
                write!(f, "let {}", target)?;

                if let Some(value) = value {
                    write!(f, " = {}", value)?;
                }

                Ok(())
            },
            ItemKind::Structure { name, fields} => {
                let fields = fields.iter().map(|field| field.to_string()).collect::<Vec<_>>().join(", ");

                write!(f, "struct ({}) {}", name, fields)
            },
            ItemKind::Enum { name, body} => write!(f, "enum ({}) {}", name, body),
            ItemKind::Macro { name, parameters, body} => {
                let params = parameters.iter().map(|param| param.to_string()).collect::<Vec<_>>().join(", ");

                write!(f, "macro {}({}) {}", name, params, body)
            },
            ItemKind::Function { name, parameters, body} => {
                let params = parameters.iter().map(|param| param.to_string()).collect::<Vec<_>>().join(", ");

                write!(f, "fn {}({}) {}", name, params, body)
            },
            ItemKind::Field { name, value, ty } => {
                write!(f, "{}", name)?;

                if let Some(ty) = ty {
                    write!(f, " : {}", ty)?;
                }

                if let Some(value) = value {
                    write!(f, " = {}", value)
                } else {
                    write!(f, "")
                }
            },
            ItemKind::Unit => write!(f, "()")
        }
    }
}

impl Debug for ItemKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ItemKind::Expression(expr) => write!(f, "~{:?}~", expr),
            ItemKind::Use(expr) => write!(f, "Use({:?})", expr),
            ItemKind::Implement { expr, body } => write!(f, "Implement({:?} => {:?})", expr, body),
            ItemKind::Trait { name, body} => write!(f, "Trait({:?} {:?})", name, body),
            ItemKind::Variable { target, value, mutable, ty } => {
                let kind = if *mutable { "Variable" } else { "Constant" };
                write!(f, "{}({:?}", kind, target)?;

                if let Some(ty) = ty {
                    write!(f, " : {:?}", ty)?;
                }

                if let Some(value) = value {
                    write!(f, " = {:?}", value)?;
                }

                write!(f, ")")
            },
            ItemKind::Structure { name, fields } => write!(f, "Structure({:?} | {:?})", name, fields),
            ItemKind::Enum { name, body } => write!(f, "Enum({:?} | {:?})", name, body),
            ItemKind::Macro { name, parameters, body } => write!(f, "Macro({:?}({:?}) {:?})", name, parameters, body),
            ItemKind::Function { name, parameters, body } => write!(f, "Function({:?}({:?}) {:?})", name, parameters, body),
            ItemKind::Field { name, value, ty } => {
                write!(f, "Field({:?}", name)?;

                if let Some(ty) = ty {
                    write!(f, " : {:?}", ty)?;
                }

                if let Some(value) = value {
                    write!(f, " = {:?}", value)?;
                }

                write!(f, ")")
            },
            ItemKind::Unit => write!(f, "()")
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            write!(f, "{:?} | [{}]", self.kind, self.span)
        } else {
            write!(f, "{:?}", self.kind)
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.kind)
    }
}

impl Debug for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.kind)
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ExprKind::Literal(token) => write!(f, "{}", token),
            ExprKind::Identifier(ident) => write!(f, "{}", ident),

            ExprKind::Collection(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "[{}]", elems.join(", "))
            }
            ExprKind::Group(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "({})", elems.join(", "))
            }
            ExprKind::Bundle(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "{{{}}}", elems.join(", "))
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
            ExprKind::Constructor { name, body } => {
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
            ExprKind::Path { tree } => write!(f, "{}", tree),

            ExprKind::Error(e) => write!(f, "error: {}", e)
        }
    }
}

impl Debug for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ExprKind::Literal(literal) => {
                write!(f, "{:?}", literal)
            },
            ExprKind::Identifier(identifier) => {
                write!(f, "Identifier({})", identifier)
            },
            ExprKind::Collection(elements) => {
                write!(f, "Collection({:?})", elements)
            },
            ExprKind::Group(elements) => {
                write!(f, "Group({:?})", elements)
            },
            ExprKind::Bundle(elements) => {
                write!(f, "Bundle({:?})", elements)
            }

            ExprKind::Binary { left, operator, right } => {
                write!(f, "Binary({:?} {:?} {:?})", left, operator, right)
            }
            ExprKind::Unary { operator, operand: expr } => {
                write!(f, "Unary({:?} {:?})", operator, expr)
            },

            ExprKind::Labeled { label: expr, expr: ty } => {
                write!(f, "Labeled({:?}: {:?})", expr, ty)
            },
            ExprKind::Index { expr, index } => {
                write!(f, "Index({:?}[{:?}])", expr, index)
            },
            ExprKind::Invoke { target, parameters } => {
                write!(f, "Invoke({:?}({:?}))", target, parameters)
            },
            ExprKind::Member { object, member} => {
                write!(f, "Member({:?}.{:?})", object, member)
            },
            ExprKind::Closure { parameters, body } => {
                write!(f, "Closure(|{:?}| {:?})", parameters, body)
            },

            ExprKind::Match { target, body } => {
                write!(f, "Match({:?} => {:?})", target, body)
            },
            ExprKind::Conditional { condition, then_branch, else_branch } => {
                write!(f, "Conditional({:?} | Then: {:?}", condition, then_branch)?;

                if let Some(else_expr) = else_branch {
                    write!(f, " | Else: {:?}", else_expr)?;
                }

                write!(f, ")")
            }
            ExprKind::Loop { body } => {
                write!(f, "Loop({:?})", body)
            },
            ExprKind::While { condition, body: then } => {
                write!(f, "While({:?} do {:?})", condition, then)
            },
            ExprKind::For { clause, body } => {
                write!(f, "For({:?} in {:?})", clause, body)
            },
            ExprKind::Block(stmts) => {
                write!(f, "Block({:#?})", stmts)
            },

            ExprKind::Assignment { target, value } => {
                write!(f, "Assignment({:?} = {:?})", target, value)
            },
            ExprKind::Constructor { name, body } => {
                write!(f, "Constructor({:?} | {:?})", name, body)
            },

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
            ExprKind::Path { tree } => write!(f, "Path({:?})", tree),

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
