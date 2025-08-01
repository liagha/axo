use {
    super::{
        Formable,
        form::Form,
        order::*,
        former::{
            record::*,
            Composer,
            Draft
        },
    },
    crate::{
        axo_internal::{
            compiler::Registry,
        },
        thread::{
            Arc,
        },
    },
};

#[derive(Clone)]
pub struct Literal<Input> {
    pub value: Arc<dyn PartialEq<Input> + Send + Sync>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Literal<Input> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Negate<Input: Formable, Output: Formable, Failure: Formable> {
    pub classifier: Box<Classifier<Input, Output, Failure>>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Negate<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Predicate<Input> {
    pub function: Arc<dyn Fn(&Input) -> bool + Send + Sync>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Predicate<Input> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Alternative<Input: Formable, Output: Formable, Failure: Formable, const SIZE: usize> {
    pub patterns: [Classifier<Input, Output, Failure>; SIZE],
    pub perfection: Vec<Record>,
    pub blacklist: Vec<Record>,
}

impl<Input: Formable, Output: Formable, Failure: Formable, const SIZE: usize> Order<Input, Output, Failure> for Alternative<Input, Output, Failure, SIZE> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut best: Option<Draft<Input, Output, Failure>> = None;

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
pub struct Deferred<Input: Formable, Output: Formable, Failure: Formable> {
    pub function: Arc<dyn Fn() -> Classifier<Input, Output, Failure> + Send + Sync>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Deferred<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Optional<Input: Formable, Output: Formable, Failure: Formable> {
    pub classifier: Box<Classifier<Input, Output, Failure>>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Optional<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Wrapper<Input: Formable, Output: Formable, Failure: Formable> {
    pub classifier: Box<Classifier<Input, Output, Failure>>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Wrapper<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Ranked<Input: Formable, Output: Formable, Failure: Formable> {
    pub classifier: Box<Classifier<Input, Output, Failure>>,
    pub precedence: Record,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Ranked<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Sequence<Input: Formable, Output: Formable, Failure: Formable, const SIZE: usize> {
    pub patterns: [Classifier<Input, Output, Failure>; SIZE],
}

impl<Input: Formable, Output: Formable, Failure: Formable, const SIZE: usize> Order<Input, Output, Failure> for Sequence<Input, Output, Failure, SIZE> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
pub struct Repetition<Input: Formable, Output: Formable, Failure: Formable> {
    pub classifier: Box<Classifier<Input, Output, Failure>>,
    pub minimum: usize,
    pub maximum: Option<usize>,
    pub update: Vec<Record>,
    pub accept: Vec<Record>,
    pub consume: Vec<Record>,
    pub halt: Vec<Record>,
    pub align_on_success: bool,
    pub empty_on_failure: bool,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Repetition<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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

#[derive(Clone)]
pub struct Classifier<Input: Formable, Output: Formable, Failure: Formable> {
    pub order: Arc<dyn Order<Input, Output, Failure>>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Classifier<Input, Output, Failure> {
    #[inline]
    pub const fn new(classifier: Arc<dyn Order<Input, Output, Failure>>) -> Self {
        Self {
            order: classifier,
        }
    }

    #[inline]
    pub fn literal(value: impl PartialEq<Input> + Send + Sync + 'static) -> Self {
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
        F: Fn(&Input) -> bool + Send + Sync + 'static,
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
        F: Fn() -> Self + Send + Sync + 'static,
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
    pub fn with_order(mut self, order: Arc<dyn Order<Input, Output, Failure>>) -> Self {
        let order = Multiple { orders: vec![self.order, order] };
        self.order = Arc::new(order);
        self
    }

    #[inline]
    pub fn with_align(self) -> Self {
        self.with_order(Arc::new(Align))
    }

    #[inline]
    pub fn with_branch(self, found: Arc<dyn Order<Input, Output, Failure>>, missing: Arc<dyn Order<Input, Output, Failure>>) -> Self {
        self.with_order(Arc::new(
            Branch {
                found,
                missing,
            }
        ))
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
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
        I: Fn(Draft<Input, Output, Failure>) -> Arc<dyn Order<Input, Output, Failure>> + Send + Sync + 'static
    {
        self.with_order(Arc::new(Inspect { inspector: Arc::new(inspector) }))
    }

    #[inline]
    pub fn with_multiple(self, orders: Vec<Arc<dyn Order<Input, Output, Failure>>>) -> Self {
        self.with_order(Self::multiple(orders))
    }

    #[inline]
    pub fn with_panic<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
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
        F: FnMut() + Send + Sync + 'static,
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
        T: FnMut(&mut Registry, Form<Input, Output, Failure>) -> Result<Form<Input, Output, Failure>, Failure>
        + Send
        + Sync
        + 'static,
    {
        self.with_order(Self::transform(transform))
    }

    #[inline]
    pub fn with_fallback(self, order: Arc<dyn Order<Input, Output, Failure>>) -> Self {
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