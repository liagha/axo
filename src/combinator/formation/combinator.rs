use crate::combinator::{
    formation::Joint,
    Alternative, Combinator, Deferred, Fail, Form, Formable, Formation, Former, Ignore,
    Literal, Memoize, Memo, Multiple, Optional, Outcome, Panic, Predicate, Recover, Repetition,
    Sequence, Skip, Transform,
};
use crate::data::{
    memory::{replace, take},
    Identity, Offset, Scale,
};
use crate::tracker::Peekable;

struct Recall<'a, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub memo: Memo<'a, Source, Input, Output, Failure>,
}

impl<'a, Source, Input, Output, Failure> Recall<'a, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn new(memo: Memo<'a, Source, Input, Output, Failure>) -> Self {
        Self { memo }
    }
}

impl<'a, Source, Input, Output, Failure> Recall<'a, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn apply<'source>(
        &self,
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let delta = (
            former.forms.len() as isize - self.memo.form_base as isize,
            former.consumed.len() as isize - self.memo.input_base as isize,
        );

        former.forms.extend(self.memo.forms.iter().cloned());
        former.consumed.extend(self.memo.inputs.iter().cloned());

        formation.consumed.extend(
            self.memo
                .consumed
                .iter()
                .map(|&id| (id as isize + delta.1) as Identity),
        );

        formation.stack.extend(self.memo.stack.iter().map(|&id| {
            if id == 0 {
                0
            } else {
                (id as isize + delta.0) as Identity
            }
        }));

        formation.form = if self.memo.form == 0 {
            0
        } else {
            (self.memo.form as isize + delta.0) as Identity
        };

        formation.marker += self.memo.advance;
        formation.state = self.memo.state;
        formation.outcome = self.memo.outcome;
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Multiple<'a, 'source, Joint<'a, 'source, Source, Input, Output, Failure>>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        for combinator in self.combinators.iter() {
            combinator.combinator(joint);
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Literal<'a, 'source, Input>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        match joint.0.source.get(joint.1.marker) {
            Some(input) if self.value.eq(input) => {
                joint.1.set_align();
                joint.0.push(&mut joint.1, input.clone());
            }
            _ => joint.1.set_empty(),
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Predicate<'a, 'source, Input>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        match joint.0.source.get(joint.1.marker) {
            Some(input) if (self.function)(input) => {
                joint.1.set_align();
                joint.0.push(&mut joint.1, input.clone());
            }
            _ => joint.1.set_empty(),
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure, const SIZE: Scale>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Alternative<Formation<'a, 'source, Source, Input, Output, Failure>, SIZE>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let (former, formation) = (&mut joint.0, &mut joint.1);

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
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Deferred<Formation<'a, 'source, Source, Input, Output, Failure>>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let id = self.factory as usize;
        let combinator = match joint.0.stash.iter().find(|(item, _)| *item == id) {
            Some((_, combo)) => combo.clone(),
            None => {
                let state = (self.factory)();
                let combo = state.combinator.clone();
                joint.0.stash.push((id, combo.clone()));
                combo
            }
        };
        combinator.combinator(joint);
    }
}

impl<'a, 'source, Source, Input, Output, Failure, C>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Memoize<C>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
    C: Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>,
{
    #[inline]
    fn combinator(
        &self,
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let id = self as *const Self as usize;
        let key = (id, joint.1.marker);

        if let Some(memo) = joint.0.memo.get(&key).cloned() {
            Recall::new(memo).apply(&mut joint.0, &mut joint.1);
            return;
        }

        let base_forms = joint.0.forms.len() as Offset;
        let base_consumed = joint.0.consumed.len() as Offset;
        let base_self_consumed = joint.1.consumed.len();
        let base_self_stack = joint.1.stack.len();
        let base_marker = joint.1.marker;

        self.inner.combinator(joint);

        let forms = joint.0.forms[base_forms as usize..].to_vec().into_boxed_slice();
        let inputs = joint.0.consumed[base_consumed as usize..]
            .to_vec()
            .into_boxed_slice();
        let self_consumed = joint.1.consumed[base_self_consumed..]
            .to_vec()
            .into_boxed_slice();
        let self_stack = joint.1.stack[base_self_stack..]
            .to_vec()
            .into_boxed_slice();

        let memo = Memo {
            outcome: joint.1.outcome,
            advance: joint.1.marker - base_marker,
            state: joint.1.state,
            forms,
            inputs,
            consumed: self_consumed,
            stack: self_stack,
            form: joint.1.form,
            form_base: base_forms,
            input_base: base_consumed,
        };

        joint.0.memo.insert(key, memo);
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Optional<Formation<'a, 'source, Source, Input, Output, Failure>>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let (former, formation) = (&mut joint.0, &mut joint.1);

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
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Sequence<Formation<'a, 'source, Source, Input, Output, Failure>, SIZE>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let (former, formation) = (&mut joint.0, &mut joint.1);

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
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Repetition<Formation<'a, 'source, Source, Input, Output, Failure>>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let (former, formation) = (&mut joint.0, &mut joint.1);

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
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Recover<
'source,
Joint<'a, 'source, Source, Input, Output, Failure>,
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
joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
) {
if !joint.1.is_failed() && !joint.1.is_panicked() {
return;
}

let failure = (self.emitter)(joint);
let form_id = joint.0.forms.len();
joint.0.forms.push(Form::Failure(failure));

let mut moved = false;

while let Some(input) = joint.0.source.get(joint.1.marker) {
if (self.sync)(input) {
break;
}
let input = input.clone();
joint.0.push(&mut joint.1, input);
moved = true;
}

if !moved {
if let Some(input) = joint.0.source.get(joint.1.marker) {
let input = input.clone();
joint.0.push(&mut joint.1, input);
}
}

joint.1.set_align();
joint.1.form = form_id;
}
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Ignore
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if joint.1.is_aligned() {
            joint.1.set_ignore();
            joint.1.form = 0;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Skip
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if joint.1.is_aligned() {
            joint.1.set_empty();
            joint.1.form = 0;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Transform<'source, Joint<'a, 'source, Source, Input, Output, Failure>, Failure>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if joint.1.is_aligned() {
            if let Err(error) = (self.transformer)(joint) {
                let form_id = joint.0.forms.len();
                joint.0.forms.push(Form::Failure(error));
                joint.1.set_fail();
                joint.1.form = form_id;
            }
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Fail<'source, Joint<'a, 'source, Source, Input, Output, Failure>, Failure>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !joint.1.is_aligned() {
            let failure = (self.emitter)(joint);
            let form_id = joint.0.forms.len();
            joint.0.forms.push(Form::Failure(failure));
            joint.1.set_fail();
            joint.1.form = form_id;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
for Panic<'source, Joint<'a, 'source, Source, Input, Output, Failure>, Failure>
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
        joint: &mut Joint<'a, 'source, Source, Input, Output, Failure>,
    ) {
        if !joint.1.is_aligned() {
            let failure = (self.emitter)(joint);
            let form_id = joint.0.forms.len();
            joint.0.forms.push(Form::Failure(failure));
            joint.1.set_panic();
            joint.1.form = form_id;
        }
    }
}