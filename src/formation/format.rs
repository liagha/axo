use {
    super::{
        classifier::Classifier,
        form::Form,
        helper::Formable,
    },
    crate::{
        format::{
            vector::Show,
            Debug, Display, Formatter, Result
        },
    },
};

impl<'classifier, Input: Formable<'classifier>, Output: Formable<'classifier>, Failure: Formable<'classifier>> Debug for Classifier<'classifier, Input, Output, Failure> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Todo")
    }
}


impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Display for Form<'form, Input, Output, Failure> {
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
            Form::_Phantom(_) => unreachable!(),
        }
    }
}