use {
    crate::{
        format::{
            Debug, Display,
            Formatter, Result
        },
        
        axo_parser::{
            symbol::Symbol,
            Element, ElementKind, SymbolKind
        },
        
        axo_format::indent,
    },
};

impl Debug for SymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SymbolKind::Inclusion { target } => write!(f, "Inclusion({:?})", target),
            SymbolKind::Formation { identifier, form } => write!(f, "Formed({:?}: {:?})", identifier, form),
            SymbolKind::Implementation { element, body } => write!(f, "Implement({:?} => {:?})", element, body),
            SymbolKind::Interface { name, body} => write!(f, "Trait({:?} {:?})", name, body),
            SymbolKind::Slot { target, value, ty } => {
                write!(f, "Slot({:?}", target)?;

                if let Some(ty) = ty {
                    write!(f, " : {:?}", ty)?;
                }

                if let Some(value) = value {
                    write!(f, " = {:?}", value)?;
                }

                write!(f, ")")
            },
            SymbolKind::Binding { target, value, mutable, ty } => {
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
            SymbolKind::Structure { name, fields } => write!(f, "Structure({:?} | {:?})", name, fields),
            SymbolKind::Enumeration { name, variants } => write!(f, "Enumeration({:?} | {:?})", name, variants),
            SymbolKind::Function { name, parameters, body } => write!(f, "Function({:?}({:?}) {:?})", name, parameters, body),
        }
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?} | {:#?}", self.kind, self.span)
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.kind)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?} | {:#?}", self.kind, self.span)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.kind)
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
            ElementKind::Index { target: element, indexes } => {
                write!(f, "Index({:?}[{:?}])", element, indexes)
            },
            ElementKind::Invoke { target, arguments: parameters } => {
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
            ElementKind::Cycle { condition, body } => {
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
            ElementKind::Constructor { name, fields: body } => {
                write!(f, "Constructor({:?} | {:?})", name, body)
            },

            ElementKind::Symbolization(symbol) => write!(f, "+ {:?}", symbol),

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

            ElementKind::Path { tree } => write!(f, "Path({:?})", tree),
        }
    }
}