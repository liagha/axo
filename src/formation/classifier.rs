use {
    super::{
        form::Form,
        former::{
            record::{
                Record,
            },
            Former,
        },
        helper::Formable,
        order::*,
    },
    crate::{
        data::{
            thread::{
                Arc, Mutex,
            },
            Scale,
            Offset,
            Boolean,
        },
        tracker::{
            Position,
            Location,
        },
    },
};

#[derive(Clone)]
pub struct Classifier<'classifier, Input: Formable<'classifier>, Output: Formable<'classifier>, Failure: Formable<'classifier>> {
    pub order: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    pub marker: Offset,
    pub position: Position<'classifier>,
    pub consumed: Vec<Input>,
    pub record: Record,
    pub form: Form<'classifier, Input, Output, Failure>,
    pub depth: Scale,
}

impl<'classifier, Input: Formable<'classifier>, Output: Formable<'classifier>, Failure: Formable<'classifier>> Classifier<'classifier, Input, Output, Failure> {
    #[inline]
    pub fn new(classifier: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>, marker: Offset, position: Position<'classifier>) -> Self {
        Self {
            order: classifier,
            marker,
            position,
            consumed: Vec::new(),
            record: Record::Blank,
            form: Form::Blank,
            depth: 0,
        }
    }

    #[inline]
    pub fn new_with_depth(classifier: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>, marker: Offset, position: Position<'classifier>, depth: Scale) -> Self {
        Self {
            order: classifier,
            marker,
            position,
            consumed: Vec::new(),
            record: Record::Blank,
            form: Form::Blank,
            depth,
        }
    }

    #[inline]
    pub const fn is_panicked(&self) -> bool {
        matches!(self.record, Record::Panicked)
    }

    #[inline]
    pub const fn is_aligned(&self) -> bool {
        matches!(self.record, Record::Aligned)
    }

    #[inline]
    pub const fn is_failed(&self) -> bool {
        matches!(self.record, Record::Failed)
    }

    #[inline]
    pub const fn is_effected(&self) -> bool {
        matches!(self.record, Record::Aligned | Record::Failed)
    }

    #[inline]
    pub const fn is_blank(&self) -> bool {
        matches!(self.record, Record::Blank)
    }

    #[inline]
    pub const fn is_ignored(&self) -> bool {
        matches!(self.record, Record::Ignored)
    }

    #[inline]
    pub fn set_panic(&mut self) {
        self.record = Record::Panicked;
    }

    #[inline]
    pub fn set_align(&mut self) {
        self.record = Record::Aligned;
    }

    #[inline]
    pub fn set_fail(&mut self) {
        self.record = Record::Failed;
    }

    #[inline]
    pub fn set_empty(&mut self) {
        self.record = Record::Blank;
    }

    #[inline]
    pub fn set_ignore(&mut self) {
        self.record = Record::Ignored;
    }

    #[inline]
    fn create_child(&self, order: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Self {
        Self {
            order,
            marker: self.marker,
            position: self.position,
            consumed: Vec::new(),
            record: Record::Blank,
            form: Form::Blank,
            depth: self.depth + 1,
        }
    }

    #[inline]
    pub fn literal(value: impl PartialEq<Input> + 'classifier) -> Self {
        Self::new(Arc::new(Literal {
            value: Arc::new(value),
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn negate(classifier: Self) -> Self {
        Self::new(Arc::new(Negate {
            classifier: Box::new(classifier),
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + 'classifier,
    {
        Self::new(Arc::new(Predicate::<Input> {
            function: Arc::new(predicate),
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(Arc::new(Alternative {
            patterns,
            perfection: vec![Record::Panicked, Record::Aligned],
            blacklist: vec![Record::Blank]
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(Arc::new(Sequence { patterns }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn optional(classifier: Self) -> Self {
        Self::new(Arc::new(Optional {
            classifier: Box::new(classifier),
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn persistence(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(Arc::new(Repetition {
            classifier: Box::new(classifier),
            minimum,
            maximum,
            persist: true,
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn repetition(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(Arc::new(Repetition {
            classifier: Box::new(classifier),
            minimum,
            maximum,
            persist: false,
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn wrapper(classifier: Self) -> Self {
        Self::new(Arc::new(Wrapper {
            classifier: Box::new(classifier),
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn ranked(classifier: Self, precedence: i8) -> Self {
        Self::new(Arc::new(Ranked {
            classifier: Box::new(classifier),
            precedence,
        }), 0, Position::new(Location::Void))
    }

    #[inline]
    pub fn deferred<F>(factory: F) -> Self
    where
        F: Fn() -> Self + 'classifier,
    {
        Self::new(Arc::new(Deferred {
            function: Arc::new(factory),
        }), 0, Position::new(Location::Void))
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
        F: Fn(Form<Input, Output, Failure>) -> Failure + 'classifier,
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
        I: Fn(Classifier<'classifier, Input, Output, Failure>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> + 'classifier
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
        F: Fn(Form<Input, Output, Failure>) -> Failure + 'classifier,
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
        T: FnMut(Form<'classifier, Input, Output, Failure>) -> Result<Form<'classifier, Input, Output, Failure>, Failure> + 'classifier,
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
    pub fn as_persistence(&self, min: Scale, max: Option<Scale>) -> Self {
        Self::persistence(self.clone(), min, max)
    }

    #[inline]
    pub fn transform<T>(transformer: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: FnMut(Form<'classifier, Input, Output, Failure>) -> Result<Form<'classifier, Input, Output, Failure>, Failure> + 'classifier,
    {
        Arc::new(Transform { transformer: Arc::new(Mutex::new(transformer))})
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(Form<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
    {
        Arc::new(Fail { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(Form<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
    {
        Arc::new(Panic { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn ignore() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Ignore)
    }

    #[inline]
    pub fn inspect<T>(inspector: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(Classifier<'classifier, Input, Output, Failure>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> + 'classifier
    {
        Arc::new(Inspect { inspector: Arc::new(inspector) })
    }

    #[inline]
    pub fn multiple(orders: Vec<Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Arc::new(Multiple { orders })
    }

    #[inline]
    pub fn pardon() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Pardon)
    }

    #[inline]
    pub fn perform<T>(executor: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: FnMut() + 'classifier,
    {
        Arc::new(Perform { performer: Arc::new(Mutex::new(executor))})
    }

    #[inline]
    pub fn skip() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Skip)
    }

    #[inline]
    pub fn branch(found: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>, missing: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Arc::new(Branch { found, missing })
    }
}

#[derive(Clone)]
pub struct Literal<'literal, Input> {
    pub value: Arc<dyn PartialEq<Input> + 'literal>,
}

impl<'literal, Input: Formable<'literal>, Output: Formable<'literal>, Failure: Formable<'literal>> Order<'literal, Input, Output, Failure> for Literal<'literal, Input> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'literal, Input, Output, Failure>, classifier: &mut Classifier<'literal, Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(classifier.marker).cloned() {
            if self.value.eq(&peek) {
                classifier.set_align();
                composer.source.next(&mut classifier.marker, &mut classifier.position);
                classifier.consumed.push(peek.clone());
                classifier.form = Form::input(peek);
            } else {
                classifier.set_empty();
            }
        } else {
            classifier.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Negate<'negate, Input: Formable<'negate>, Output: Formable<'negate>, Failure: Formable<'negate>> {
    pub classifier: Box<Classifier<'negate, Input, Output, Failure>>,
}

impl<'negate, Input: Formable<'negate>, Output: Formable<'negate>, Failure: Formable<'negate>> Order<'negate, Input, Output, Failure> for Negate<'negate, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'negate, Input, Output, Failure>, classifier: &mut Classifier<'negate, Input, Output, Failure>) {
        let mut child = classifier.create_child(self.classifier.order.clone());
        composer.build(&mut child);

        if child.is_aligned() {
            classifier.set_empty();
        } else if child.is_effected() {
            classifier.set_align();
            classifier.form = Form::Blank;
        } else {
            classifier.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Predicate<'predicate, Input: Formable<'predicate>> {
    pub function: Arc<dyn Fn(&Input) -> bool + 'predicate>,
}

impl<'predicate, Input: Formable<'predicate>, Output: Formable<'predicate>, Failure: Formable<'predicate>> Order<'predicate, Input, Output, Failure> for Predicate<'predicate, Input> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'predicate, Input, Output, Failure>, classifier: &mut Classifier<'predicate, Input, Output, Failure>) {
        if let Some(peek) = composer.source.peek_ahead(classifier.marker) {
            if (self.function)(peek) {
                let value = composer.source.get(classifier.marker).cloned().unwrap();
                classifier.set_align();
                composer.source.next(&mut classifier.marker, &mut classifier.position);
                classifier.consumed.push(value.clone());
                classifier.form = Form::input(value);
            } else {
                classifier.set_empty();
            }
        } else {
            classifier.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Alternative<'alternative, Input: Formable<'alternative>, Output: Formable<'alternative>, Failure: Formable<'alternative>, const SIZE: Scale> {
    pub patterns: [Classifier<'alternative, Input, Output, Failure>; SIZE],
    pub perfection: Vec<Record>,
    pub blacklist: Vec<Record>,
}

impl<'alternative, Input: Formable<'alternative>, Output: Formable<'alternative>, Failure: Formable<'alternative>, const SIZE: Scale> Order<'alternative, Input, Output, Failure> for Alternative<'alternative, Input, Output, Failure, SIZE> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'alternative, Input, Output, Failure>, classifier: &mut Classifier<'alternative, Input, Output, Failure>) {
        let mut best: Option<Classifier<'alternative, Input, Output, Failure>> = None;

        for pattern in &self.patterns {
            let mut child = classifier.create_child(pattern.order.clone());
            composer.build(&mut child);

            if self.blacklist.contains(&child.record) {
                continue;
            }

            if let Some(ref mut champion) = best {
                if child.is_aligned() && (champion.is_failed() || child.marker > champion.marker) {
                    *champion = child;
                }
            } else {
                best = Some(child);
            }

            if let Some(ref champion) = best {
                if self.perfection.contains(&champion.record) {
                    break;
                }
            }
        }

        match best {
            Some(champion) => {
                classifier.record = champion.record;
                classifier.marker = champion.marker;
                classifier.position = champion.position;
                classifier.consumed = champion.consumed;
                classifier.form = champion.form;
            }
            None => classifier.set_empty(),
        }
    }
}

#[derive(Clone)]
pub struct Deferred<'deferred, Input: Formable<'deferred>, Output: Formable<'deferred>, Failure: Formable<'deferred>> {
    pub function: Arc<dyn Fn() -> Classifier<'deferred, Input, Output, Failure> + 'deferred>,
}

impl<'deferred, Input: Formable<'deferred>, Output: Formable<'deferred>, Failure: Formable<'deferred>> Order<'deferred, Input, Output, Failure> for Deferred<'deferred, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'deferred, Input, Output, Failure>, classifier: &mut Classifier<'deferred, Input, Output, Failure>) {
        let mut resolved = (self.function)();
        resolved.marker = classifier.marker;
        resolved.position = classifier.position;
        resolved.depth = classifier.depth + 1;
        composer.build(&mut resolved);

        classifier.marker = resolved.marker;
        classifier.position = resolved.position;
        classifier.consumed = resolved.consumed;
        classifier.record = resolved.record;
        classifier.form = resolved.form;
    }
}

#[derive(Clone)]
pub struct Optional<'optional, Input: Formable<'optional>, Output: Formable<'optional>, Failure: Formable<'optional>> {
    pub classifier: Box<Classifier<'optional, Input, Output, Failure>>,
}

impl<'optional, Input: Formable<'optional>, Output: Formable<'optional>, Failure: Formable<'optional>> Order<'optional, Input, Output, Failure> for Optional<'optional, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'optional, Input, Output, Failure>, classifier: &mut Classifier<'optional, Input, Output, Failure>) {
        let mut child = classifier.create_child(self.classifier.order.clone());
        composer.build(&mut child);

        if child.is_effected() {
            classifier.marker = child.marker;
            classifier.position = child.position;
            classifier.consumed = child.consumed;
            classifier.form = child.form;
            classifier.set_align();
        } else {
            classifier.set_ignore();
        }
    }
}

#[derive(Clone)]
pub struct Wrapper<'wrapper, Input: Formable<'wrapper>, Output: Formable<'wrapper>, Failure: Formable<'wrapper>> {
    pub classifier: Box<Classifier<'wrapper, Input, Output, Failure>>,
}

impl<'wrapper, Input: Formable<'wrapper>, Output: Formable<'wrapper>, Failure: Formable<'wrapper>> Order<'wrapper, Input, Output, Failure> for Wrapper<'wrapper, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'wrapper, Input, Output, Failure>, classifier: &mut Classifier<'wrapper, Input, Output, Failure>) {
        let mut child = classifier.create_child(self.classifier.order.clone());
        composer.build(&mut child);

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.record = child.record;
        classifier.form = child.form;
    }
}

#[derive(Clone)]
pub struct Ranked<'ranked, Input: Formable<'ranked>, Output: Formable<'ranked>, Failure: Formable<'ranked>> {
    pub classifier: Box<Classifier<'ranked, Input, Output, Failure>>,
    pub precedence: i8,
}

impl<'ranked, Input: Formable<'ranked>, Output: Formable<'ranked>, Failure: Formable<'ranked>> Order<'ranked, Input, Output, Failure> for Ranked<'ranked, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'ranked, Input, Output, Failure>, classifier: &mut Classifier<'ranked, Input, Output, Failure>) {
        let mut child = classifier.create_child(self.classifier.order.clone());
        composer.build(&mut child);

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed.clone();
        classifier.form = child.form.clone();

        if child.is_aligned() {
            classifier.record = self.precedence.max(Record::Aligned.into()).into();
        } else if child.is_failed() {
            classifier.record = self.precedence.min(Record::Failed.into()).into();
        } else {
            classifier.record = child.record;
        }
    }
}

#[derive(Clone)]
pub struct Sequence<'sequence, Input: Formable<'sequence>, Output: Formable<'sequence>, Failure: Formable<'sequence>, const SIZE: Scale> {
    pub patterns: [Classifier<'sequence, Input, Output, Failure>; SIZE],
}

impl<'sequence, Input: Formable<'sequence>, Output: Formable<'sequence>, Failure: Formable<'sequence>, const SIZE: Scale> Order<'sequence, Input, Output, Failure> for Sequence<'sequence, Input, Output, Failure, SIZE> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'sequence, Input, Output, Failure>, classifier: &mut Classifier<'sequence, Input, Output, Failure>) {
        let mut index = classifier.marker;
        let mut position = classifier.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::with_capacity(SIZE);

        for pattern in &self.patterns {
            let mut child = Classifier {
                order: pattern.order.clone(),
                marker: index,
                position,
                consumed: Vec::new(),
                record: Record::Blank,
                form: Form::Blank,
                depth: classifier.depth + 1,
            };
            composer.build(&mut child);

            match child.record {
                Record::Aligned => {
                    classifier.record = child.record;
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                }
                Record::Panicked | Record::Failed => {
                    classifier.record = child.record;
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                    break;
                }
                Record::Ignored => {
                    index = child.marker;
                    position = child.position;
                }
                _ => {
                    classifier.record = child.record;
                    break;
                }
            }
        }

        classifier.marker = index;
        classifier.position = position;
        classifier.consumed = consumed;
        classifier.form = Form::multiple(forms);
    }
}

#[derive(Clone)]
pub struct Repetition<'repetition, Input: Formable<'repetition>, Output: Formable<'repetition>, Failure: Formable<'repetition>> {
    pub classifier: Box<Classifier<'repetition, Input, Output, Failure>>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}

impl<'repetition, Input: Formable<'repetition>, Output: Formable<'repetition>, Failure: Formable<'repetition>> Order<'repetition, Input, Output, Failure> for Repetition<'repetition, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Former<'_, 'repetition, Input, Output, Failure>, classifier: &mut Classifier<'repetition, Input, Output, Failure>) {
        let mut index = classifier.marker;
        let mut position = classifier.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::new();

        while composer.source.peek_ahead(index).is_some() {
            let mut child = Classifier {
                order: self.classifier.order.clone(),
                marker: index,
                position,
                consumed: Vec::new(),
                record: Record::Blank,
                form: Form::Blank,
                depth: classifier.depth + 1,
            };
            composer.build(&mut child);

            if child.marker == index {
                break;
            }

            if self.persist {
                match child.record {
                    Record::Panicked => {
                        index = child.marker;
                        position = child.position;
                        consumed.extend(child.consumed);
                        forms.push(child.form);
                    }

                    Record::Aligned => {
                        index = child.marker;
                        position = child.position;
                        consumed.extend(child.consumed);
                        forms.push(child.form);
                    }

                    Record::Failed => {
                        index = child.marker;
                        position = child.position;
                        consumed.extend(child.consumed);
                        forms.push(child.form);
                    }

                    Record::Ignored => {
                        index = child.marker;
                        position = child.position;
                    }

                    _ => {}
                }
            } else {
                match child.record {
                    Record::Panicked => {
                        classifier.record = child.record;
                        index = child.marker;
                        position = child.position;
                        consumed.extend(child.consumed);
                        forms.push(child.form);
                        break;
                    }

                    Record::Aligned => {
                        classifier.record = child.record;
                        index = child.marker;
                        position = child.position;
                        consumed.extend(child.consumed);
                        forms.push(child.form);
                    }

                    Record::Failed => {
                        classifier.record = child.record;
                        index = child.marker;
                        position = child.position;
                        consumed.extend(child.consumed);
                        forms.push(child.form);
                        break;
                    }

                    Record::Ignored => {
                        index = child.marker;
                        position = child.position;
                    }

                    _ => {}
                }
            }

            if let Some(max) = self.maximum {
                if forms.len() >= max {
                    break;
                }
            }
        }

        if forms.len() >= self.minimum {
            if self.persist {
                classifier.set_align();
            }
            classifier.marker = index;
            classifier.position = position;
            classifier.consumed = consumed;
            classifier.form = Form::multiple(forms);
        } else {
            if self.persist {
                classifier.set_empty();
            }
        }
    }
}