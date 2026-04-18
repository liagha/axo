use crate::combinator::{
    Action, Formation, Fail, Form, Formable, Former, Ignore, Multiple, Panic, Recover, Skip,
    Transform,
};
use crate::tracker::Peekable;

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
>
for Multiple<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        for action in self.actions.iter() {
            action.action(former, formation);
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
>
for Recover<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !formation.is_failed() && !formation.is_panicked() {
            return;
        }

        let failure = (self.emitter)(former, formation.clone());
        let form_id = former.forms.len();
        former.forms.push(Form::Failure(failure));

        let mut moved = false;
        while let Some(input) = former.source.get(formation.marker) {
            if (self.sync)(input) {
                break;
            }

            former
                .source
                .next(&mut formation.marker, &mut formation.state);

            let consumed_id = former.consumed.len();
            let stack_id = former.forms.len();

            former.consumed.push(input.clone());
            former.forms.push(Form::input(input.clone()));

            formation.consumed.push(consumed_id);
            formation.stack.push(stack_id);
            moved = true;
        }

        if !moved {
            if let Some(input) = former.source.get(formation.marker) {
                former
                    .source
                    .next(&mut formation.marker, &mut formation.state);

                let consumed_id = former.consumed.len();
                let stack_id = former.forms.len();

                former.consumed.push(input.clone());
                former.forms.push(Form::input(input.clone()));

                formation.consumed.push(consumed_id);
                formation.stack.push(stack_id);
            }
        }

        formation.set_align();
        formation.form = form_id;
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if formation.is_aligned() {
            formation.set_ignore();
            formation.form = 0;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if formation.is_aligned() {
            formation.set_empty();
            formation.form = 0;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
>
for Transform<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if formation.is_aligned() {
            if let Err(error) = (self.transformer)(former, formation) {
                let form_id = former.forms.len();
                former.forms.push(Form::Failure(error));

                formation.set_fail();
                formation.form = form_id;
            }
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
>
for Fail<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !formation.is_aligned() {
            let failure = (self.emitter)(former, formation.clone());

            let form_id = former.forms.len();
            former.forms.push(Form::Failure(failure));

            formation.set_fail();
            formation.form = form_id;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
>
for Panic<
    'a,
    'source,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
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
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !formation.is_aligned() {
            let failure = (self.emitter)(former, formation.clone());

            let form_id = former.forms.len();
            former.forms.push(Form::Failure(failure));

            formation.set_panic();
            formation.form = form_id;
        }
    }
}
