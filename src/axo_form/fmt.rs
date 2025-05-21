use core::fmt::{Display, Debug, Formatter};
use crate::axo_form::{Action, Form, Formed, PatternKind};

impl<Input, Output, Error> Display for Formed<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.form.clone() {
            Form::Empty => {
                write!(f, "Empty")
            }
            
            Form::Raw(raw) => {
                write!(f, "Raw({:?})", raw)
            }
            
            Form::Single(single) => {
                write!(f, "Single({:?})", single)
            }
            
            Form::Multiple(forms) => {
                write!(f, "Multiple(")?;
                
                write!(f, "{:?}", forms.iter().map(|formed| formed.form.clone()).collect::<Vec<_>>())?;
                
                write!(f, ")")
            }
            
            Form::Error(error) => {
                write!(f, "Error({:?})", error)
            }
        }
    }
}

impl<Input, Output, Error> Debug for PatternKind<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PatternKind::Literal(literal) => {
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
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Action::Transform(_) => write!(f, "Transform"),
            Action::Ignore => write!(f, "Ignore"),
            Action::Error(_) => write!(f, "Error"),
            Action::Conditional { found, missing } => write!(f, "Conditional({:?}, {:?})", found, missing),
        }
    }
}