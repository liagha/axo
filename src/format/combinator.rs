use crate::{
    combinator::{Form, Formable},
    format::{Show, Stencil},
};

impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Show<'form>
    for Form<'form, Input, Output, Failure>
{
    fn format(&self, config: Stencil) -> Stencil {
        let base = config.clone().new("Form");
        match self.clone() {
            Form::Blank => base.variant("Blank"),
            Form::Input(input) => base
                .variant("Input")
                .field("value", input.format(config.clone())),
            Form::Output(output) => base
                .variant("Output")
                .field("value", output.format(config.clone())),
            Form::Multiple(forms) => base
                .variant("Multiple")
                .field("values", forms.format(config.clone())),
            Form::Failure(error) => base
                .variant("Failure")
                .field("error", error.format(config.clone())),
            Form::_Phantom(_) => unreachable!(),
        }
    }
}
