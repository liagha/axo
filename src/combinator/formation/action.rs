use crate::combinator::{
    Action, Classifier, Fail, Form, Formable, Former, Ignore, Multiple, Panic, Recover, Skip,
    Transform,
};
use crate::tracker::Peekable;

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
>
for Multiple<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        for action in self.actions.iter() {
            action.action(former, classifier);
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
>
for Recover<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
    Input,
    Failure,
>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !classifier.is_failed() && !classifier.is_panicked() {
            return;
        }

        let failure = (self.emitter)(former, classifier.clone());
        let form_id = former.forms.len();
        former.forms.push(Form::Failure(failure));

        let mut moved = false;
        while let Some(input) = former.source.get(classifier.marker) {
            if (self.sync)(input) {
                break;
            }

            former
                .source
                .next(&mut classifier.marker, &mut classifier.state);

            let consumed_id = former.consumed.len();
            let stack_id = former.forms.len();

            former.consumed.push(input.clone());
            former.forms.push(Form::input(input.clone()));

            classifier.consumed.push(consumed_id);
            classifier.stack.push(stack_id);
            moved = true;
        }

        if !moved {
            if let Some(input) = former.source.get(classifier.marker) {
                former
                    .source
                    .next(&mut classifier.marker, &mut classifier.state);

                let consumed_id = former.consumed.len();
                let stack_id = former.forms.len();

                former.consumed.push(input.clone());
                former.forms.push(Form::input(input.clone()));

                classifier.consumed.push(consumed_id);
                classifier.stack.push(stack_id);
            }
        }

        classifier.set_align();
        classifier.form = form_id;
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Ignore
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        _former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            classifier.set_ignore();
            classifier.form = 0;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Skip
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        _former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            classifier.set_empty();
            classifier.form = 0;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
>
for Transform<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
    Failure,
>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            if let Err(error) = (self.transformer)(former, classifier) {
                let form_id = former.forms.len();
                former.forms.push(Form::Failure(error));

                classifier.set_fail();
                classifier.form = form_id;
            }
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
>
for Fail<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
    Failure,
>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !classifier.is_aligned() {
            let failure = (self.emitter)(former, classifier.clone());

            let form_id = former.forms.len();
            former.forms.push(Form::Failure(failure));

            classifier.set_fail();
            classifier.form = form_id;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
>
for Panic<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
    Failure,
>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !classifier.is_aligned() {
            let failure = (self.emitter)(former, classifier.clone());

            let form_id = former.forms.len();
            former.forms.push(Form::Failure(failure));

            classifier.set_panic();
            classifier.form = form_id;
        }
    }
}
