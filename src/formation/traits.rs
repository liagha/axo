use {
    super::{classifier::Classifier, form::Form, helper::Formable},
    crate::{
        data::Str,
        format::{Debug, Display, Formatter, Result, Show},
    },
};

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Show<'form>
    for Form<'form, Input, Output, Failure>
{
    type Verbosity = u16;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'form> {
        match verbosity {
            1 => {
                match self.clone() {
                    Form::Blank => {
                        Str::from("Blank")
                    }

                    Form::Input(input) => {
                        Str::from(format!("Input({})", input.format(verbosity)))
                    }

                    Form::Output(output) => {
                        Str::from(format!("Output({})", output.format(verbosity)))
                    }

                    Form::Multiple(forms) => {
                        Str::from(format!("Multiple({})", forms.format(verbosity)))
                    }

                    Form::Failure(error) => {
                        Str::from(format!("Failure({})", error.format(verbosity)))
                    }

                    Form::_Phantom(_) => unreachable!(),
                }
            }
            _ => {
                Str::from("")
            }
        }
    }
}

impl<
        'classifier,
        Input: Formable<'classifier>,
        Output: Formable<'classifier>,
        Failure: Formable<'classifier>,
    > Debug for Classifier<'classifier, Input, Output, Failure>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Todo")
    }
}

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Display
    for Form<'form, Input, Output, Failure>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "")
    }
}
