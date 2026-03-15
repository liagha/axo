use {
    super::{classifier::Classifier, form::Form, helper::Formable},
    crate::{
        data::Str,
        tracker::{Span, Spanned},
        format::{Display, Formatter, Result, Show},
    },
};

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

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Clone for Classifier<'form, Input, Output, Failure> {
    fn clone(&self) -> Self {
        Self {
            order: self.order,
            marker: self.marker.clone(),
            position: self.position.clone(),
            consumed: self.consumed.clone(),
            record: self.record.clone(),
            form: self.form.clone(),
            stack: self.stack.clone(),
            depth: self.depth,
        }
    }
}
