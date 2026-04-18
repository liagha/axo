use crate::{
    combinator::{
        formation::former::{outcome::Outcome, Former, Memo, Record},
        next_identity, Action, Alternative, Deferred, Fail, Form, Formable, Ignore, Literal,
        Multiple, Optional, Panic, Predicate, Recover, Repetition, Sequence, Skip, Transform,
    },
    data::{
        memory::{replace, take, Arc},
        Identity, Offset, Scale,
    },
    tracker::Peekable,
};

pub struct Formation<'a: 'source, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub identity: Identity,
    pub action:
        Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>,
    pub marker: Offset,
    pub state: Source::State,
    pub consumed: Vec<Identity>,
    pub outcome: Outcome,
    pub form: Identity,
    pub stack: Vec<Identity>,
    pub depth: Scale,
}

impl<'a: 'source, 'source, Source, Input, Output, Failure>
Formation<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn new(
        action: Arc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source,
        >,
        marker: Offset,
        state: Source::State,
    ) -> Self {
        Self {
            identity: next_identity(),
            action,
            marker,
            state,
            consumed: Vec::new(),
            outcome: Outcome::Blank,
            form: 0,
            stack: Vec::new(),
            depth: 0,
        }
    }

    #[inline]
    fn create(
        action: Arc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source,
        >,
        marker: Offset,
        state: Source::State,
        consumed: Vec<Identity>,
        outcome: Outcome,
        form: Identity,
        stack: Vec<Identity>,
        depth: Scale,
    ) -> Self {
        Self {
            identity: next_identity(),
            action,
            marker,
            state,
            consumed,
            outcome,
            form,
            stack,
            depth,
        }
    }

    #[inline]
    fn create_child(
        &mut self,
        action: Arc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source,
        >,
    ) -> Self {
        Self {
            identity: next_identity(),
            action,
            marker: self.marker,
            state: self.state,
            consumed: take(&mut self.consumed),
            outcome: Outcome::Blank,
            form: 0,
            stack: take(&mut self.stack),
            depth: self.depth + 1,
        }
    }

    #[inline]
    pub const fn is_panicked(&self) -> bool {
        matches!(self.outcome, Outcome::Panicked)
    }

    #[inline]
    pub const fn is_aligned(&self) -> bool {
        matches!(self.outcome, Outcome::Aligned)
    }

    #[inline]
    pub const fn is_failed(&self) -> bool {
        matches!(self.outcome, Outcome::Failed)
    }

    #[inline]
    pub const fn is_effected(&self) -> bool {
        matches!(self.outcome, Outcome::Aligned | Outcome::Failed)
    }

    #[inline]
    pub const fn is_blank(&self) -> bool {
        matches!(self.outcome, Outcome::Blank)
    }

    #[inline]
    pub const fn is_ignored(&self) -> bool {
        matches!(self.outcome, Outcome::Ignored)
    }

    #[inline]
    pub const fn is_terminal(&self) -> bool {
        self.outcome.is_terminal()
    }

    #[inline]
    pub const fn is_neutral(&self) -> bool {
        self.outcome.is_neutral()
    }

    #[inline]
    pub fn set_panic(&mut self) {
        self.outcome = Outcome::Panicked;
    }

    #[inline]
    pub fn set_align(&mut self) {
        self.outcome = Outcome::Aligned;
    }

    #[inline]
    pub fn set_fail(&mut self) {
        self.outcome = Outcome::Failed;
    }

    #[inline]
    pub fn set_empty(&mut self) {
        self.outcome = Outcome::Blank;
    }

    #[inline]
    pub fn set_ignore(&mut self) {
        self.outcome = Outcome::Ignored;
    }

    #[inline]
    pub fn escalate(&mut self, other: Outcome) {
        self.outcome = self.outcome.escalate(other);
    }

    #[inline]
    pub fn literal(value: impl PartialEq<Input> + Send + Sync + 'source + 'a) -> Self {
        Self::new(
            Arc::new(Literal {
                value: Arc::new(value),
                phantom: Default::default(),
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + Send + Sync + 'source + 'a,
    {
        Self::new(
            Arc::new(Predicate::<Input> {
                function: Arc::new(predicate),
                phantom: Default::default(),
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::alternative_with(
            patterns,
            |state| state.is_aligned() || state.is_panicked(),
            |new, old| new.is_aligned() && (old.is_failed() || new.marker > old.marker),
        )
    }

    #[inline]
    pub fn alternative_with<const SIZE: Scale>(
        patterns: [Self; SIZE],
        halt: fn(&Self) -> bool,
        compare: fn(&Self, &Self) -> bool,
    ) -> Self {
        Self::new(
            Arc::new(Alternative {
                states: patterns,
                halt,
                compare,
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Arc::new(Sequence {
                states: patterns,
                halt: |state| !(state.is_aligned() || state.is_ignored()),
                keep: |state| state.is_aligned(),
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn optional(formation: Self) -> Self {
        Self::new(
            Arc::new(Optional {
                state: Box::new(formation),
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn persistence(formation: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Arc::new(Repetition {
                state: Box::new(formation),
                minimum,
                maximum,
                halt: |state| state.is_blank(),
                keep: |state| state.is_effected() || state.is_panicked(),
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn repetition(formation: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Arc::new(Repetition {
                state: Box::new(formation),
                minimum,
                maximum,
                halt: |state| state.is_failed() || state.is_panicked() || state.is_blank(),
                keep: |state| state.is_aligned() || state.is_failed() || state.is_panicked(),
            }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn deferred(factory: fn() -> Self) -> Self {
        Self::new(
            Arc::new(Deferred { factory }),
            0,
            Default::default(),
        )
    }

    #[inline]
    pub fn anything() -> Self {
        Self::predicate(|_| true)
    }

    #[inline]
    pub fn nothing() -> Self {
        Self::predicate(|_| false)
    }

    #[inline]
    pub fn with_action(
        mut self,
        action: Arc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source,
        >,
    ) -> Self {
        let actions = vec![self.action.clone(), action];
        self.action = Arc::new(Multiple { actions });
        self
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        self.with_action(Arc::new(Fail {
            emitter: Arc::new(emitter),
            phantom: Default::default(),
        }))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.with_action(Arc::new(Ignore))
    }

    #[inline]
    pub fn with_multiple(
        self,
        actions: Vec<
            Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>,
        >,
    ) -> Self {
        self.with_action(Arc::new(Multiple { actions }))
    }

    #[inline]
    pub fn with_panic<F>(self, emitter: F) -> Self
    where
        F: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        self.with_action(Self::panic(emitter))
    }

    #[inline]
    pub fn with_recover<S, F>(self, sync: S, emitter: F) -> Self
    where
        S: Fn(&Input) -> bool + Send + Sync + 'source,
        F: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        self.with_action(Self::recover(sync, emitter))
    }

    #[inline]
    pub fn with_skip(self) -> Self {
        self.with_action(Arc::new(Skip))
    }

    #[inline]
    pub fn with_transform<T>(self, transform: T) -> Self
    where
        T: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            &mut Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + Send
        + Sync
        + 'source,
    {
        self.with_action(Self::transform(transform))
    }

    #[inline]
    pub fn into_optional(self) -> Self {
        Self::optional(self)
    }

    #[inline]
    pub fn into_persistence(self, min: Scale, max: Option<Scale>) -> Self {
        Self::persistence(self, min, max)
    }

    #[inline]
    pub fn transform<T>(
        transformer: T,
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    where
        T: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            &mut Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + Send
        + Sync
        + 'source,
    {
        Arc::new(Transform {
            transformer: Arc::new(transformer),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn fail<T>(
        emitter: T,
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    where
        T: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        Arc::new(Fail {
            emitter: Arc::new(emitter),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn panic<T>(
        emitter: T,
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    where
        T: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        Arc::new(Panic {
            emitter: Arc::new(emitter),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn recover<S, E>(
        sync: S,
        emitter: E,
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    where
        S: Fn(&Input) -> bool + Send + Sync + 'source,
        E: Fn(
            &mut Former<'a, 'source, Source, Input, Output, Failure>,
            Formation<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        Arc::new(Recover {
            sync: Arc::new(sync),
            emitter: Arc::new(emitter),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn ignore() -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    {
        Arc::new(Ignore)
    }

    #[inline]
    pub fn multiple(
        actions: Vec<
            Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>,
        >,
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    {
        Arc::new(Multiple { actions })
    }

    #[inline]
    pub fn skip() -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    {
        Arc::new(Skip)
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Literal<'a, 'source, Input>
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
        if let Some(peek) = former.source.get(formation.marker) {
            if self.value.eq(peek) {
                formation.set_align();
                former
                    .source
.next(&mut formation.marker, &mut formation.state);

                let identity_c = former.consumed.len();
                let identity_f = former.forms.len();

                former.consumed.push(peek.clone());
                former.forms.push(Form::input(peek.clone()));

                formation.consumed.push(identity_c);
                formation.form = identity_f;
                formation.stack.push(identity_f);
            } else {
                formation.set_empty();
            }
        } else {
            formation.set_empty();
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Predicate<'a, 'source, Input>
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
        if let Some(peek) = former.source.get(formation.marker) {
            if (self.function)(peek) {
                formation.set_align();
                former
                    .source
.next(&mut formation.marker, &mut formation.state);

                let identity_c = former.consumed.len();
                let identity_f = former.forms.len();

                former.consumed.push(peek.clone());
                former.forms.push(Form::input(peek.clone()));

                formation.consumed.push(identity_c);
                formation.form = identity_f;
                formation.stack.push(identity_f);
            } else {
                formation.set_empty();
            }
        } else {
            formation.set_empty();
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure, const SIZE: Scale>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Alternative<Formation<'a, 'source, Source, Input, Output, Failure>, SIZE>
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
        let mut best: Option<Formation<'a, 'source, Source, Input, Output, Failure>> = None;
        let mut snapshot = (former.consumed.len(), former.forms.len());

        let mut consumed = take(&mut formation.consumed);
        let mut stack = take(&mut formation.stack);
        let base = (consumed.len(), stack.len());

        for (idx, pattern) in self.states.iter().enumerate() {
            let mut child = Formation::create(
                pattern.action.clone(),
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
                former.consumed.truncate(snapshot.0);
                former.forms.truncate(snapshot.1);
                continue;
            }

            let better = match &best {
                Some(champion) => (self.compare)(&child, champion),
                None => true,
            };

            if better {
                let halted = (self.halt)(&child);
                let last = idx == self.states.len() - 1;

                if !last && !halted {
                    if let Some(champion) = best.take() {
                        (consumed, stack) = (champion.consumed, champion.stack);
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

                snapshot = (former.consumed.len(), former.forms.len());
                best = Some(child);

                if halted {
                    break;
                }
            } else {
                (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));
                consumed.truncate(base.0);
                stack.truncate(base.1);
                former.consumed.truncate(snapshot.0);
                former.forms.truncate(snapshot.1);
            }
        }

        match best {
            Some(mut champion) => {
                formation.outcome = champion.outcome;
                formation.marker = champion.marker;
                formation.state = champion.state;
                formation.consumed = take(&mut champion.consumed);
                formation.form = champion.form;
                formation.stack = take(&mut champion.stack);
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
    Source: Peekable<'a, Input>,
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
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Deferred<Formation<'a, 'source, Source, Input, Output, Failure>>
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
        let identifier = self.factory as usize;
        let memory = (identifier, formation.marker);

        if let Some(memo) = former.memo.get(&memory) {
            if let Some(record) = &memo.record {
                let offset = (
                    former.forms.len() as isize - record.form_base as isize,
                    former.consumed.len() as isize - record.input_base as isize,
                );

                former.forms.extend(record.forms.iter().cloned());
                former.consumed.extend(record.inputs.iter().cloned());

                formation.consumed.extend(
                    record
                        .consumed
                        .iter()
                        .map(|&i| (i as isize + offset.1) as Identity),
                );

                formation.stack.extend(record.stack.iter().map(|&i| {
                    if i == 0 {
                        0
                    } else {
                        (i as isize + offset.0) as Identity
                    }
                }));

                formation.form = if record.form == 0 {
                    0
                } else {
                    (record.form as isize + offset.0) as Identity
                };
            } else {
                formation.form = 0;
            }

            formation.marker = formation.marker + memo.advance;
            formation.state = memo.state;
            formation.outcome = memo.outcome;

            return;
        }

        let stashed = match former.stash.iter().find(|(k, _)| *k == identifier) {
            Some((_, action)) => action.clone(),
            None => {
                let built = (self.factory)();
                former.stash.push((identifier, built.action.clone()));
                built.action
            }
        };

        let consumed = take(&mut formation.consumed);
        let stack = take(&mut formation.stack);

        let origin = (
            consumed.len(),
            stack.len(),
            former.forms.len() as Offset,
            former.consumed.len() as Offset,
            formation.marker,
        );

        let mut child = Formation::create(
            stashed,
            formation.marker,
            formation.state,
            consumed,
            Outcome::Blank,
            0,
            stack,
            formation.depth + 1,
        );

        former.build(&mut child);

        let has_data = !former.forms[origin.2 as usize..].is_empty()
            || !former.consumed[origin.3 as usize..].is_empty()
            || !child.consumed[origin.0..].is_empty()
            || !child.stack[origin.1..].is_empty()
            || child.form != 0;

        let record = if has_data {
            Some(Box::new(Record {
                forms: former.forms[origin.2 as usize..].to_vec().into_boxed_slice(),
                inputs: former.consumed[origin.3 as usize..].to_vec().into_boxed_slice(),
                consumed: child.consumed[origin.0..].to_vec().into_boxed_slice(),
                stack: child.stack[origin.1..].to_vec().into_boxed_slice(),
                form: child.form,
                form_base: origin.2,
                input_base: origin.3,
            }))
        } else {
            None
        };

        if former.memo.len() > 2048 {
            former.memo.clear();
        }

        former.memo.insert(
            memory,
            Memo {
                outcome: child.outcome,
                advance: child.marker - origin.4,
                state: child.state,
                record,
            },
        );

        (formation.marker, formation.state, formation.outcome, formation.form) = (child.marker, child.state, child.outcome, child.form);

        (formation.consumed, formation.stack) = (child.consumed, child.stack);
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Optional<Formation<'a, 'source, Source, Input, Output, Failure>>
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
        let base = (
            former.consumed.len(),
            former.forms.len(),
            formation.consumed.len(),
            formation.stack.len(),
        );

        let mut child = formation.create_child(self.state.action.clone());
        former.build(&mut child);

        let panicked = child.is_panicked();
        let aligned = child.is_aligned();

        formation.consumed = child.consumed;
        formation.stack = child.stack;

        if panicked {
            (formation.marker, formation.state, formation.form) =
                (child.marker, child.state, child.form);
            formation.set_panic();
        } else if aligned {
            (formation.marker, formation.state, formation.form) = (child.marker, child.state, child.form);
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
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Sequence<Formation<'a, 'source, Source, Input, Output, Failure>, SIZE>
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
        let mut consumed = take(&mut formation.consumed);
        let mut stack = take(&mut formation.stack);

        let origin = (
            formation.marker,
            formation.state,
            former.consumed.len(),
            former.forms.len(),
            consumed.len(),
            stack.len(),
        );

        let mut forms = Vec::with_capacity(SIZE);
        let mut broke = false;

        for pattern in &self.states {
            let mut child = Formation::create(
                pattern.action.clone(),
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
            let preserved = if formation.is_failed() || formation.is_panicked() {
                former.forms.get(formation.form).cloned()
            } else {
                None
            };

            formation.marker = origin.0;
            formation.state = origin.1;
            former.consumed.truncate(origin.2);
            former.forms.truncate(origin.3);
            formation.consumed.truncate(origin.4);
            formation.stack.truncate(origin.5);

            if let Some(failure) = preserved {
                let identity = former.forms.len();
                former.forms.push(failure);
                formation.form = identity;
            }
        } else {
            formation.set_align();

            let group = Form::multiple(
                forms
                    .into_iter()
                    .map(|identity| replace(&mut former.forms[identity], Form::Blank))
                    .collect(),
            );

            let identity = former.forms.len();
            former.forms.push(group);
            formation.form = identity;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> for Repetition<Formation<'a, 'source, Source, Input, Output, Failure>>
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
        let mut consumed = take(&mut formation.consumed);
        let mut stack = take(&mut formation.stack);

        let origin = (
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
                self.state.action.clone(),
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
                (formation.outcome, formation.marker, formation.state) = (child.outcome, child.marker, child.state);

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
                (formation.outcome, formation.marker, formation.state) = (child.outcome, child.marker, child.state);

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

            let group = Form::multiple(
                forms
                    .into_iter()
                    .map(|identity| replace(&mut former.forms[identity], Form::Blank))
                    .collect(),
            );

            let identity = former.forms.len();
            former.forms.push(group);
            formation.form = identity;
        } else {
            formation.marker = origin.0;
            formation.state = origin.1;

            former.consumed.truncate(origin.2);
            former.forms.truncate(origin.3);
            formation.consumed.truncate(origin.4);
            formation.stack.truncate(origin.5);

            if !formation.is_failed() && !formation.is_panicked() {
                formation.set_empty();
            }
        }
    }
}
