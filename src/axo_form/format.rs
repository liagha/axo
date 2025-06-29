use {
    super::{
        order::Order,
        form::{Form, FormKind},
        pattern::PatternKind,
    },
    crate::{
        hash::Hash,
        any::{Any},
        vector::Show,
        format::{Debug, Display, Formatter, Result},
        axo_cursor::Spanned,
    },
};

impl<Input, Output, Failure> Display for Form<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.kind.clone() {
            FormKind::Blank => {
                write!(f, "Empty | {:?}", self.span)
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
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PatternKind::Deferred { function } => {
                write!(f, "Lazy({:?})", function.type_id())
            }
            PatternKind::Identical { value } => {
                write!(f, "Identical({:?})", value.type_id())
            }
            PatternKind::Alternative { patterns } => {
                write!(f, "Alternative({:?})", patterns)
            }
            PatternKind::Sequence { patterns } => {
                write!(f, "Sequence({:?})", patterns)
            }
            PatternKind::Repetition {
                pattern,
                minimum,
                maximum,
                ..
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
            PatternKind::Optional { pattern } => {
                write!(f, "Optional({:?})", pattern)
            }
            PatternKind::Predicate { .. } => write!(f, "Predicate"),
            PatternKind::Reject { pattern } => write!(f, "Reject({:?})", pattern),
            PatternKind::Wrapper { pattern } => {
                write!(f, "Wrap({:?})", pattern)
            }
        }
    }
}

impl<Input, Output, Failure> Debug for Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Order::Convert(function) => write!(f, "Map({:?})", function.type_id()),
            Order::Perform(function) => write!(f, "Execute({:?})", function.type_id()),
            Order::Multiple(actions) => write!(f, "Multiple({:?})", actions),
            Order::Trigger { found, missing } => write!(f, "Trigger({:?}, {:?})", found, missing),
            Order::Capture(identifier) => write!(f, "Capture({:?})", identifier),
            Order::Ignore => write!(f, "Ignore"),
            Order::Skip => write!(f, "Skip"),
            Order::Shift(function) => write!(f, "Shifter({:?})", function.type_id()),
            Order::Failure(function) => write!(f, "Failure({:?})", function.type_id()),
            Order::Tweak(function) => write!(f, "Tweak({:?})", function.type_id()),
            Order::Remove => write!(f, "Remove"),
            Order::Pardon => write!(f, "Pardon"),
        }
    }
}