use crate::{
    combinator::{
        Formable,
        formation::classifier::Classifier,
        formation::form::Form,
    },
    tracker::{Span, Spanned, Peekable},
};

impl<'a, Input, Output, Failure> Spanned<'a> for Form<'a, Input, Output, Failure>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    fn span(&self) -> Span<'a> {
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

impl<'a, 'src, Source, Input, Output, Failure> Clone for Classifier<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
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

