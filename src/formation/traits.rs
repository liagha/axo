use {
    crate::{
        formation::{classifier::Classifier, form::Form, helper::Formable},
        tracker::{Span, Spanned},
    },
};


impl<'form, Input: Formable<'form>, Output: Formable<'form>, Failure: Formable<'form>> Spanned<'form> for Form<'form, Input, Output, Failure> {
    fn span(&self) -> Span<'form> {
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
            identity: self.identity,
            action: self.action.clone(),
            marker: self.marker.clone(),
            position: self.position.clone(),
            consumed: self.consumed.clone(),
            outcome: self.outcome.clone(),
            form: self.form.clone(),
            stack: self.stack.clone(),
            depth: self.depth,
        }
    }
}
