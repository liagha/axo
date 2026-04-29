use crate::combinator::{
    Alternative, Combinator, Deferred, Fail, Form, Formable, Formation, Former, Ignore, Literal,
    Memo, Multiple, Optional, Outcome, Panic, Predicate, Record, Recover, Repetition, Sequence,
    Skip, Transform,
};
use crate::data::{
    memory::{replace, take},
    Identity, Offset, Scale,
};
use crate::tracker::Peekable;

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
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
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        for combinator in self.combinators.iter() {
            combinator.combinator(former, formation);
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Literal<'a, 'source, Input>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        match former.source.get(formation.marker) {
            Some(input) if self.value.eq(input) => {
                formation.set_align();
                former.push(formation, input.clone());
            }
            _ => formation.set_empty(),
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Predicate<'a, 'source, Input>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        match former.source.get(formation.marker) {
            Some(input) if (self.function)(input) => {
                formation.set_align();
                former.push(formation, input.clone());
            }
            _ => formation.set_empty(),
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure, const SIZE: Scale>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Alternative<Formation<'a, 'source, Source, Input, Output, Failure>, SIZE>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let mut best: Option<Formation<'a, 'source, Source, Input, Output, Failure>> = None;
        let mut point = (former.consumed.len(), former.forms.len());

        let mut consumed = take(&mut formation.consumed);
        let mut stack = take(&mut formation.stack);
        let base = (consumed.len(), stack.len());

        for (index, state) in self.states.iter().enumerate() {
            let mut child = Formation::create(
                state.combinator.clone(),
                formation.marker,
                formation.state,
                consumed,
                Outcome::Blank,
                0,
                stack,
                formation.depth + 1,
            );

            former.build(&mut child);

            if child.is_blank() {
                (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));
                consumed.truncate(base.0);
                stack.truncate(base.1);
                former.consumed.truncate(point.0);
                former.forms.truncate(point.1);
                continue;
            }

            let better = match &best {
                Some(old) => (self.compare)(&child, old),
                None => true,
            };

            if better {
                let halted = (self.halt)(&child);
                let last = index == self.states.len() - 1;

                if !last && !halted {
                    if let Some(old) = best.take() {
                        (consumed, stack) = (old.consumed, old.stack);
                        consumed.truncate(base.0);
                        stack.truncate(base.1);
                    } else {
                        consumed = child.consumed[..base.0].to_vec();
                        stack = child.stack[..base.1].to_vec();
                    }
                } else {
                    consumed = Vec::new();
                    stack = Vec::new();
                }

                point = (former.consumed.len(), former.forms.len());
                best = Some(child);

                if halted {
                    break;
                }
            } else {
                (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));
                consumed.truncate(base.0);
                stack.truncate(base.1);
                former.consumed.truncate(point.0);
                former.forms.truncate(point.1);
            }
        }

        match best {
            Some(mut state) => {
                formation.outcome = state.outcome;
                formation.marker = state.marker;
                formation.state = state.state;
                formation.consumed = take(&mut state.consumed);
                formation.form = state.form;
                formation.stack = take(&mut state.stack);
            }
            None => {
                formation.set_empty();
                formation.consumed = consumed;
                formation.stack = stack;
            }
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure> Clone
    for Deferred<Formation<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    fn clone(&self) -> Self {
        Self {
            factory: self.factory,
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Deferred<Formation<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let id = self.factory as usize;
        let key = (id, formation.marker);

        if let Some(memo) = (&former.memo.get(&key)).cloned() {
            apply_memo(former, formation, &memo);
            return;
        }

        let combinator = match former.stash.iter().find(|(item, _)| *item == id) {
            Some((_, combinator)) => combinator.clone(),
            None => {
                let state = (self.factory)();
                former.stash.push((id, state.combinator.clone()));
                state.combinator
            }
        };

        let memo = record_memo(former, formation, combinator);

        if former.memo.len() > 2048 {
            former.memo.clear();
        }

        former.memo.insert(key, memo);
    }
}

fn apply_memo<'a, 'source, Source, Input, Output, Failure>(
    former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
    formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    memo: &Memo<'a, Source, Input, Output, Failure>,
) where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    if let Some(record) = &memo.record {
        let delta = (
            former.forms.len() as isize - record.form_base as isize,
            former.consumed.len() as isize - record.input_base as isize,
        );

        former.forms.extend(record.forms.iter().cloned());
        former.consumed.extend(record.inputs.iter().cloned());

        formation.consumed.extend(
            record
                .consumed
                .iter()
                .map(|&id| (id as isize + delta.1) as Identity),
        );

        formation.stack.extend(record.stack.iter().map(|&id| {
            if id == 0 {
                0
            } else {
                (id as isize + delta.0) as Identity
            }
        }));

        formation.form = if record.form == 0 {
            0
        } else {
            (record.form as isize + delta.0) as Identity
        };
    } else {
        formation.form = 0;
    }

    formation.marker += memo.advance;
    formation.state = memo.state;
    formation.outcome = memo.outcome;
}

fn record_memo<'a, 'source, Source, Input, Output, Failure>(
    former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
    formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    combinator: crate::data::memory::Arc<
        dyn Combinator<
                'a,
                Former<'a, 'source, Source, Input, Output, Failure>,
                Formation<'a, 'source, Source, Input, Output, Failure>,
            > + Send
            + Sync
            + 'source,
    >,
) -> Memo<'a, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    let consumed = take(&mut formation.consumed);
    let stack = take(&mut formation.stack);
    let base = (
        consumed.len(),
        stack.len(),
        former.forms.len() as Offset,
        former.consumed.len() as Offset,
        formation.marker,
    );

    let mut child = Formation::create(
        combinator,
        formation.marker,
        formation.state,
        consumed,
        Outcome::Blank,
        0,
        stack,
        formation.depth + 1,
    );

    former.build(&mut child);

    let record = if !former.forms[base.2 as usize..].is_empty()
        || !former.consumed[base.3 as usize..].is_empty()
        || !child.consumed[base.0..].is_empty()
        || !child.stack[base.1..].is_empty()
        || child.form != 0
    {
        Some(Box::new(Record {
            forms: former.forms[base.2 as usize..].to_vec().into_boxed_slice(),
            inputs: former.consumed[base.3 as usize..]
                .to_vec()
                .into_boxed_slice(),
            consumed: child.consumed[base.0..].to_vec().into_boxed_slice(),
            stack: child.stack[base.1..].to_vec().into_boxed_slice(),
            form: child.form,
            form_base: base.2,
            input_base: base.3,
        }))
    } else {
        None
    };

    formation.marker = child.marker;
    formation.state = child.state;
    formation.outcome = child.outcome;
    formation.form = child.form;
    formation.consumed = child.consumed;
    formation.stack = child.stack;

    Memo {
        outcome: formation.outcome,
        advance: formation.marker - base.4,
        state: formation.state,
        record,
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Optional<Formation<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let base = (
            former.consumed.len(),
            former.forms.len(),
            formation.consumed.len(),
            formation.stack.len(),
        );

        let mut child = formation.create_child(self.state.combinator.clone());
        former.build(&mut child);

        let outcome = child.outcome;
        let marker = child.marker;
        let state = child.state;
        let form = child.form;

        formation.consumed = child.consumed;
        formation.stack = child.stack;

        if outcome.is_terminal() && matches!(outcome, Outcome::Panicked) {
            formation.marker = marker;
            formation.state = state;
            formation.form = form;
            formation.set_panic();
        } else if matches!(outcome, Outcome::Aligned) {
            formation.marker = marker;
            formation.state = state;
            formation.form = form;
            formation.set_align();
        } else {
            former.consumed.truncate(base.0);
            former.forms.truncate(base.1);
            formation.consumed.truncate(base.2);
            formation.stack.truncate(base.3);
            formation.set_ignore();
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure, const SIZE: Scale>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Sequence<Formation<'a, 'source, Source, Input, Output, Failure>, SIZE>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let mut consumed = take(&mut formation.consumed);
        let mut stack = take(&mut formation.stack);
        let base = (
            formation.marker,
            formation.state,
            former.consumed.len(),
            former.forms.len(),
            consumed.len(),
            stack.len(),
        );

        let mut forms = Vec::with_capacity(SIZE);
        let mut broke = false;

        for state in &self.states {
            let mut child = Formation::create(
                state.combinator.clone(),
                formation.marker,
                formation.state,
                consumed,
                Outcome::Blank,
                0,
                stack,
                formation.depth + 1,
            );

            former.build(&mut child);

            let halted = (self.halt)(&child);
            let kept = (self.keep)(&child);

            (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));

            if halted {
                formation.outcome = child.outcome;
                formation.form = child.form;
                broke = true;
                break;
            }

            if kept {
                forms.push(child.form);
            }

            formation.marker = child.marker;
            formation.state = child.state;
        }

        formation.consumed = consumed;
        formation.stack = stack;

        if broke {
            let saved = if formation.is_failed() || formation.is_panicked() {
                former.forms.get(formation.form).cloned()
            } else {
                None
            };

            formation.marker = base.0;
            formation.state = base.1;
            former.consumed.truncate(base.2);
            former.forms.truncate(base.3);
            formation.consumed.truncate(base.4);
            formation.stack.truncate(base.5);

            if let Some(form) = saved {
                let id = former.forms.len();
                former.forms.push(form);
                formation.form = id;
            }
        } else {
            formation.set_align();

            let form = Form::multiple(
                forms
                    .into_iter()
                    .map(|id| replace(&mut former.forms[id], Form::Blank))
                    .collect(),
            );

            let id = former.forms.len();
            former.forms.push(form);
            formation.form = id;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Repetition<Formation<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let mut consumed = take(&mut formation.consumed);
        let mut stack = take(&mut formation.stack);
        let base = (
            formation.marker,
            formation.state,
            former.consumed.len(),
            former.forms.len(),
            consumed.len(),
            stack.len(),
        );

        let mut forms = Vec::new();

        while former.source.peek_ahead(formation.marker).is_some() {
            let step = (
                former.consumed.len(),
                former.forms.len(),
                consumed.len(),
                stack.len(),
            );

            let mut child = Formation::create(
                self.state.combinator.clone(),
                formation.marker,
                formation.state,
                consumed,
                Outcome::Blank,
                0,
                stack,
                formation.depth + 1,
            );

            former.build(&mut child);

            let halted = (self.halt)(&child);
            let kept = (self.keep)(&child);

            if child.marker == formation.marker && !halted {
                (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));
                former.consumed.truncate(step.0);
                former.forms.truncate(step.1);
                consumed.truncate(step.2);
                stack.truncate(step.3);
                break;
            }

            (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));

            if halted {
                formation.outcome = child.outcome;
                formation.marker = child.marker;
                formation.state = child.state;

                if kept {
                    forms.push(child.form);
                } else {
                    former.consumed.truncate(step.0);
                    former.forms.truncate(step.1);
                    consumed.truncate(step.2);
                    stack.truncate(step.3);
                }
                break;
            }

            if kept {
                formation.outcome = child.outcome;
                formation.marker = child.marker;
                formation.state = child.state;
                forms.push(child.form);
            } else {
                former.consumed.truncate(step.0);
                former.forms.truncate(step.1);
                consumed.truncate(step.2);
                stack.truncate(step.3);
                formation.marker = child.marker;
                formation.state = child.state;
            }

            if let Some(maximum) = self.maximum {
                if forms.len() >= maximum as Identity {
                    break;
                }
            }
        }

        formation.consumed = consumed;
        formation.stack = stack;

        if forms.len() >= self.minimum as Identity {
            if !formation.is_failed() && !formation.is_panicked() {
                formation.set_align();
            }

            let form = Form::multiple(
                forms
                    .into_iter()
                    .map(|id| replace(&mut former.forms[id], Form::Blank))
                    .collect(),
            );

            let id = former.forms.len();
            former.forms.push(form);
            formation.form = id;
        } else {
            formation.marker = base.0;
            formation.state = base.1;
            former.consumed.truncate(base.2);
            former.forms.truncate(base.3);
            formation.consumed.truncate(base.4);
            formation.stack.truncate(base.5);

            if !formation.is_failed() && !formation.is_panicked() {
                formation.set_empty();
            }
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
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
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
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
            former.push(formation, input.clone());
            moved = true;
        }

        if !moved {
            if let Some(input) = former.source.get(formation.marker) {
                former.push(formation, input.clone());
            }
        }

        formation.set_align();
        formation.form = form_id;
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Ignore
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
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
    Combinator<
        'a,
        Former<'a, 'source, Source, Input, Output, Failure>,
        Formation<'a, 'source, Source, Input, Output, Failure>,
    > for Skip
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
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
    Combinator<
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
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
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
    Combinator<
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
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
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
    Combinator<
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
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn combinator(
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
