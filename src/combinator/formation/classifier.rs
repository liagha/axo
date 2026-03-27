use crate::{
    combinator::{
        formation::former::{outcome::Outcome, Former, Memo},
        next_identity, Action, Alternative, Deferred, Fail, Form, Formable, Ignore, Literal,
        Multiple, Optional, Panic, Predicate, Repetition, Sequence, Skip, Transform,
    },
    data::{
        memory::{replace, take, Arc},
        Identity, Offset, Scale,
    },
    tracker::{Location, Peekable, Position},
};

pub struct Classifier<'a: 'source, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub identity: Identity,
    pub action:
        Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>,
    pub marker: Offset,
    pub position: Position<'a>,
    pub consumed: Vec<Identity>,
    pub outcome: Outcome,
    pub form: Identity,
    pub stack: Vec<Identity>,
    pub depth: Scale,
}

impl<'a: 'source, 'source, Source, Input, Output, Failure>
Classifier<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
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
        position: Position<'a>,
    ) -> Self {
        Self {
            identity: next_identity(),
            action,
            marker,
            position,
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
        position: Position<'a>,
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
            position,
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
            position: self.position,
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
            Position::new(Location::Void),
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
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Arc::new(Alternative {
                states: patterns,
                halt: |state| state.is_aligned() || state.is_panicked(),
                compare: |new, old| new.is_aligned() && (old.is_failed() || new.marker > old.marker),
            }),
            0,
            Position::new(Location::Void),
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
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn optional(classifier: Self) -> Self {
        Self::new(
            Arc::new(Optional {
                state: Box::new(classifier),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn persistence(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Arc::new(Repetition {
                state: Box::new(classifier),
                minimum,
                maximum,
                halt: |state| state.is_blank(),
                keep: |state| state.is_effected() || state.is_panicked(),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn repetition(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Arc::new(Repetition {
                state: Box::new(classifier),
                minimum,
                maximum,
                halt: |state| state.is_failed() || state.is_panicked() || state.is_blank(),
                keep: |state| state.is_aligned() || state.is_failed() || state.is_panicked(),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn deferred(factory: fn() -> Self) -> Self {
        Self::new(
            Arc::new(Deferred { factory }),
            0,
            Position::new(Location::Void),
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
            Classifier<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send + Sync + 'source,
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
            Classifier<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send
        + Sync
        + 'source,
    {
        self.with_action(Self::panic(emitter))
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
            &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + Send + Sync + 'source,
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
            &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + Send + Sync + 'source,
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
            Classifier<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send + Sync + 'source,
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
            Classifier<'a, 'source, Source, Input, Output, Failure>,
        ) -> Failure
        + Send + Sync + 'source,
    {
        Arc::new(Panic {
            emitter: Arc::new(emitter),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn ignore(
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
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
    pub fn skip(
    ) -> Arc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + Send + Sync + 'source>
    {
        Arc::new(Skip)
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Literal<'a, 'source, Input>
where
    Source: Peekable<'a, Input>,
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
        if let Some(peek) = former.source.get(classifier.marker) {
            if self.value.eq(peek) {
                classifier.set_align();
                former
                    .source
                    .next(&mut classifier.marker, &mut classifier.position);

                let val = peek.clone();
                let input_id = former.consumed.len();
                former.consumed.push(val.clone());
                classifier.consumed.push(input_id);

                let form_id = former.forms.len();
                former.forms.push(Form::input(val));
                classifier.form = form_id;
                classifier.stack.push(form_id);
            } else {
                classifier.set_empty();
            }
        } else {
            classifier.set_empty();
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Predicate<'a, 'source, Input>
where
    Source: Peekable<'a, Input>,
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
        if let Some(peek) = former.source.get(classifier.marker) {
            if (self.function)(peek) {
                let val = peek.clone();
                classifier.set_align();
                former
                    .source
                    .next(&mut classifier.marker, &mut classifier.position);

                let input_id = former.consumed.len();
                former.consumed.push(val.clone());
                classifier.consumed.push(input_id);

                let form_id = former.forms.len();
                former.forms.push(Form::input(val));
                classifier.form = form_id;
                classifier.stack.push(form_id);
            } else {
                classifier.set_empty();
            }
        } else {
            classifier.set_empty();
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure, const SIZE: Scale>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Alternative<Classifier<'a, 'source, Source, Input, Output, Failure>, SIZE>
where
    Source: Peekable<'a, Input>,
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
        let mut best: Option<Classifier<'a, 'source, Source, Input, Output, Failure>> = None;
        let mut best_input = former.consumed.len();
        let mut best_form = former.forms.len();

        let mut current_consumed = take(&mut classifier.consumed);
        let mut current_stack = take(&mut classifier.stack);
        let base_consumed = current_consumed.len();
        let base_stack = current_stack.len();

        for pattern in &self.states {
            let mut child = Classifier::create(
                pattern.action.clone(),
                classifier.marker,
                classifier.position,
                current_consumed,
                Outcome::Blank,
                0,
                current_stack,
                classifier.depth + 1,
            );

            former.build(&mut child);

            if child.is_blank() {
                current_consumed = child.consumed;
                current_stack = child.stack;
                current_consumed.truncate(base_consumed);
                current_stack.truncate(base_stack);
                former.consumed.truncate(best_input);
                former.forms.truncate(best_form);
                continue;
            }

            let better = match &best {
                Some(champion) => (self.compare)(&child, champion),
                None => true,
            };

            if better {
                if let Some(champion) = best.take() {
                    current_consumed = champion.consumed;
                    current_stack = champion.stack;
                    current_consumed.truncate(base_consumed);
                    current_stack.truncate(base_stack);
                } else {
                    current_consumed = child.consumed[..base_consumed].to_vec();
                    current_stack = child.stack[..base_stack].to_vec();
                }

                best_input = former.consumed.len();
                best_form = former.forms.len();
                best = Some(child);
            } else {
                current_consumed = child.consumed;
                current_stack = child.stack;
                current_consumed.truncate(base_consumed);
                current_stack.truncate(base_stack);
                former.consumed.truncate(best_input);
                former.forms.truncate(best_form);
            }

            if let Some(ref champion) = best {
                if (self.halt)(champion) {
                    break;
                }
            }
        }

        match best {
            Some(mut champion) => {
                classifier.outcome = champion.outcome;
                classifier.marker = champion.marker;
                classifier.position = champion.position;
                classifier.consumed = take(&mut champion.consumed);
                classifier.form = champion.form;
                classifier.stack = take(&mut champion.stack);
            }
            None => {
                classifier.set_empty();
                classifier.consumed = current_consumed;
                classifier.stack = current_stack;
            }
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure> Clone
for Deferred<Classifier<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input>,
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
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Deferred<Classifier<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input>,
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
        let key = self.factory as usize;
        let memo_key = (key, classifier.marker);

        if let Some(entry) = former.memo.get(&memo_key) {
            let form_offset = former.forms.len() as isize - entry.form_base as isize;
            let input_offset = former.consumed.len() as isize - entry.input_base as isize;

            former.forms.extend(entry.forms.iter().cloned());
            former.consumed.extend(entry.inputs.iter().cloned());

            classifier.consumed.extend(
                entry.consumed.iter().map(|&i| (i as isize + input_offset) as Identity)
            );

            classifier.stack.extend(
                entry.stack.iter().map(|&i| {
                    if i == 0 {
                        0
                    } else {
                        (i as isize + form_offset) as Identity
                    }
                })
            );

            (classifier.marker, classifier.position, classifier.outcome, classifier.form) = (
                classifier.marker + entry.advance,
                entry.position,
                entry.outcome,
                if entry.form == 0 { 0 } else { (entry.form as isize + form_offset) as Identity },
            );

            return;
        }

        let stashed = match former.stash.iter().find(|(k, _)| *k == key) {
            Some((_, action)) => action.clone(),
            None => {
                let built = (self.factory)();
                former.stash.push((key, built.action.clone()));
                built.action
            }
        };

        let consumed = take(&mut classifier.consumed);
        let stack = take(&mut classifier.stack);

        let base = (
            consumed.len(),
            stack.len(),
            former.forms.len(),
            former.consumed.len(),
            classifier.marker,
        );

        let mut child = Classifier::create(
            stashed,
            classifier.marker,
            classifier.position,
            consumed,
            Outcome::Blank,
            0,
            stack,
            classifier.depth + 1,
        );

        former.build(&mut child);

        former.memo.insert(
            memo_key,
            Memo {
                outcome: child.outcome,
                advance: child.marker - base.4,
                position: child.position,
                forms: former.forms[base.2..].to_vec(),
                inputs: former.consumed[base.3..].to_vec(),
                consumed: child.consumed[base.0..].to_vec(),
                stack: child.stack[base.1..].to_vec(),
                form: child.form,
                form_base: base.2,
                input_base: base.3,
            },
        );

        (classifier.marker, classifier.position, classifier.outcome, classifier.form) =
            (child.marker, child.position, child.outcome, child.form);

        (classifier.consumed, classifier.stack) = (child.consumed, child.stack);
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Optional<Classifier<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input>,
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
        let base = (
            former.consumed.len(),
            former.forms.len(),
            classifier.consumed.len(),
            classifier.stack.len(),
        );

        let mut child = classifier.create_child(self.state.action.clone());
        former.build(&mut child);

        let effected = child.is_effected();

        classifier.consumed = child.consumed;
        classifier.stack = child.stack;

        if effected {
            (classifier.marker, classifier.position, classifier.form) =
                (child.marker, child.position, child.form);
            classifier.set_align();
        } else {
            former.consumed.truncate(base.0);
            former.forms.truncate(base.1);
            classifier.consumed.truncate(base.2);
            classifier.stack.truncate(base.3);
            classifier.set_ignore();
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure, const SIZE: Scale>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Sequence<Classifier<'a, 'source, Source, Input, Output, Failure>, SIZE>
where
    Source: Peekable<'a, Input>,
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
        let mut consumed = take(&mut classifier.consumed);
        let mut stack = take(&mut classifier.stack);

        let origin = (
            classifier.marker,
            classifier.position,
            former.consumed.len(),
            former.forms.len(),
            consumed.len(),
            stack.len(),
        );

        let mut forms = Vec::with_capacity(SIZE);
        let mut broke = false;

        for pattern in &self.states {
            let mut child = Classifier::create(
                pattern.action.clone(),
                classifier.marker,
                classifier.position,
                consumed,
                Outcome::Blank,
                0,
                stack,
                classifier.depth + 1,
            );

            former.build(&mut child);

            let halted = (self.halt)(&child);
            let kept = (self.keep)(&child);

            (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));

            if halted {
                (classifier.outcome, classifier.marker, classifier.position) =
                    (child.outcome, child.marker, child.position);

                if child.is_panicked() || child.is_failed() {
                    forms.push(child.form);
                }

                broke = true;
                break;
            }

            if kept {
                classifier.outcome = child.outcome;
                forms.push(child.form);
            }

            (classifier.marker, classifier.position) = (child.marker, child.position);
        }

        classifier.consumed = consumed;
        classifier.stack = stack;

        if broke {
            (classifier.marker, classifier.position) = (origin.0, origin.1);

            former.consumed.truncate(origin.2);
            former.forms.truncate(origin.3);
            classifier.consumed.truncate(origin.4);
            classifier.stack.truncate(origin.5);
        } else {
            let group = Form::multiple(
                forms
                    .into_iter()
                    .map(|id| replace(&mut former.forms[id], Form::Blank))
                    .collect(),
            );

            let form_id = former.forms.len();
            former.forms.push(group);
            classifier.form = form_id;
        }
    }
}

impl<'a, 'source, Source, Input, Output, Failure>
Action<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Classifier<'a, 'source, Source, Input, Output, Failure>,
> for Repetition<Classifier<'a, 'source, Source, Input, Output, Failure>>
where
    Source: Peekable<'a, Input>,
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
        let mut consumed = take(&mut classifier.consumed);
        let mut stack = take(&mut classifier.stack);

        let origin = (
            classifier.marker,
            classifier.position,
            former.consumed.len(),
            former.forms.len(),
            consumed.len(),
            stack.len(),
        );

        let mut forms = Vec::new();

        while former.source.peek_ahead(classifier.marker).is_some() {
            let step = (
                former.consumed.len(),
                former.forms.len(),
                consumed.len(),
                stack.len(),
            );

            let mut child = Classifier::create(
                self.state.action.clone(),
                classifier.marker,
                classifier.position,
                consumed,
                Outcome::Blank,
                0,
                stack,
                classifier.depth + 1,
            );

            former.build(&mut child);

            if child.marker == classifier.marker {
                (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));

                former.consumed.truncate(step.0);
                former.forms.truncate(step.1);
                consumed.truncate(step.2);
                stack.truncate(step.3);
                break;
            }

            let halted = (self.halt)(&child);
            let kept = (self.keep)(&child);

            (consumed, stack) = (take(&mut child.consumed), take(&mut child.stack));

            if halted {
                (classifier.outcome, classifier.marker, classifier.position) =
                    (child.outcome, child.marker, child.position);

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
                (classifier.outcome, classifier.marker, classifier.position) =
                    (child.outcome, child.marker, child.position);

                forms.push(child.form);
            } else {
                former.consumed.truncate(step.0);
                former.forms.truncate(step.1);
                consumed.truncate(step.2);
                stack.truncate(step.3);

                (classifier.marker, classifier.position) = (child.marker, child.position);
            }

            if let Some(max) = self.maximum {
                if forms.len() >= max as Identity {
                    break;
                }
            }
        }

        classifier.consumed = consumed;
        classifier.stack = stack;

        if forms.len() >= self.minimum as Identity {
            if !classifier.is_failed() && !classifier.is_panicked() {
                classifier.set_align();
            }

            let group = Form::multiple(
                forms
                    .into_iter()
                    .map(|id| replace(&mut former.forms[id], Form::Blank))
                    .collect(),
            );

            let form_id = former.forms.len();
            former.forms.push(group);
            classifier.form = form_id;
        } else {
            (classifier.marker, classifier.position) = (origin.0, origin.1);

            former.consumed.truncate(origin.2);
            former.forms.truncate(origin.3);
            classifier.consumed.truncate(origin.4);
            classifier.stack.truncate(origin.5);

            if !classifier.is_failed() && !classifier.is_panicked() {
                classifier.set_empty();
            }
        }
    }
}
