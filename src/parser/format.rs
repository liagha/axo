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
            ElementKind::Procedural(procedural) => {
                write!(f, "Procedural({:?})", procedural.body)
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
                write!(f, "Binary({:?} {:?} {:?})", binary.left, binary.operator, binary.right)
            }
            ElementKind::Unary(unary) => {
                write!(f, "Unary({:?} {:?})", unary.operator, unary.operand)
            },

            ElementKind::Label(label) => {
                write!(f, "Labeled({:?}: {:?})", label.label, label.element)
            },
            ElementKind::Index(index) => {
                write!(f, "Index({:?}[{:?}])", index.target, index.indexes)
            },
            ElementKind::Invoke(invoke) => {
                write!(f, "Invoke({:?}({:?}))", invoke.target, invoke.arguments)
            },
            ElementKind::Access(access) => {
                write!(f, "Access({:?}.{:?})", access.target, access.member)
            },

            ElementKind::Conditional(cond) => {
                write!(f, "Conditional({:?} | Then: {:?}", cond.condition, cond.then)?;

                if let Some(else_expr) = &cond.alternate {
                    write!(f, " | Else: {:?}", else_expr)?;
                }

                write!(f, ")")
            }
            ElementKind::While(repeat) => {
                if let Some(condition) = &repeat.condition {
                    write!(f, "While({:?} | {:?})", condition, repeat.body)
                } else {
                    write!(f, "Loop({:?})", repeat.body)
                }
            },
            ElementKind::Cycle(walk) => {
                write!(f, "For({:?} in {:?})", walk.clause, walk.body)
            },

            ElementKind::Assign(assign) => {
                write!(f, "Assignment({:?} = {:?})", assign.target, assign.value)
            },
            ElementKind::Construct(construct) => {
                write!(f, "Constructor({:?} | {:?})", construct.target, construct.fields)
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