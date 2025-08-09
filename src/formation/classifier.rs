use {
    super::{
        form::Form,
        former::{
            record::*,
            Composer,
            Draft
        },
        helper::Formable,
        order::*,
    },
    crate::{
        data::{
            thread::{
                Arc,
            },
        },
        internal::{
            compiler::Registry,
        },
    },
};

#[derive(Clone)]
pub struct Classifier<'classifier, Input: Formable<'classifier>, Output: Formable<'classifier>, Failure: Formable<'classifier>> {
    pub order: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
}

impl<'classifier, Input: Formable<'classifier>, Output: Formable<'classifier>, Failure: Formable<'classifier>> Classifier<'classifier, Input, Output, Failure> {
    #[inline]
    pub const fn new(classifier: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Self {
        Self {
            order: classifier,
        }
    }

    #[inline]
    pub fn literal(value: impl PartialEq<Input> + 'classifier) -> Self {
        Self::new(Arc::new(Literal {
            value: Arc::new(value),
        }))
    }

    #[inline]
    pub fn negate(classifier: Self) -> Self {
        Self::new(Arc::new(Negate {
            classifier: Box::new(classifier),
        }))
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + 'classifier,
    {
        Self::new(Arc::new(Predicate {
            function: Arc::new(predicate),
        }))
    }

    #[inline]
    pub fn alternative<const SIZE: usize>(patterns: [Self; SIZE]) -> Self {
        Self::new(Arc::new(Alternative {
            patterns,
            perfection: vec![PANICKED, ALIGNED],
            blacklist: vec![BLANK]
        }))
    }

    #[inline]
    pub fn choice<const SIZE: usize>(patterns: [Self; SIZE], perfection: Vec<Record>) -> Self {
        Self::new(Arc::new(Alternative {
            patterns,
            perfection,
            blacklist: vec![]
        }))
    }

    #[inline]
    pub fn sequence<const SIZE: usize>(patterns: [Self; SIZE]) -> Self {
        Self::new(Arc::new(Sequence { patterns }))
    }

    #[inline]
    pub fn optional(classifier: Self) -> Self {
        Self::new(Arc::new(Optional {
            classifier: Box::new(classifier),
        }))
    }

    #[inline]
    pub fn persistence(classifier: Self, minimum: usize, maximum: Option<usize>) -> Self {
        Self::new(Arc::new(Repetition {
            classifier: Box::new(classifier),
            minimum,
            maximum,
            update: vec![],
            accept: vec![PANICKED, ALIGNED, FAILED, IGNORED],
            consume: vec![PANICKED, ALIGNED, FAILED],
            halt: vec![],
            align_on_success: true,
            empty_on_failure: true,
        }))
    }

    #[inline]
    pub fn repetition(classifier: Self, minimum: usize, maximum: Option<usize>) -> Self {
        Self::new(Arc::new(Repetition {
            classifier: Box::new(classifier),
            minimum,
            maximum,
            update: vec![ALIGNED, PANICKED, FAILED],
            accept: vec![ALIGNED, PANICKED, FAILED, IGNORED],
            consume: vec![ALIGNED, PANICKED, FAILED],
            halt: vec![PANICKED, FAILED],
            align_on_success: false,
            empty_on_failure: false,
        }))
    }

    #[inline]
    pub fn continuous(classifier: Self, minimum: usize, maximum: Option<usize>) -> Self {
        Self::new(Arc::new(Repetition {
            classifier: Box::new(classifier),
            minimum,
            maximum,
            update: vec![ALIGNED, PANICKED, FAILED],
            accept: vec![ALIGNED, PANICKED, FAILED, IGNORED],
            consume: vec![ALIGNED, PANICKED, FAILED],
            halt: vec![],
            align_on_success: false,
            empty_on_failure: false,
        }))
    }

    #[inline]
    pub fn wrapper(classifier: Self) -> Self {
        Self::new(Arc::new(Wrapper {
            classifier: Box::new(classifier),
        }))
    }

    #[inline]
    pub fn ranked(classifier: Self, precedence: Record) -> Self {
        Self::new(Arc::new(Ranked {
            classifier: Box::new(classifier),
            precedence,
        }))
    }

    #[inline]
    pub fn deferred<F>(factory: F) -> Self
    where
        F: Fn() -> Self + 'classifier,
    {
        Self::new(Arc::new(Deferred {
            function: Arc::new(factory),
        }))
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
    pub fn with_order(mut self, order: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Self {
        let orders = vec![self.order.clone(), order];
        let multiple: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> = Arc::new(Multiple { orders });

        self.order = multiple;
        self
    }

    #[inline]
    pub fn with_align(self) -> Self {
        self.with_order(Arc::new(Align))
    }

    #[inline]
    pub fn with_branch(self, found: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>, missing: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Self {
        let branch: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> = Arc::new(Branch { found, missing });

        self.with_order(branch)
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + 'classifier,
    {
        self.with_order(Arc::new(Fail { emitter: Arc::new(emitter) }))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.with_order(Arc::new(Ignore))
    }

    #[inline]
    pub fn with_inspect<I>(self, inspector: I) -> Self
    where
        I: Fn(Draft<'classifier, Input, Output, Failure>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> + 'classifier
    {
        self.with_order(Arc::new(Inspect { inspector: Arc::new(inspector) }))
    }

    #[inline]
    pub fn with_multiple(self, orders: Vec<Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>>) -> Self {
        let multiple: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> = Arc::new(Multiple { orders });

        self.with_order(multiple)
    }

    #[inline]
    pub fn with_panic<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + 'classifier,
    {
        self.with_order(Self::panic(emitter))
    }

    #[inline]
    pub fn with_pardon(self) -> Self {
        self.with_order(Arc::new(Pardon))
    }

    #[inline]
    pub fn with_perform<F>(self, executor: F) -> Self
    where
        F: FnMut() + 'classifier,
    {
        self.with_order(Self::perform(executor))
    }

    #[inline]
    pub fn with_skip(self) -> Self {
        self.with_order(Arc::new(Skip))
    }

    #[inline]
    pub fn with_transform<T>(self, transform: T) -> Self
    where
        T: FnMut(&mut Registry, Form<'classifier, Input, Output, Failure>) -> Result<Form<'classifier, Input, Output, Failure>, Failure> + 'classifier,
    {
        self.with_order(Self::transform(transform))
    }

    #[inline]
    pub fn with_fallback(self, order: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Self {
        self.with_branch(Self::perform(|| {}), order)
    }

    #[inline]
    pub fn as_optional(&self) -> Self {
        Self::optional(self.clone())
    }

    #[inline]
    pub fn as_persistence(&self, min: usize, max: Option<usize>) -> Self {
        Self::persistence(self.clone(), min, max)
    }
}

#[derive(Clone)]
pub struct Literal<'literal, Input> {
    pub value: Arc<dyn PartialEq<Input> + 'literal>,
}

impl<'literal, Input: Formable<'literal>, Output: Formable<'literal>, Failure: Formable<'literal>> Order<'literal, Input, Output, Failure> for Literal<'literal, Input> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'literal, Input, Output, Failure>, draft: &mut Draft<'literal, Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(draft.marker).cloned() {
            if self.value.eq(&peek) {
                draft.set_align();
                composer.source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.form = Form::input(peek);
            } else {
                draft.set_empty();
            }
        } else {
            draft.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Negate<'negate, Input: Formable<'negate>, Output: Formable<'negate>, Failure: Formable<'negate>> {
    pub classifier: Box<Classifier<'negate, Input, Output, Failure>>,
}

impl<'negate, Input: Formable<'negate>, Output: Formable<'negate>, Failure: Formable<'negate>> Order<'negate, Input, Output, Failure> for Negate<'negate, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'negate, Input, Output, Failure>, draft: &mut Draft<'negate, Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(draft.marker).cloned() {
            let mut child = Draft::new(draft.marker, draft.position, self.classifier.as_ref().clone());
            composer.build(&mut child);

            if !child.is_aligned() {
                draft.set_align();
                composer.source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.form = Form::input(peek);
            } else {
                draft.set_empty();
            }
        } else {
            draft.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Predicate<'predicate, Input> {
    pub function: Arc<dyn Fn(&Input) -> bool + 'predicate>,
}

impl<'predicate, Input: Formable<'predicate>, Output: Formable<'predicate>, Failure: Formable<'predicate>> Order<'predicate, Input, Output, Failure> for Predicate<'predicate, Input> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'predicate, Input, Output, Failure>, draft: &mut Draft<'predicate, Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(draft.marker).cloned() {
            let predicate = (self.function)(&peek);

            if predicate {
                draft.set_align();
                composer.source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.form = Form::input(peek);
            } else {
                draft.set_empty();
            }
        } else {
            draft.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Alternative<'alternative, Input: Formable<'alternative>, Output: Formable<'alternative>, Failure: Formable<'alternative>, const SIZE: usize> {
    pub patterns: [Classifier<'alternative, Input, Output, Failure>; SIZE],
    pub perfection: Vec<Record>,
    pub blacklist: Vec<Record>,
}

impl<'alternative, Input: Formable<'alternative>, Output: Formable<'alternative>, Failure: Formable<'alternative>, const SIZE: usize> Order<'alternative, Input, Output, Failure> for Alternative<'alternative, Input, Output, Failure, SIZE> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'alternative, Input, Output, Failure>, draft: &mut Draft<'alternative, Input, Output, Failure>) {
        let mut best: Option<Draft<'alternative, Input, Output, Failure>> = None;

        for classifier in &self.patterns {
            let mut child = Draft::new(draft.marker, draft.position, classifier.clone());
            composer.build(&mut child);

            if self.blacklist.contains(&child.record) {
                continue;
            }

            match &best {
                None => {
                    best = Some(child.clone())
                },
                Some(champion) => {
                    if child.record > champion.record {
                        best = Some(child.clone());
                    }
                }
            }

            if self.perfection.contains(&child.record) {
                break;
            }
        }

        match best {
            Some(champion) => {
                draft.record = champion.record;
                draft.marker = champion.marker;
                draft.position = champion.position;
                draft.consumed = champion.consumed;
                draft.form = champion.form;
            }
            None => draft.set_empty(),
        }
    }
}

#[derive(Clone)]
pub struct Deferred<'deferred, Input: Formable<'deferred>, Output: Formable<'deferred>, Failure: Formable<'deferred>> {
    pub function: Arc<dyn Fn() -> Classifier<'deferred, Input, Output, Failure> + 'deferred>,
}

impl<'deferred, Input: Formable<'deferred>, Output: Formable<'deferred>, Failure: Formable<'deferred>> Order<'deferred, Input, Output, Failure> for Deferred<'deferred, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'deferred, Input, Output, Failure>, draft: &mut Draft<'deferred, Input, Output, Failure>) {
        let resolved = (self.function)();
        let mut child = Draft::new(draft.marker, draft.position, resolved);
        composer.build(&mut child);

        draft.marker = child.marker;
        draft.position = child.position;
        draft.consumed = child.consumed;
        draft.record = child.record;
        draft.form = child.form;
    }
}

#[derive(Clone)]
pub struct Optional<'optional, Input: Formable<'optional>, Output: Formable<'optional>, Failure: Formable<'optional>> {
    pub classifier: Box<Classifier<'optional, Input, Output, Failure>>,
}

impl<'optional, Input: Formable<'optional>, Output: Formable<'optional>, Failure: Formable<'optional>> Order<'optional, Input, Output, Failure> for Optional<'optional, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'optional, Input, Output, Failure>, draft: &mut Draft<'optional, Input, Output, Failure>) {
        let mut child = Draft::new(draft.marker, draft.position, self.classifier.as_ref().clone());
        composer.build(&mut child);

        if child.is_effected() {
            draft.marker = child.marker;
            draft.position = child.position;
            draft.consumed = child.consumed;
            draft.form = child.form;
            draft.set_align();
        } else {
            draft.set_ignore();
        }
    }
}

#[derive(Clone)]
pub struct Wrapper<'wrapper, Input: Formable<'wrapper>, Output: Formable<'wrapper>, Failure: Formable<'wrapper>> {
    pub classifier: Box<Classifier<'wrapper, Input, Output, Failure>>,
}

impl<'wrapper, Input: Formable<'wrapper>, Output: Formable<'wrapper>, Failure: Formable<'wrapper>> Order<'wrapper, Input, Output, Failure> for Wrapper<'wrapper, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'wrapper, Input, Output, Failure>, draft: &mut Draft<'wrapper, Input, Output, Failure>) {
        let mut child = Draft::new(draft.marker, draft.position, self.classifier.as_ref().clone());
        composer.build(&mut child);

        draft.marker = child.marker;
        draft.position = child.position;
        draft.consumed = child.consumed;
        draft.record = child.record;
        draft.form = child.form;
    }
}

#[derive(Clone)]
pub struct Ranked<'ranked, Input: Formable<'ranked>, Output: Formable<'ranked>, Failure: Formable<'ranked>> {
    pub classifier: Box<Classifier<'ranked, Input, Output, Failure>>,
    pub precedence: Record,
}

impl<'ranked, Input: Formable<'ranked>, Output: Formable<'ranked>, Failure: Formable<'ranked>> Order<'ranked, Input, Output, Failure> for Ranked<'ranked, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'ranked, Input, Output, Failure>, draft: &mut Draft<'ranked, Input, Output, Failure>) {
        let mut child = Draft::new(draft.marker, draft.position, self.classifier.as_ref().clone());
        composer.build(&mut child);

        draft.marker = child.marker;
        draft.position = child.position;
        draft.consumed = child.consumed.clone();
        draft.form = child.form.clone();

        if child.is_aligned() {
            draft.record = self.precedence.max(ALIGNED);
        } else if child.is_failed() {
            draft.record = self.precedence.min(FAILED);
        } else {
            draft.record = child.record;
        }
    }
}

#[derive(Clone)]
pub struct Sequence<'sequence, Input: Formable<'sequence>, Output: Formable<'sequence>, Failure: Formable<'sequence>, const SIZE: usize> {
    pub patterns: [Classifier<'sequence, Input, Output, Failure>; SIZE],
}

impl<'sequence, Input: Formable<'sequence>, Output: Formable<'sequence>, Failure: Formable<'sequence>, const SIZE: usize> Order<'sequence, Input, Output, Failure> for Sequence<'sequence, Input, Output, Failure, SIZE> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'sequence, Input, Output, Failure>, draft: &mut Draft<'sequence, Input, Output, Failure>) {
        let mut index = draft.marker;
        let mut position = draft.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::with_capacity(SIZE);

        for classifier in &self.patterns {
            let mut child = Draft::new(index, position, classifier.clone());
            composer.build(&mut child);

            match child.record {
                ALIGNED => {
                    draft.record = child.record;
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                }
                PANICKED | FAILED => {
                    draft.record = child.record;
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                    break;
                }
                IGNORED => {
                    index = child.marker;
                    position = child.position;
                }
                _ => {
                    draft.record = child.record;
                    break;
                }
            }
        }

        draft.marker = index;
        draft.position = position;
        draft.consumed = consumed;
        draft.form = Form::multiple(forms);
    }
}

#[derive(Clone)]
pub struct Repetition<'repetition, Input: Formable<'repetition>, Output: Formable<'repetition>, Failure: Formable<'repetition>> {
    pub classifier: Box<Classifier<'repetition, Input, Output, Failure>>,
    pub minimum: usize,
    pub maximum: Option<usize>,
    pub update: Vec<Record>,
    pub accept: Vec<Record>,
    pub consume: Vec<Record>,
    pub halt: Vec<Record>,
    pub align_on_success: bool,
    pub empty_on_failure: bool,
}

impl<'repetition, Input: Formable<'repetition>, Output: Formable<'repetition>, Failure: Formable<'repetition>> Order<'repetition, Input, Output, Failure> for Repetition<'repetition, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'repetition, Input, Output, Failure>, draft: &mut Draft<'repetition, Input, Output, Failure>) {
        let mut index = draft.marker;
        let mut position = draft.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::new();

        while composer.source.peek_ahead(index).is_some() {
            let mut child = Draft::new(index, position, self.classifier.as_ref().clone());
            composer.build(&mut child);

            if child.marker == index {
                break;
            }

            if self.update.contains(&child.record) {
                draft.record = child.record;
            }

            if self.accept.contains(&child.record) {
                index = child.marker;
                position = child.position;
            }

            if self.consume.contains(&child.record) {
                consumed.extend(child.consumed);
                forms.push(child.form);
            }

            if self.halt.contains(&child.record) {
                break;
            }

            if let Some(max) = self.maximum {
                if forms.len() >= max {
                    break;
                }
            }
        }

        if forms.len() >= self.minimum {
            if self.align_on_success {
                draft.set_align();
            }
            draft.marker = index;
            draft.position = position;
            draft.consumed = consumed;
            draft.form = Form::multiple(forms);
        } else {
            if self.empty_on_failure {
                draft.set_empty();
            }
        }
    }
}