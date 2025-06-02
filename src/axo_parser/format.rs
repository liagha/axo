use {
    crate::{
        format::{
            Debug, Display,
            Formatter, Result
        },
        
        axo_parser::{
            item::Item,
            Element, ElementKind, ItemKind
        },
        
        axo_format::indent,
    },
};

impl Display for ItemKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ItemKind::Use(element) => write!(f, "use {}", element),
            ItemKind::Formed { identifier, form } => write!(f, "formed({}, {})", identifier, form),
            ItemKind::Implement { element, body} => write!(f, "impl ({}) {}", element, body),
            ItemKind::Trait{ name, body } => write!(f, "trait ({}) {}", name, body),
            ItemKind::Variable { target, value, mutable, ty } => {
                if *mutable {
                    write!(f, "var {}", target)?;
                } else { 
                    write!(f, "const {}", target)?;
                }

                if let Some(ty) = ty {
                    write!(f, " : {}", ty)?;
                }

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
            ItemKind::Use(element) => write!(f, "Use({:?})", element),
            ItemKind::Formed { identifier, form } => write!(f, "Formed({:?}: {:?})", identifier, form),
            ItemKind::Implement { element, body } => write!(f, "Implement({:?} => {:?})", element, body),
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

impl Debug for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            write!(f, "{:?} | [{}]", self.kind, self.span)
        } else {
            write!(f, "{:?}", self.kind)
        }
    }
}

impl Display for Element {
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

impl Display for ElementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ElementKind::Literal(token) => write!(f, "{}", token),
            ElementKind::Identifier(ident) => write!(f, "{}", ident),
            
            ElementKind::Procedural(element) => {
                write!(f, "procedural {}", element)
            }

            ElementKind::Collection(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "[{}]", elems.join(", "))
            }
            ElementKind::Series(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "[{}]", elems.join("; "))
            }
            ElementKind::Group(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "({})", elems.join(", "))
            }
            ElementKind::Sequence(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "({})", elems.join("; "))
            }
            ElementKind::Bundle(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| e.to_string()).collect();

                write!(f, "{{{}}}", elems.join(", "))
            }

            ElementKind::Binary { left, operator, right } => write!(f, "({} {} {})", left, operator, right),
            ElementKind::Unary { operator, operand: element } => write!(f, "({}{})", operator, element),

            ElementKind::Labeled { label: element, element: ty } => write!(f, "{}: {}", element, ty),
            ElementKind::Index { element, index } => write!(f, "{}[{}]", element, index),
            ElementKind::Invoke { target, parameters } => {
                let args_str: Vec<String> = parameters.iter().map(|e| e.to_string()).collect();
                write!(f, "{}({})", target, args_str.join(", "))
            }
            ElementKind::Member { object, member } => write!(f, "{}.{}", object, member),

            ElementKind::Scope(stmts) => {
                if stmts.is_empty() {
                    write!(f, "{{}}")
                } else {
                    let stmts_str: Vec<String> = stmts.iter().map(|e| indent(&e.to_string())).collect();
                    write!(f, "{{\n{}\n}}", stmts_str.join("\n"))
                }
            }
            ElementKind::Match { target, body } => write!(f, "match {} {}\n", target, body),
            ElementKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                write!(f, "if {} {}\n", condition, then_branch)?;

                if let Some(else_expr) = else_branch {
                    write!(f, " else {}\n", else_expr)?;
                }

                Ok(())
            }
            ElementKind::Loop { condition, body } => {
                if let Some(condition) = condition {
                    write!(f, "while {} {}\n", condition, body)
                } else {
                    write!(f, "loop {}", body)
                }
            },
            ElementKind::Iterate { clause, body} => write!(f, "for {} {}\n", clause, body),

            ElementKind::Item(item) => write!(f, "{}", item),
            ElementKind::Assignment { target, value} => write!(f, "{} = {}", target, value),
            ElementKind::Constructor { name, body } => {
                write!(f, "{} {}", name, body)
            }

            ElementKind::Return(element) => {
                write!(f, "return")?;

                if let Some(element) = element {
                    write!(f, " {}", element)?;
                }

                Ok(())
            }
            ElementKind::Break(element) => {
                write!(f, "break")?;

                if let Some(element) = element {
                    write!(f, " {}", element)?;
                }

                Ok(())
            }
            ElementKind::Skip(element) => {
                write!(f, "continue")?;

                if let Some(element) = element {
                    write!(f, " {}", element)?;
                }

                Ok(())
            }

            ElementKind::Bind { key, value } => write!(f, "{} => {}", key, value),
            ElementKind::Path { tree } => write!(f, "{}", tree),

            ElementKind::Invalid(e) => write!(f, "error: {}", e)
        }
    }
}

impl Debug for ElementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ElementKind::Literal(literal) => {
                write!(f, "{:?}", literal)
            },
            ElementKind::Identifier(identifier) => {
                write!(f, "Identifier({})", identifier)
            },
            ElementKind::Procedural(element) => {
                write!(f, "Procedural({:?})", element)
            }
            ElementKind::Series(elements) => {
                write!(f, "Series({:?})", elements)
            }
            ElementKind::Collection(elements) => {
                write!(f, "Collection({:?})", elements)
            },
            ElementKind::Group(elements) => {
                write!(f, "Group({:?})", elements)
            },
            ElementKind::Sequence(elements) => {
                write!(f, "Sequence({:?})", elements)
            }
            ElementKind::Bundle(elements) => {
                write!(f, "Bundle({:?})", elements)
            }

            ElementKind::Binary { left, operator, right } => {
                write!(f, "Binary({:?} {:?} {:?})", left, operator, right)
            }
            ElementKind::Unary { operator, operand: element } => {
                write!(f, "Unary({:?} {:?})", operator, element)
            },

            ElementKind::Labeled { label: element, element: ty } => {
                write!(f, "Labeled({:?}: {:?})", element, ty)
            },
            ElementKind::Index { element, index } => {
                write!(f, "Index({:?}[{:?}])", element, index)
            },
            ElementKind::Invoke { target, parameters } => {
                write!(f, "Invoke({:?}({:?}))", target, parameters)
            },
            ElementKind::Member { object, member} => {
                write!(f, "Member({:?}.{:?})", object, member)
            },

            ElementKind::Match { target, body } => {
                write!(f, "Match({:?} => {:?})", target, body)
            },
            ElementKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                write!(f, "Conditional({:?} | Then: {:?}", condition, then_branch)?;

                if let Some(else_expr) = else_branch {
                    write!(f, " | Else: {:?}", else_expr)?;
                }

                write!(f, ")")
            }
            ElementKind::Loop { condition, body } => {
                if let Some(condition) = condition {
                    write!(f, "While({:?} | {:?})", condition, body)
                } else {
                    write!(f, "Loop({:?})", body)
                }
            },
            ElementKind::Iterate { clause, body } => {
                write!(f, "For({:?} in {:?})", clause, body)
            },
            ElementKind::Scope(stmts) => {
                write!(f, "Block({:#?})", stmts)
            },

            ElementKind::Assignment { target, value } => {
                write!(f, "Assignment({:?} = {:?})", target, value)
            },
            ElementKind::Constructor { name, body } => {
                write!(f, "Constructor({:?} | {:?})", name, body)
            },

            ElementKind::Item(item) => write!(f, "+ {:?}", item),

            ElementKind::Return(element) => {
                write!(f, "Return")?;

                if let Some(element) = element {
                    write!(f, "({:?})", element)?;
                }

                Ok(())
            }
            ElementKind::Break(element) => {
                write!(f, "Break")?;

                if let Some(element) = element {
                    write!(f, "({:?})", element)?;
                }

                Ok(())
            }
            ElementKind::Skip(element) => {
                write!(f, "Continue")?;

                if let Some(element) = element {
                    write!(f, "({:?})", element)?;
                }

                Ok(())
            }

            ElementKind::Bind { key, value } => write!(f, "Bind({:?} => {:?})", key, value),
            ElementKind::Path { tree } => write!(f, "Path({:?})", tree),

            ElementKind::Invalid(e) => write!(f, "Error({:?})", e)
        }
    }
}