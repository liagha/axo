use {
    super::{
        Formable,
        form::{
            Form
        },
        classifier::Classifier,
    },
    crate::{
        vector::Show,
        format::{Debug, Display, Formatter, Result},
    },
};

impl<Input: Formable, Output: Formable, Failure: Formable> Debug for Classifier<Input, Output, Failure> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Todo")
    }
}


impl<Input: Formable, Output: Formable, Failure: Formable> Display for Form<Input, Output, Failure> {
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