use {
    super::{
        former::{Form, FormKind},
        pattern::PatternKind,
        action::Action,
    },

    crate::{
        format::{Debug, Display, Formatter, Result},
    }
};

impl<Input, Output, Error> Display for Form<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + PartialEq + Debug,
    Error: Clone + PartialEq + Debug,
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

            FormKind::Error(error) => {
                write!(f, "Error({:?})", error)
            }
        }
    }
}

impl<Input, Output, Error> Debug for PatternKind<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + PartialEq + Debug,
    Error: Clone + PartialEq + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PatternKind::Guard { .. } => write!(f, "Guard"),
            PatternKind::Deferred(_) => {
                write!(f, "Lazy")
            }
            PatternKind::Capture { pattern, identifier } => {
                write!(f, "Capture({:?} as {:?})", pattern, identifier)
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

impl<Input, Output, Error> Debug for Action<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + PartialEq + Debug,
    Error: Clone + PartialEq + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Action::Map(_) => write!(f, "Map"),
            Action::Ignore => write!(f, "Ignore"),
            Action::Error(_) => write!(f, "Error"),
            Action::Trigger { found, missing } => {
                write!(f, "Trigger({:?}, {:?})", found, missing)
            }
        }
    }
}
