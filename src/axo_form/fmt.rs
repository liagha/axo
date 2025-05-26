use crate::format::{Display, Debug, Formatter, Result};
use crate::axo_form::{Action, FormKind, Form, PatternKind};

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
            
            FormKind::Raw(raw) => {
                write!(f, "Raw({:?})", raw)
            }
            
            FormKind::Single(single) => {
                write!(f, "Single({:?})", single)
            }
            
            FormKind::Multiple(forms) => {
                write!(f, "Multiple(")?;
                
                write!(f, "{:?}", forms.iter().map(|form| form.kind.clone()).collect::<Vec<_>>())?;
                
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
            PatternKind::Lazy(_) => {
                write!(f, "Lazy(_)")
            }
            PatternKind::Exact(literal) => {
                write!(f, "Literal({:?})", literal)
            }
            PatternKind::Alternative(patterns) => {
                write!(f, "Alternative({:?})", patterns)
            }
            PatternKind::Sequence(sequence) => {
                write!(f, "Sequence({:?})", sequence)
            }
            PatternKind::Repeat {
                pattern, minimum, maximum
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
            PatternKind::Predicate(_) => write!(f, "Predicate"),
            PatternKind::Negate(_) => write!(f, "Negate"),
            PatternKind::Anything => write!(f, "Anything"),
            PatternKind::Required { 
                pattern,
                action
            } => {
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
            Action::Transform(_) => write!(f, "Transform"),
            Action::Ignore => write!(f, "Ignore"),
            Action::Error(_) => write!(f, "Error"),
            Action::Conditional { found, missing } => write!(f, "Conditional({:?}, {:?})", found, missing),
        }
    }
}