use crate::{
    combinator::{formation::formation::Formation, formation::form::Form, Formable},
    tracker::{Peekable, Span, Spanned},
};

impl<'a, Input, Output, Failure> Spanned<'a> for Form<'a, Input, Output, Failure>
where
    Input: Formable<'a> + Spanned<'a>,
    Output: Formable<'a> + Spanned<'a>,
    Failure: Formable<'a> + Spanned<'a>,
{
    fn span(&self) -> Span {
        match self {
            Form::Blank => Span::void(),
            Form::Input(input) => input.span(),
            Form::Output(output) => output.span(),
            Form::Multiple(multiple) => multiple.span(),
            Form::Failure(failure) => failure.span(),
            Form::_Phantom(_) => unreachable!(),
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure> Clone
    for Formation<'a, 'source, Source, Input, Output, Failure>
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
            marker: self.marker,
            state: self.state,
            consumed: self.consumed.clone(),
            outcome: self.outcome.clone(),
            form: self.form.clone(),
            stack: self.stack.clone(),
            depth: self.depth,
        }
    }
}
