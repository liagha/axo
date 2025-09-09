use {
    super::{
        Element, ElementKind,
        Symbol, SymbolKind,
    },

    crate::{
        data::memory::discriminant,
        format::{self, Display, Debug, Formatter},
        internal::{
            operation::Ordering,
            hash::{
                Hash, Hasher
            },
        },
        tracker::{Span, Spanned},
    },
};

impl<'element> Debug for Element<'element> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.kind)
        } else {
            write!(f, "{:?} | {:?}", self.kind, self.span)
        }
    }
}

impl<'element> Debug for ElementKind<'element> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                ElementKind::Literal(literal) => {
                    write!(f, "{:#?}", literal)
                },
                ElementKind::Procedural(procedural) => {
                    write!(f, "Procedural({:#?})", procedural.body)
                }
                ElementKind::Delimited(delimited) => {
                    write!(
                        f,
                        "Delimited({}{:#?}{})",
                        format!("{} ", delimited.start),
                        delimited.items
                            .iter()
                            .map(|item| format!("{:#?}", item))
                            .collect::<Vec<_>>()
                            .join(
                                &*delimited.clone().separator.map(
                                    |separator| format!("{} ", separator)
                                ).unwrap_or(" ".to_string())
                            ),
                        format!("{} ", delimited.end)
                    )
                }

                ElementKind::Binary(binary) => {
                    write!(f, "Binary({:#?} {:#?} {:#?})", binary.left, binary.operator, binary.right)
                }
                ElementKind::Unary(unary) => {
                    write!(f, "Unary({:#?} {:#?})", unary.operator, unary.operand)
                },
                
                ElementKind::Closure(closure) => {
                    write!(f, "Closure({:#?} {:#?})", closure.members, closure.body)
                }

                ElementKind::Index(index) => {
                    write!(f, "Index({:#?}[{:#?}])", index.target, index.members)
                },
                ElementKind::Invoke(invoke) => {
                    write!(f, "Invoke({:#?}({:#?}))", invoke.target, invoke.members)
                },

                ElementKind::Conditional(cond) => {
                    write!(f, "Conditional({:#?} | Then: {:#?}", cond.guard, cond.then)?;

                    if let Some(else_expr) = &cond.alternate {
                        write!(f, " | Else: {:#?}", else_expr)?;
                    }

                    write!(f, ")")
                }
                ElementKind::While(repeat) => {
                    if let Some(condition) = &repeat.guard {
                        write!(f, "While({:#?} | {:#?})", condition, repeat.body)
                    } else {
                        write!(f, "Loop({:#?})", repeat.body)
                    }
                },
                ElementKind::Cycle(walk) => {
                    write!(f, "For({:#?} in {:#?})", walk.guard, walk.body)
                },

                ElementKind::Construct(construct) => {
                    write!(f, "Constructor({:#?} | {:#?})", construct.target, construct.members)
                },

                ElementKind::Symbolize(symbol) => write!(f, "+ {:#?}", symbol),

                ElementKind::Return(element) => {
                    write!(f, "Return")?;

                    if let Some(element) = element {
                        write!(f, "({:#?})", element)?;
                    }

                    Ok(())
                }
                ElementKind::Break(element) => {
                    write!(f, "Break")?;

                    if let Some(element) = element {
                        write!(f, "({:#?})", element)?;
                    }

                    Ok(())
                }
                ElementKind::Continue(element) => {
                    write!(f, "Continue")?;

                    if let Some(element) = element {
                        write!(f, "({:#?})", element)?;
                    }

                    Ok(())
                }
            }
        } else {
            match self {
                ElementKind::Literal(literal) => {
                    write!(f, "{:?}", literal)
                },
                ElementKind::Procedural(procedural) => {
                    write!(f, "Procedural({:?})", procedural.body)
                }
                ElementKind::Delimited(delimited) => {
                    write!(
                        f,
                        "Delimited({}{:#?}{})",
                        format!("{} ", delimited.start),
                        delimited.items
                            .iter()
                            .map(|item| format!("{:?}", item))
                            .collect::<Vec<_>>()
                            .join(
                                &*delimited.clone().separator.map(
                                    |separator| format!("{} ", separator)
                                ).unwrap_or(" ".to_string())
                            ),
                        format!("{} ", delimited.end)
                    )
                }

                ElementKind::Binary(binary) => {
                    write!(f, "Binary({:?} {:?} {:?})", binary.left, binary.operator, binary.right)
                }
                ElementKind::Unary(unary) => {
                    write!(f, "Unary({:?} {:?})", unary.operator, unary.operand)
                },

                ElementKind::Closure(closure) => {
                    write!(f, "Closure({:#?} {:#?})", closure.members, closure.body)
                }

                ElementKind::Index(index) => {
                    write!(f, "Index({:?}[{:?}])", index.target, index.members)
                },
                ElementKind::Invoke(invoke) => {
                    write!(f, "Invoke({:?}({:?}))", invoke.target, invoke.members)
                },

                ElementKind::Conditional(cond) => {
                    write!(f, "Conditional({:?} | Then: {:?}", cond.guard, cond.then)?;

                    if let Some(else_expr) = &cond.alternate {
                        write!(f, " | Else: {:?}", else_expr)?;
                    }

                    write!(f, ")")
                }
                ElementKind::While(repeat) => {
                    if let Some(condition) = &repeat.guard {
                        write!(f, "While({:?} | {:?})", condition, repeat.body)
                    } else {
                        write!(f, "Loop({:?})", repeat.body)
                    }
                },
                ElementKind::Cycle(walk) => {
                    write!(f, "For({:?} in {:?})", walk.guard, walk.body)
                },

                ElementKind::Construct(construct) => {
                    write!(f, "Constructor({:?} | {:?})", construct.target, construct.members)
                },

                ElementKind::Symbolize(symbol) => write!(f, "+ {:?}", symbol),

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
                ElementKind::Continue(element) => {
                    write!(f, "Continue")?;

                    if let Some(element) = element {
                        write!(f, "({:?})", element)?;
                    }

                    Ok(())
                }
            }
        }
    }
}

impl<'symbol> Debug for Symbol<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.kind)
        } else {
            write!(f, "{:?} | {:?}", self.kind, self.span)
        }
    }
}

impl<'symbol> Debug for SymbolKind<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                SymbolKind::Inclusion(inclusion) => {
                    write!(f, "Inclusion({:#?})", inclusion.target)
                }
                SymbolKind::Extension(extension) => {
                    write!(f, "Extension(")?;

                    if let Some(extension) = &extension.extension {
                        write!(f, "{:#?}, ", extension)?;
                    }

                    write!(f, "{:#?}, {:#?})", extension.target, extension.members)
                }
                SymbolKind::Binding(binding) => {
                    write!(
                        f,
                        "Binding({} {:#?}",
                        if binding.constant { "Constant" } else { "Variable" },
                        binding.target
                    )?;

                    if let Some(annotation) = &binding.annotation {
                        write!(f, " : {:#?}", annotation)?;
                    }

                    if let Some(value) = &binding.value {
                        write!(f, " = {:#?}", value)?;
                    }

                    write!(f, ")")
                }
                SymbolKind::Structure(structure) => {
                    write!(f, "Structure({:#?} {:#?})", structure.target, structure.members)
                }
                SymbolKind::Enumeration(enumeration) => {
                    write!(f, "Enumeration({:#?} {:#?})", enumeration.target, enumeration.members)
                }
                SymbolKind::Method(method) => {
                    write!(
                        f,
                        "Method({:#?} {:#?}{} -> {:#?} : {:#?})",
                        method.target,
                        method.members,
                        if method.variadic {
                            "- Variadic"
                        } else {
                            ""
                        },
                        method.output,
                        method.body)
                }
                SymbolKind::Module(module) => {
                    write!(f, "Module({:#?})", module.target)
                }
                SymbolKind::Preference(preference) => {
                    write!(f, "Preference({:#?}, {:#?})", preference.target, preference.value)
                }
            }
        } else {
            match self {
                SymbolKind::Inclusion(inclusion) => {
                    write!(f, "Inclusion({:?})", inclusion.target)
                }
                SymbolKind::Extension(extension) => {
                    write!(f, "Extension(")?;

                    if let Some(extension) = &extension.extension {
                        write!(f, "{:?}, ", extension)?;
                    }

                    write!(f, "{:?}, {:?})", extension.target, extension.members)
                }
                SymbolKind::Binding(binding) => {
                    write!(
                        f,
                        "Binding({} {:?}",
                        if binding.constant { "Constant" } else { "Variable" },
                        binding.target
                    )?;

                    if let Some(annotation) = &binding.annotation {
                        write!(f, " : {:?}", annotation)?;
                    }

                    if let Some(value) = &binding.value {
                        write!(f, " = {:?}", value)?;
                    }

                    write!(f, ")")
                }
                SymbolKind::Structure(structure) => {
                    write!(f, "Structure({:?} {:?})", structure.target, structure.members)
                }
                SymbolKind::Enumeration(enumeration) => {
                    write!(f, "Enumeration({:?} {:?})", enumeration.target, enumeration.members)
                }
                SymbolKind::Method(method) => {
                    write!(
                        f,
                        "Method({:?} {:?}{} -> {:?} : {:?})",
                        method.target,
                        method.members,
                        if method.variadic {
                            "- Variadic"
                        } else {
                            ""
                        },
                        method.output,
                        method.body)
                }
                SymbolKind::Module(module) => {
                    write!(f, "Module({:?})", module.target)
                }
                SymbolKind::Preference(preference) => {
                    write!(f, "Preference({:?}, {:?})", preference.target, preference.value)
                }
            }
        }
    }
}

impl<'element> Hash for Element<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'element> Spanned<'element> for Element<'element> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'element> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'element> {
        self.span
    }
}

impl<'element> Hash for ElementKind<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ElementKind::Literal(kind) => {
                discriminant(self).hash(state);
                kind.hash(state);
            }
            ElementKind::Procedural(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }

            ElementKind::Delimited(delimited) => {
                discriminant(self).hash(state);
                delimited.hash(state);
            }

            ElementKind::Construct(construct) => {
                discriminant(self).hash(state);
                construct.hash(state);
            }

            ElementKind::Binary(binary) => {
                discriminant(self).hash(state);
                binary.hash(state);
            }
            ElementKind::Unary(unary) => {
                discriminant(self).hash(state);
                unary.hash(state);
            }
            
            ElementKind::Closure(closure) => {
                discriminant(self).hash(state);
                closure.hash(state);
            }

            ElementKind::Index(index) => {
                discriminant(self).hash(state);
                index.hash(state);
            }
            ElementKind::Invoke(invoke) => {
                discriminant(self).hash(state);
                invoke.hash(state);
            }

            ElementKind::Conditional(conditioned) => {
                discriminant(self).hash(state);
                conditioned.hash(state);
            }
            ElementKind::While(repeat) => {
                discriminant(self).hash(state);
                repeat.hash(state);
            }
            ElementKind::Cycle(walk) => {
                discriminant(self).hash(state);
                walk.hash(state);
            }

            ElementKind::Symbolize(symbol) => {
                discriminant(self).hash(state);
                symbol.hash(state);
            }

            ElementKind::Return(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
            ElementKind::Break(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
            ElementKind::Continue(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
        }
    }
}

impl<'element> PartialEq for Element<'element> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'element> PartialEq for ElementKind<'element> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ElementKind::Literal(a), ElementKind::Literal(b)) => a == b,
            (ElementKind::Procedural(a), ElementKind::Procedural(b)) => a == b,

            (ElementKind::Delimited(a), ElementKind::Delimited(b)) => a == b,
            (ElementKind::Construct(a), ElementKind::Construct(b)) => a == b,

            (ElementKind::Binary(a), ElementKind::Binary(b)) => a == b,
            (ElementKind::Unary(a), ElementKind::Unary(b)) => a == b,

            (ElementKind::Index(a), ElementKind::Index(b)) => a == b,
            (ElementKind::Invoke(a), ElementKind::Invoke(b)) => a == b,

            (ElementKind::Conditional(a), ElementKind::Conditional(b)) => a == b,
            (ElementKind::While(a), ElementKind::While(b)) => a == b,
            (ElementKind::Cycle(a), ElementKind::Cycle(b)) => a == b,

            (ElementKind::Symbolize(a), ElementKind::Symbolize(b)) => a == b,

            (ElementKind::Return(a), ElementKind::Return(b)) => a == b,
            (ElementKind::Break(a), ElementKind::Break(b)) => a == b,
            (ElementKind::Continue(a), ElementKind::Continue(b)) => a == b,

            _ => false,
        }
    }
}

impl<'element> Clone for Element<'element> {
    fn clone(&self) -> Self {
        Element {
            kind: self.kind.clone(),
            span: self.span.clone(),
        }
    }
}

impl<'element> Clone for ElementKind<'element> {
    fn clone(&self) -> Self {
        match self {
            ElementKind::Literal(kind) => ElementKind::Literal(kind.clone()),
            ElementKind::Procedural(element) => ElementKind::Procedural(element.clone()),

            ElementKind::Delimited(delimited) => ElementKind::Delimited(delimited.clone()),
            ElementKind::Construct(construct) => ElementKind::Construct(construct.clone()),

            ElementKind::Binary(binary) => ElementKind::Binary(binary.clone()),
            ElementKind::Unary(unary) => ElementKind::Unary(unary.clone()),
            
            ElementKind::Closure(closure) => ElementKind::Closure(closure.clone()),

            ElementKind::Index(index) => ElementKind::Index(index.clone()),
            ElementKind::Invoke(invoke) => ElementKind::Invoke(invoke.clone()),

            ElementKind::Conditional(conditioned) => ElementKind::Conditional(conditioned.clone()),
            ElementKind::While(repeat) => ElementKind::While(repeat.clone()),
            ElementKind::Cycle(walk) => ElementKind::Cycle(walk.clone()),

            ElementKind::Symbolize(symbol) => ElementKind::Symbolize(symbol.clone()),

            ElementKind::Return(element) => ElementKind::Return(element.clone()),
            ElementKind::Break(element) => ElementKind::Break(element.clone()),
            ElementKind::Continue(element) => ElementKind::Continue(element.clone()),
        }
    }
}

impl<'element> Eq for Element<'element> {}

impl<'element> Eq for ElementKind<'element> {}

impl<'symbol> Clone for Symbol<'symbol> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            span: self.span.clone(),
            scope: self.scope.clone(),
            specifier: self.specifier.clone(),
        }
    }
}

impl<'symbol> Display for Symbol<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self)
    }
}

impl<'symbol> Eq for Symbol<'symbol> {}

impl<'symbol> PartialEq for Symbol<'symbol> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'symbol> Hash for Symbol<'symbol> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'symbol> PartialOrd for Symbol<'symbol> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'symbol> Ord for Symbol<'symbol> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}