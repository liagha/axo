use {
    super::{
        form::{Form},
        classifier::Classifier,
    },
    crate::{
        hash::Hash,
        vector::Show,
        format::{Debug, Display, Formatter, Result},
    },
};

impl<Input, Output, Failure> Debug for Classifier<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Todo")
    }
}


impl<Input, Output, Failure> Display for Form<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.clone() {
            Form::Blank => {
                write!(f, "Blank")
            }

            Form::Input(input) => {
                write!(f, "Input({:?})", input)
            }

            Form::Output(output) => {
                write!(f, "Output({:?})", output)
            }

            Form::Multiple(forms) => {
                write!(f, "Multiple({})", forms.format())
            }

            Form::Failure(error) => {
                write!(f, "Failure({:?})", error)
            }
        }
    }
}