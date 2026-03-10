use {
    super::{classifier::Classifier, form::Form, helper::Formable},
    crate::{
        data::Str,
        format::{Display, Formatter, Result, Show},
    },
};
use crate::tracker::{Span, Spanned};

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Show<'form>
    for Form<'form, Input, Output, Failure>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'form> {
        match verbosity {
            0 => {
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
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}

impl<
        'classifier,
        Input: Formable<'classifier> + Show<'classifier>,
        Output: Formable<'classifier> + Show<'classifier>,
        Failure: Formable<'classifier> + Show<'classifier>,
    > Show<'classifier> for Classifier<'classifier, Input, Output, Failure>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'classifier> {
        format!(
            "Classifier(\n{}\n{}\n{}\n{}\n{}\n{}\n)",
            format!("marker: {}", self.marker).indent(verbosity),
            format!("consumed: {}", self.consumed.format(verbosity)).indent(verbosity),
            format!("record: {:?}", self.record).indent(verbosity),
            format!("form: {}", self.form.format(verbosity)).indent(verbosity),
            format!("stack: {}", self.stack.format(verbosity)).indent(verbosity),
            format!("depth: {}", self.depth).indent(verbosity),
        )
            .into()
    }
}

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Display
    for Form<'form, Input, Output, Failure>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "")
    }
}

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Spanned<'form> for Form<'form, Input, Output, Failure> {
    fn borrow_span(&self) -> Span<'form> {
        match self.clone() {
            Form::Blank => {
                Span::void()
            }
            Form::Input(input) => {
                input.span()
            }
            Form::Output(output) => {
                output.span()
            }
            Form::Multiple(multiple) => {
                multiple.span()
            }
            Form::Failure(failure) => {
                failure.span()
            }
            Form::_Phantom(_) => {
                unreachable!()
            }
        }
    }

    fn span(self) -> Span<'form> {
        match self {
            Form::Blank => {
                Span::void()
            }
            Form::Input(input) => {
                input.span()
            }
            Form::Output(output) => {
                output.span()
            }
            Form::Multiple(multiple) => {
                multiple.span()
            }
            Form::Failure(failure) => {
                failure.span()
            }
            Form::_Phantom(_) => {
                unreachable!()
            }
        }
    }
}
