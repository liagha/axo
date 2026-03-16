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
        if verbosity == Verbosity::Off {
            return "".into();
        }

        match self.clone() {
            Form::Blank => match verbosity {
                Verbosity::Minimal => "".to_string(),
                Verbosity::Detailed => "Blank".to_string(),
                Verbosity::Debug => "Blank {}".to_string(),
                _ => "".to_string(),
            },
            Form::Input(input) => match verbosity {
                Verbosity::Minimal => input.format(verbosity).to_string(),
                Verbosity::Detailed => format!("Input({})", input.format(verbosity)),
                Verbosity::Debug => format!("Input {{\n{}\n}}", input.format(verbosity).indent(verbosity)),
                _ => "".to_string(),
            },
            Form::Output(output) => match verbosity {
                Verbosity::Minimal => output.format(verbosity).to_string(),
                Verbosity::Detailed => format!("Output({})", output.format(verbosity)),
                Verbosity::Debug => format!("Output {{\n{}\n}}", output.format(verbosity).indent(verbosity)),
                _ => "".to_string(),
            },
            Form::Multiple(forms) => match verbosity {
                Verbosity::Minimal => forms.format(verbosity).to_string(),
                Verbosity::Detailed => format!("Multiple({})", forms.format(verbosity)),
                Verbosity::Debug => format!("Multiple {{\n{}\n}}", forms.format(verbosity).indent(verbosity)),
                _ => "".to_string(),
            },
            Form::Failure(error) => match verbosity {
                Verbosity::Minimal => error.format(verbosity).to_string(),
                Verbosity::Detailed => format!("Failure({})", error.format(verbosity)),
                Verbosity::Debug => format!("Failure {{\n{}\n}}", error.format(verbosity).indent(verbosity)),
                _ => "".to_string(),
            },
            Form::_Phantom(_) => unreachable!(),
        }.into()
    }
}
