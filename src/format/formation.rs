use {
    crate::{
        data::Str,
        format::{Show, Verbosity},
        formation::{
            form::Form,
            helper::Formable,
        },
    }
};

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Show<'form>
for Form<'form, Input, Output, Failure>
{
    fn format(&self, verbosity: Verbosity) -> Str<'form> {
        match verbosity {
            Verbosity::Minimal => {
                match self.clone() {
                    Form::Blank => {
                        "Blank".to_string()
                    }

                    Form::Input(input) => {
                        format!("Input({})", input.format(verbosity))
                    }

                    Form::Output(output) => {
                        format!("Output({})", output.format(verbosity))
                    }

                    Form::Multiple(forms) => {
                        format!("Multiple({})", forms.format(verbosity))
                    }

                    Form::Failure(error) => {
                        format!("Failure({})", error.format(verbosity))
                    }

                    Form::_Phantom(_) => unreachable!(),
                }
            }
            _ => {
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}
