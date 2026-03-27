use crate::{
    combinator::{
        formation::former::{outcome::Outcome, Former, Memo},
        next_identity, Action, Alternative, Deferred, Fail, Form, Formable, Ignore, Literal,
        Multiple, Optional, Panic, Predicate, Repetition, Sequence, Skip, Transform,
    },
    data::{
        memory::{replace, swap, take, Rc},
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
        Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>,
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
        action: Rc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source,
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
        action: Rc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source,
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
        action: Rc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source,
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
    pub fn literal(value: impl PartialEq<Input> + 'source + 'a) -> Self {
        Self::new(
            Rc::new(Literal {
                value: Rc::new(value),
                phantom: Default::default(),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + 'source + 'a,
    {
        Self::new(
            Rc::new(Predicate::<Input> {
                function: Rc::new(predicate),
                phantom: Default::default(),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Rc::new(Alternative { states: patterns }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Rc::new(Sequence { states: patterns }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn optional(classifier: Self) -> Self {
        Self::new(
            Rc::new(Optional {
                state: Box::new(classifier),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn persistence(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Rc::new(Repetition {
                state: Box::new(classifier),
                minimum,
                maximum,
                persist: true,
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn repetition(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Rc::new(Repetition {
                state: Box::new(classifier),
                minimum,
                maximum,
                persist: false,
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn deferred(factory: fn() -> Self) -> Self {
        Self::new(
            Rc::new(Deferred { factory }),
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
        action: Rc<
            dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source,
        >,
    ) -> Self {
        let actions = vec![self.action.clone(), action];
        self.action = Rc::new(Multiple { actions });
        self
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(
                &mut Former<'a, 'source, Source, Input, Output, Failure>,
                Classifier<'a, 'source, Source, Input, Output, Failure>,
            ) -> Failure
            + 'source,
    {
        self.with_action(Rc::new(Fail {
            emitter: Rc::new(emitter),
            phantom: Default::default(),
        }))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.with_action(Rc::new(Ignore))
    }

    #[inline]
    pub fn with_multiple(
        self,
        actions: Vec<
            Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>,
        >,
    ) -> Self {
        self.with_action(Rc::new(Multiple { actions }))
    }

    #[inline]
    pub fn with_panic<F>(self, emitter: F) -> Self
    where
        F: Fn(
                &mut Former<'a, 'source, Source, Input, Output, Failure>,
                Classifier<'a, 'source, Source, Input, Output, Failure>,
            ) -> Failure
            + 'source,
    {
        self.with_action(Self::panic(emitter))
    }

    #[inline]
    pub fn with_skip(self) -> Self {
        self.with_action(Rc::new(Skip))
    }

    #[inline]
    pub fn with_transform<T>(self, transform: T) -> Self
    where
        T: Fn(
                &mut Former<'a, 'source, Source, Input, Output, Failure>,
                &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
            ) -> Result<(), Failure>
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
    ) -> Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>
    where
        T: Fn(
                &mut Former<'a, 'source, Source, Input, Output, Failure>,
                &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
            ) -> Result<(), Failure>
            + 'source,
    {
        Rc::new(Transform {
            transformer: Rc::new(transformer),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn fail<T>(
        emitter: T,
    ) -> Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>
    where
        T: Fn(
                &mut Former<'a, 'source, Source, Input, Output, Failure>,
                Classifier<'a, 'source, Source, Input, Output, Failure>,
            ) -> Failure
            + 'source,
    {
        Rc::new(Fail {
            emitter: Rc::new(emitter),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn panic<T>(
        emitter: T,
    ) -> Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>
    where
        T: Fn(
                &mut Former<'a, 'source, Source, Input, Output, Failure>,
                Classifier<'a, 'source, Source, Input, Output, Failure>,
            ) -> Failure
            + 'source,
    {
        Rc::new(Panic {
            emitter: Rc::new(emitter),
            phantom: Default::default(),
        })
    }

    #[inline]
    pub fn ignore(
    ) -> Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>
    {
        Rc::new(Ignore)
    }

    #[inline]
    pub fn multiple(
        actions: Vec<
            Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>,
        >,
    ) -> Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>
    {
        Rc::new(Multiple { actions })
    }

    #[inline]
    pub fn skip(
    ) -> Rc<dyn Action<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self> + 'source>
    {
        Rc::new(Skip)
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
                let use_id = former.consumed.len();
                former.consumed.push(val.clone());
                classifier.consumed.push(use_id);

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

                let use_id = former.consumed.len();
                former.consumed.push(val.clone());
                classifier.consumed.push(use_id);

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

        let mut stack = take(&mut classifier.stack);
        let mut consumed = take(&mut classifier.consumed);
        let base_stack = stack.len();
        let base_consumed = consumed.len();

        let mut form_used = former.consumed.len();
        let mut form_forms = former.forms.len();

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

            if matches!(child.outcome, Outcome::Blank) {
                stack = child.stack;
                consumed = child.consumed;
                stack.truncate(base_stack);
                consumed.truncate(base_consumed);
                former.consumed.truncate(form_used);
                former.forms.truncate(form_forms);
                continue;
            }

            if let Some(ref mut champion) = best {
                if child.is_aligned() && (champion.is_failed() || child.marker > champion.marker) {
                    swap(champion, &mut child);
                    stack = child.stack;
                    consumed = child.consumed;
                    stack.truncate(base_stack);
                    consumed.truncate(base_consumed);

                    form_used = former.consumed.len();
                    form_forms = former.forms.len();
                } else {
                    stack = child.stack;
                    consumed = child.consumed;
                    stack.truncate(base_stack);
                    consumed.truncate(base_consumed);

                    former.consumed.truncate(form_used);
                    former.forms.truncate(form_forms);
                }
            } else {
                let mut next_stack = Vec::with_capacity(child.stack.capacity());
                next_stack.extend_from_slice(&child.stack[..base_stack]);
                stack = next_stack;

                let mut next_consumed = Vec::with_capacity(child.consumed.capacity());
                next_consumed.extend_from_slice(&child.consumed[..base_consumed]);
                consumed = next_consumed;

                best = Some(child);

                form_used = former.consumed.len();
                form_forms = former.forms.len();
            }

            if let Some(ref champion) = best {
                if matches!(champion.outcome, Outcome::Panicked | Outcome::Aligned) {
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
                classifier.consumed = consumed;
                classifier.stack = stack;
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
            let form_offset: isize = former.forms.len() as isize - entry.form_base as isize;
            let input_offset: isize = former.consumed.len() as isize - entry.input_base as isize;

            for form in &entry.forms {
                former.forms.push(form.clone());
            }

            for input in &entry.inputs {
                former.consumed.push(input.clone());
            }

            for &index in &entry.consumed {
                classifier
                    .consumed
                    .push((index as isize + input_offset) as Identity);
            }

            for &index in &entry.stack {
                let shifted = if index == 0 {
                    0
                } else {
                    (index as isize + form_offset) as Identity
                };
                classifier.stack.push(shifted);
            }

            classifier.marker = classifier.marker + entry.advance;
            classifier.position = entry.position;
            classifier.outcome = entry.outcome;
            classifier.form = if entry.form == 0 {
                0
            } else {
                (entry.form as isize + form_offset) as Identity
            };

            return;
        }

        let stashed = match former.stash.iter().find(|(k, _)| *k == key) {
            Some((_, action)) => action.clone(),
            None => {
                let built = (self.factory)();
                let action = built.action;
                former.stash.push((key, action.clone()));
                action
            }
        };

        let class_consumed = classifier.consumed.len();
        let class_stack = classifier.stack.len();
        let form_base = former.forms.len();
        let input_base = former.consumed.len();
        let origin = classifier.marker;

        let mut child = Classifier::create(
            stashed,
            classifier.marker,
            classifier.position,
            take(&mut classifier.consumed),
            Outcome::Blank,
            0,
            take(&mut classifier.stack),
            classifier.depth + 1,
        );

        former.build(&mut child);

        let forms: Vec<_> = former.forms[form_base..].iter().cloned().collect();
        let inputs: Vec<_> = former.consumed[input_base..].iter().cloned().collect();
        let consumed: Vec<_> = child.consumed[class_consumed..].to_vec();
        let stack: Vec<_> = child.stack[class_stack..].to_vec();

        former.memo.insert(
            memo_key,
            Memo {
                outcome: child.outcome,
                advance: child.marker - origin,
                position: child.position,
                forms,
                inputs,
                consumed,
                stack,
                form: child.form,
                form_base,
                input_base,
            },
        );

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.outcome = child.outcome;
        classifier.form = child.form;
        classifier.stack = child.stack;
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
        let form_used = former.consumed.len();
        let form_forms = former.forms.len();
        let class_used = classifier.consumed.len();
        let class_stack = classifier.stack.len();

        let mut child = classifier.create_child(self.state.action.clone());
        former.build(&mut child);

        let effected = child.is_effected();

        classifier.consumed = child.consumed;
        classifier.stack = child.stack;

        if effected {
            classifier.marker = child.marker;
            classifier.position = child.position;
            classifier.form = child.form;
            classifier.set_align();
        } else {
            former.consumed.truncate(form_used);
            former.forms.truncate(form_forms);
            classifier.consumed.truncate(class_used);
            classifier.stack.truncate(class_stack);
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
        let mut mark = classifier.marker;
        let mut pos = classifier.position;

        let form_used = former.consumed.len();
        let form_forms = former.forms.len();

        let mut consumed = take(&mut classifier.consumed);
        let mut stack = take(&mut classifier.stack);
        let class_used = consumed.len();
        let class_stack = stack.len();

        let mut forms = Vec::with_capacity(SIZE);
        let mut broke = false;

        for pattern in &self.states {
            let mut child = Classifier::create(
                pattern.action.clone(),
                mark,
                pos,
                consumed,
                Outcome::Blank,
                0,
                stack,
                classifier.depth + 1,
            );

            former.build(&mut child);

            consumed = child.consumed;
            stack = child.stack;

            match child.outcome {
                Outcome::Aligned => {
                    classifier.outcome = child.outcome;
                    mark = child.marker;
                    pos = child.position;
                    forms.push(child.form);
                }
                Outcome::Panicked | Outcome::Failed => {
                    classifier.outcome = child.outcome;
                    mark = child.marker;
                    pos = child.position;
                    forms.push(child.form);
                    break;
                }
                Outcome::Ignored => {
                    mark = child.marker;
                    pos = child.position;
                }
                _ => {
                    classifier.outcome = child.outcome;
                    broke = true;
                    break;
                }
            }
        }

        classifier.consumed = consumed;
        classifier.stack = stack;

        if broke {
            former.consumed.truncate(form_used);
            former.forms.truncate(form_forms);
            classifier.consumed.truncate(class_used);
            classifier.stack.truncate(class_stack);
        } else {
            classifier.marker = mark;
            classifier.position = pos;

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
        let mut mark = classifier.marker;
        let mut pos = classifier.position;
        let mut forms = Vec::new();

        let form_used = former.consumed.len();
        let form_forms = former.forms.len();

        let mut consumed = take(&mut classifier.consumed);
        let mut stack = take(&mut classifier.stack);
        let class_used = consumed.len();
        let class_stack = stack.len();

        while former.source.peek_ahead(mark).is_some() {
            let step_used = former.consumed.len();
            let step_forms = former.forms.len();
            let step_consumed = consumed.len();
            let step_stack = stack.len();

            let mut child = Classifier::create(
                self.state.action.clone(),
                mark,
                pos,
                consumed,
                Outcome::Blank,
                0,
                stack,
                classifier.depth + 1,
            );

            former.build(&mut child);

            consumed = child.consumed;
            stack = child.stack;

            if child.marker == mark {
                former.consumed.truncate(step_used);
                former.forms.truncate(step_forms);
                consumed.truncate(step_consumed);
                stack.truncate(step_stack);
                break;
            }

            if self.persist {
                match child.outcome {
                    Outcome::Panicked | Outcome::Aligned | Outcome::Failed => {
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                    }
                    Outcome::Ignored => {
                        former.consumed.truncate(step_used);
                        former.forms.truncate(step_forms);
                        consumed.truncate(step_consumed);
                        stack.truncate(step_stack);
                        mark = child.marker;
                        pos = child.position;
                    }
                    _ => {
                        former.consumed.truncate(step_used);
                        former.forms.truncate(step_forms);
                        consumed.truncate(step_consumed);
                        stack.truncate(step_stack);
                    }
                }
            } else {
                match child.outcome {
                    Outcome::Panicked | Outcome::Failed => {
                        classifier.outcome = child.outcome;
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                        break;
                    }
                    Outcome::Aligned => {
                        classifier.outcome = child.outcome;
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                    }
                    Outcome::Ignored => {
                        former.consumed.truncate(step_used);
                        former.forms.truncate(step_forms);
                        consumed.truncate(step_consumed);
                        stack.truncate(step_stack);
                        mark = child.marker;
                        pos = child.position;
                    }
                    _ => {
                        former.consumed.truncate(step_used);
                        former.forms.truncate(step_forms);
                        consumed.truncate(step_consumed);
                        stack.truncate(step_stack);
                    }
                }
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
            if self.persist {
                classifier.set_align();
            }
            classifier.marker = mark;
            classifier.position = pos;

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
            former.consumed.truncate(form_used);
            former.forms.truncate(form_forms);
            classifier.consumed.truncate(class_used);
            classifier.stack.truncate(class_stack);

            if self.persist {
                classifier.set_empty();
            }
        }
    }
}
