use {
    super::{
        action::Action,
        form::{Form, FormKind},
        pattern::PatternKind,
    },
    crate::{
        any::{Any},
        vector::Show,
        format::{Debug, Display, Formatter, Result},
        hash::Hash,
    },
};

impl<Input, Output, Failure> Display for Form<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.kind.clone() {
            FormKind::Blank => {
                write!(f, "Empty")
            }

            FormKind::Input(input) => {
                write!(f, "Input({:?})", input)
            }

            FormKind::Output(output) => {
                write!(f, "Output({:?})", output)
            }

            FormKind::Multiple(forms) => {
                write!(f, "Multiple({})", forms.format())
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
            PatternKind::Deferred(function) => {
                write!(f, "Lazy({:?})", function.type_id())
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
                write!(f, "Repeat({:?}", pattern)?;

                if *minimum != 0 {
                    write!(f, ", {}", minimum)?;
                }
                
                if let Some(maximum) = maximum {
                    write!(f, "-{}", maximum)?;
                }

                write!(f, ")")
            }
            PatternKind::Optional(pattern) => {
                write!(f, "Optional({:?})", pattern)
            }
            PatternKind::Predicate(_) => write!(f, "Predicate"),
            PatternKind::Negation(_) => write!(f, "Negate"),
            PatternKind::WildCard => write!(f, "Anything"),
            PatternKind::Wrapper(pattern) => {
                write!(f, "Wrap({:?})", pattern)
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
            Action::Map(function) => write!(f, "Map({:?})", function.type_id()),
            Action::Perform(function) => write!(f, "Execute({:?})", function.type_id()),
            Action::Multiple(actions) => write!(f, "Multiple({:?})", actions),
            Action::Trigger { found, missing } => write!(f, "Trigger({:?}, {:?})", found, missing),
            Action::Capture { identifier } => write!(f, "Capture({:?})", identifier),
            Action::Ignore => write!(f, "Ignore"),
            Action::Skip => write!(f, "Skip"),
            Action::Failure(function) => write!(f, "Failure({:?})", function.type_id()),
        }
    }
}
