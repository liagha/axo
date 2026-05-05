use crate::{
    combinator::{
        next_identity, Alternative, Combinator, Deferred, Fail, Formable, Former, Ignore, Literal,
        Multiple, Optional, Outcome, Panic, Predicate, Recover, Repetition, Sequence, Skip,
        Transform,
    },
    data::{
        memory::{take, Arc},
        Identity, Offset, Scale,
    },
    tracker::Peekable,
};

pub struct Formation<'a: 'source, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub identity: Identity,
    pub combinator: Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    >,
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
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn new(
        combinator: Arc<
            dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
            + Send
            + Sync
            + 'source,
        >,
        marker: Offset,
        state: Source::State,
    ) -> Self {
        Self {
            identity: next_identity(),
            combinator,
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
    pub(super) fn create(
        combinator: Arc<
            dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
            + Send
            + Sync
            + 'source,
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
            combinator,
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
    pub(super) fn create_child(
        &mut self,
        combinator: Arc<
            dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
            + Send
            + Sync
            + 'source,
        >,
    ) -> Self {
        Self {
            identity: next_identity(),
            combinator,
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
        Self::new(Arc::new(Deferred { factory }), 0, Default::default())
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
    pub fn with_combinator(
        mut self,
        combinator: Arc<
            dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
            + Send
            + Sync
            + 'source,
        >,
    ) -> Self {
        let combinators = vec![self.combinator.clone(), combinator];
        self.combinator = Arc::new(Multiple { combinators });
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
        self.with_combinator(Arc::new(Fail {
            emitter: Arc::new(emitter),
            phantom: Default::default(),
        }))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.with_combinator(Arc::new(Ignore))
    }

    #[inline]
    pub fn with_multiple(
        self,
        combinators: Vec<
            Arc<
                dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
                + Send
                + Sync
                + 'source,
            >,
        >,
    ) -> Self {
        self.with_combinator(Arc::new(Multiple { combinators }))
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
        self.with_combinator(Self::panic(emitter))
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
        self.with_combinator(Self::recover(sync, emitter))
    }

    #[inline]
    pub fn with_skip(self) -> Self {
        self.with_combinator(Arc::new(Skip))
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
        self.with_combinator(Self::transform(transform))
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
    ) -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    >
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
    ) -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    >
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
    ) -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    >
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
    ) -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    >
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
    pub fn ignore() -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    > {
        Arc::new(Ignore)
    }

    #[inline]
    pub fn multiple(
        combinators: Vec<
            Arc<
                dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
                + Send
                + Sync
                + 'source,
            >,
        >,
    ) -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    > {
        Arc::new(Multiple { combinators })
    }

    #[inline]
    pub fn skip() -> Arc<
        dyn Combinator<'a, Former<'a, 'source, Source, Input, Output, Failure>, Self>
        + Send
        + Sync
        + 'source,
    > {
        Arc::new(Skip)
    }
}