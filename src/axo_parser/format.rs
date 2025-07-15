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
            SymbolKind::Inclusion(inclusion) => write!(f, "Inclusion({:?})", inclusion.get_target()),
            SymbolKind::Formation(formation) => write!(f, "Formed({:?}: {:?})", formation.get_identifier(), formation.get_form()),
            SymbolKind::Implementation(implementation) => write!(f, "Implement({:?} => {:?})", implementation.get_target(), implementation.get_body()),
            SymbolKind::Interface(interface) => write!(f, "Trait({:?} {:?})", interface.get_target(), interface.get_body()),
            SymbolKind::Binding(binding) => {
                let kind = if binding.is_mutable() { "Variable" } else { "Constant" };
                write!(f, "{}({:?}", kind, binding.get_target())?;

                if let Some(ty) = binding.get_type() {
                    write!(f, " : {:?}", ty)?;
                }

                if let Some(value) = binding.get_value() {
                    write!(f, " = {:?}", value)?;
                }

                write!(f, ")")
            },
            SymbolKind::Structure(structure) => write!(f, "Structure({:?} | {:?})", structure.get_name(), structure.get_fields()),
            SymbolKind::Enumeration(enumeration) => write!(f, "Enumeration({:?} | {:?})", enumeration.get_name(), enumeration.get_variants()),
            SymbolKind::Function(function) => write!(f, "Function({:?}({:?}) {:?})", function.get_name(), function.get_parameters(), function.get_body()),
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
            ElementKind::Series(series) => {
                write!(f, "Series({:?})", series.items)
            }
            ElementKind::Collection(collection) => {
                write!(f, "Collection({:?})", collection.items)
            },
            ElementKind::Group(group) => {
                write!(f, "Group({:?})", group.items)
            },
            ElementKind::Sequence(sequence) => {
                write!(f, "Sequence({:?})", sequence.items)
            }
            ElementKind::Bundle(bundle) => {
                write!(f, "Bundle({:?})", bundle.items)
            },
            ElementKind::Scope(scope) => {
                write!(f, "Block({:#?})", scope.items)
            },

            ElementKind::Binary(binary) => {
                write!(f, "Binary({:?} {:?} {:?})", binary.get_left(), binary.get_operator(), binary.get_right())
            }
            ElementKind::Unary(unary) => {
                write!(f, "Unary({:?} {:?})", unary.get_operator(), unary.get_operand())
            },

            ElementKind::Label(label) => {
                write!(f, "Labeled({:?}: {:?})", label.get_label(), label.get_element())
            },
            ElementKind::Index(index) => {
                write!(f, "Index({:?}[{:?}])", index.get_target(), index.get_indexes())
            },
            ElementKind::Invoke(invoke) => {
                write!(f, "Invoke({:?}({:?}))", invoke.get_target(), invoke.get_arguments())
            },
            ElementKind::Access(access) => {
                write!(f, "Member({:?}.{:?})", access.get_object(), access.get_member())
            },

            ElementKind::Conditional(cond) => {
                write!(f, "Conditional({:?} | Then: {:?}", cond.get_condition(), cond.get_then())?;

                if let Some(else_expr) = cond.get_alternate() {
                    write!(f, " | Else: {:?}", else_expr)?;
                }

                write!(f, ")")
            }
            ElementKind::Repeat(repeat) => {
                if let Some(condition) = repeat.get_condition() {
                    write!(f, "While({:?} | {:?})", condition, repeat.get_body())
                } else {
                    write!(f, "Loop({:?})", repeat.get_body())
                }
            },
            ElementKind::Iterate(walk) => {
                write!(f, "For({:?} in {:?})", walk.get_clause(), walk.get_body())
            },

            ElementKind::Assign(assign) => {
                write!(f, "Assignment({:?} = {:?})", assign.get_target(), assign.get_value())
            },
            ElementKind::Construct(construct) => {
                write!(f, "Constructor({:?} | {:?})", construct.get_target(), construct.get_fields())
            },

            ElementKind::Symbolize(symbol) => write!(f, "+ {:?}", symbol),

            ElementKind::Produce(element) => {
                write!(f, "Return")?;

                if let Some(element) = element {
                    write!(f, "({:?})", element)?;
                }

                Ok(())
            }
            ElementKind::Abort(element) => {
                write!(f, "Break")?;

                if let Some(element) = element {
                    write!(f, "({:?})", element)?;
                }

                Ok(())
            }
            ElementKind::Pass(element) => {
                write!(f, "Continue")?;

                if let Some(element) = element {
                    write!(f, "({:?})", element)?;
                }

                Ok(())
            }

            ElementKind::Domain(tree) => write!(f, "Path({:?})", tree),
        }
    }
}