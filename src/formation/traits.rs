use {
    super::{classifier::Classifier, form::Form, helper::Formable},
    crate::{
        tracker::{Span, Spanned},
    },
};


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
