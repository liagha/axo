use {
    super::{
        form::{Form, FormKind},
        pattern::PatternKind,
        action::Action,
    },

    crate::{
        hash::Hash,
        format::{Debug, Display, Formatter, Result},
    }
};

impl<Input, Output, Failure> Display for Form<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.kind.clone() {
            FormKind::Empty => {
                write!(f, "Empty")
            }

            FormKind::Input(input) => {
                write!(f, "Input({:?})", input)
            }

            FormKind::Output(output) => {
                write!(f, "Output({:?})", output)
            }

            FormKind::Multiple(forms) => {
                write!(f, "Multiple(")?;

                write!(
                    f,
                    "{}",
                    forms
                        .iter()
                        .map(|form| form.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;

                write!(f, ")")
            }

            FormKind::Failure(error) => {
                write!(f, "Failure({:?})", error)
            }
        }
    }
}

impl<Input, Output, Failure> Debug for PatternKind<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PatternKind::Guard { .. } => write!(f, "Guard"),
            PatternKind::Deferred(_) => {
                write!(f, "Lazy")
            }
            PatternKind::Literal(literal) => {
                write!(f, "Literal({:?})", literal)
            }
            PatternKind::Alternative(patterns) => {
                write!(f, "Alternative({:?})", patterns)
            }
            PatternKind::Sequence(sequence) => {
                write!(f, "Sequence({:?})", sequence)
            }
            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
            } => {
                write!(f, "Repeat({:?}, {}", pattern, minimum)?;

                if let Some(maximum) = maximum {
                    write!(f, "-{}", maximum)?;
                }

                write!(f, ")")
            }
            PatternKind::Optional(pattern) => {
                write!(f, "Optional({:?})", pattern)
            }
            PatternKind::Condition(_) => write!(f, "Predicate"),
            PatternKind::Negation(_) => write!(f, "Negate"),
            PatternKind::WildCard => write!(f, "Anything"),
            PatternKind::Required { pattern, action } => {
                write!(f, "Required({:?}, {:?})", pattern, action)
            }
        }
    }
}

impl<Input, Output, Failure> Debug for Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Action::Map(_) => write!(f, "Map"),
            Action::Ignore => write!(f, "Ignore"),
            Action::Failure(_) => write!(f, "Failure"),
            Action::Capture { identifier } => write!(f, "Capture({:?})", identifier),
            Action::Trigger { found, missing } => {
                write!(f, "Trigger({:?}, {:?})", found, missing)
            }
        }
    }
}
