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
use crate::axo_form::pattern::Pattern;
use crate::format_vec;

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

impl<Input, Output, Failure> Display for Pattern<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.kind)
    }
}

impl<Input, Output, Failure> Display for PatternKind<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PatternKind::Guard { .. } => write!(f, "guard"),
            PatternKind::Deferred(_) => {
                write!(f, "lazy")
            }
            PatternKind::Literal(literal) => {
                write!(f, "{:?}", literal)
            }
            PatternKind::Alternative(patterns) => {
                let patterns = patterns.iter().map(|pattern| pattern.to_string()).collect::<Vec<_>>().join(" | ");
                
                write!(f, "{}", patterns)
            }
            PatternKind::Sequence(sequence) => {
                let patterns = sequence.iter().map(|pattern| pattern.to_string()).collect::<Vec<_>>().join(", ");
                
                write!(f, "{}", patterns)
            }
            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
            } => {
                write!(f, "{}..{}", pattern, minimum)?;

                if let Some(maximum) = maximum {
                    write!(f, "-{}", maximum)?;
                }

                write!(f, ")")
            }
            PatternKind::Optional(pattern) => {
                write!(f, "{}?", pattern)
            }
            PatternKind::Condition(_) => write!(f, "predicate"),
            PatternKind::Negation(_) => write!(f, "negate"),
            PatternKind::WildCard => write!(f, "anything"),
            PatternKind::Wrap(pattern) => write!(f, "wrap({})", pattern),
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
            PatternKind::Wrap(pattern) => {
                write!(f, "Wrap({:?})", pattern)
            }
        }
    }
}

impl<Input, Output, Failure> Display for Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Action::Map(_) => write!(f, "map"),
            Action::Execute(_) => write!(f, "execute"),
            Action::Inspect(_) => write!(f, "inspect"),
            Action::Multiple(actions) => write!(f, "multiple({})", format_vec(actions)),
            Action::Trigger { found, missing } => write!(f, "trigger({}, {})", found, missing),
            Action::Capture { identifier } => write!(f, "capture({})", identifier),
            Action::Ignore => write!(f, "ignore"),
            Action::Failure(_) => write!(f, "failure"),
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
            Action::Execute(_) => write!(f, "Execute"),
            Action::Inspect(_) => write!(f, "Inspect"),
            Action::Multiple(actions) => write!(f, "Multiple({:?})", actions),
            Action::Trigger { found, missing } => write!(f, "Trigger({:?}, {:?})", found, missing),
            Action::Capture { identifier } => write!(f, "Capture({:?})", identifier),
            Action::Ignore => write!(f, "Ignore"),
            Action::Failure(_) => write!(f, "Failure"),
        }
    }
}
