use {
    super::{
        Element, ElementKind,
    },
    
    crate::{
        format::{
            Debug,
            Formatter, Result
        },
    },
};

impl<'element> Debug for Element<'element> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?} | {:#?}", self.kind, self.span)
    }
}

impl<'element> Debug for ElementKind<'element> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ElementKind::Literal(literal) => {
                write!(f, "{:?}", literal)
            },
            ElementKind::Identifier(identifier) => {
                write!(f, "Identifier({})", identifier)
            },
            ElementKind::Procedural(procedural) => {
                write!(f, "Procedural({:?})", procedural.get_body())
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
            ElementKind::Block(block) => {
                write!(f, "Block({:#?})", block.items)
            }

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
                write!(f, "Access({:?}.{:?})", access.get_target(), access.get_member())
            },

            ElementKind::Conditional(cond) => {
                write!(f, "Conditional({:?} | Then: {:?}", cond.get_condition(), cond.get_then())?;

                if let Some(else_expr) = cond.get_alternate() {
                    write!(f, " | Else: {:?}", else_expr)?;
                }

                write!(f, ")")
            }
            ElementKind::While(repeat) => {
                if let Some(condition) = repeat.get_condition() {
                    write!(f, "While({:?} | {:?})", condition, repeat.get_body())
                } else {
                    write!(f, "Loop({:?})", repeat.get_body())
                }
            },
            ElementKind::Cycle(walk) => {
                write!(f, "For({:?} in {:?})", walk.get_clause(), walk.get_body())
            },

            ElementKind::Assign(assign) => {
                write!(f, "Assignment({:?} = {:?})", assign.get_target(), assign.get_value())
            },
            ElementKind::Construct(construct) => {
                write!(f, "Constructor({:?} | {:?})", construct.get_target(), construct.get_fields())
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