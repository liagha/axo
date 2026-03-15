use {
    super::{
        form::Form,
        former::{record::Record, Former},
        helper::Formable,
        order::*,
    },
    crate::{
        data::{memory::take, Boolean, Offset, Scale},
        tracker::{Location, Position},
    },
};

pub struct Classifier<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub order: &'a dyn Order<'a, Input, Output, Failure>,
    pub marker: Offset,
    pub position: Position<'a>,
    pub consumed: Vec<usize>,
    pub record: Record,
    pub form: usize,
    pub stack: Vec<usize>,
    pub depth: Scale,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Classifier<'a, Input, Output, Failure>
{
    #[inline]
    pub fn new(
        order: &'a dyn Order<'a, Input, Output, Failure>,
        marker: Offset,
        position: Position<'a>,
    ) -> Self {
        Self {
            order,
            marker,
            position,
            consumed: Vec::new(),
            record: Record::Blank,
            form: 0,
            stack: Vec::new(),
            depth: 0,
        }
    }

    #[inline]
    pub fn new_with_depth(
        order: &'a dyn Order<'a, Input, Output, Failure>,
        marker: Offset,
        position: Position<'a>,
        depth: Scale,
    ) -> Self {
        Self {
            order,
            marker,
            position,
            consumed: Vec::new(),
            record: Record::Blank,
            form: 0,
            stack: Vec::new(),
            depth,
        }
    }

    #[inline]
    fn create_child(&mut self, order: &'a dyn Order<'a, Input, Output, Failure>) -> Self {
        Self {
            order,
            marker: self.marker,
            position: self.position,
            consumed: take(&mut self.consumed),
            record: Record::Blank,
            form: 0,
            stack: take(&mut self.stack),
            depth: self.depth + 1,
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
    pub fn literal(value: impl PartialEq<Input> + 'a) -> Self {
        Self::new(
            Box::leak(Box::new(Literal {
                value: Box::leak(Box::new(value)),
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn negate(classifier: Self) -> Self {
        Self::new(
            Box::leak(Box::new(Negate {
                classifier: Box::new(classifier),
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + 'a,
    {
        Self::new(
            Box::leak(Box::new(Predicate::<Input> {
                function: Box::leak(Box::new(predicate)),
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Box::leak(Box::new(Alternative {
                patterns,
                targets: vec![Record::Panicked, Record::Aligned],
                rejects: vec![Record::Blank],
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Box::leak(Box::new(Sequence { patterns })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn optional(classifier: Self) -> Self {
        Self::new(
            Box::leak(Box::new(Optional {
                classifier: Box::new(classifier),
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn persistence(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Box::leak(Box::new(Repetition {
                classifier: Box::new(classifier),
                minimum,
                maximum,
                persist: true,
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn repetition(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Box::leak(Box::new(Repetition {
                classifier: Box::new(classifier),
                minimum,
                maximum,
                persist: false,
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn wrapper(classifier: Self) -> Self {
        Self::new(
            Box::leak(Box::new(Wrapper {
                classifier: Box::new(classifier),
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn ranked(classifier: Self, precedence: i8) -> Self {
        Self::new(
            Box::leak(Box::new(Ranked {
                classifier: Box::new(classifier),
                precedence,
            })),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn deferred<F>(factory: F) -> Self
    where
        F: Fn() -> Self + 'a,
    {
        Self::new(
            Box::leak(Box::new(Deferred {
                function: Box::leak(Box::new(factory)),
            })),
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
    pub fn with_order(mut self, order: &'a dyn Order<'a, Input, Output, Failure>) -> Self {
        let orders = vec![self.order, order];
        let multiple: &'a dyn Order<'a, Input, Output, Failure> =
            Box::leak(Box::new(Multiple { orders }));

        self.order = multiple;
        self
    }

    #[inline]
    pub fn with_align(self) -> Self {
        self.with_order(Box::leak(Box::new(Align)))
    }

    #[inline]
    pub fn with_branch(
        self,
        found: &'a dyn Order<'a, Input, Output, Failure>,
        missing: &'a dyn Order<'a, Input, Output, Failure>,
    ) -> Self {
        let branch: &'a dyn Order<'a, Input, Output, Failure> =
            Box::leak(Box::new(Branch { found, missing }));

        self.with_order(branch)
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        self.with_order(Box::leak(Box::new(Fail {
            emitter: Box::leak(Box::new(emitter)),
        })))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.with_order(Box::leak(Box::new(Ignore)))
    }

    #[inline]
    pub fn with_inspect<I>(self, inspector: I) -> Self
    where
        I: Fn(
            Classifier<'a, Input, Output, Failure>,
        ) -> &'a (dyn Order<'a, Input, Output, Failure>
        + 'a) + 'a,
    {
        self.with_order(Box::leak(Box::new(Inspect {
            inspector: Box::leak(Box::new(inspector)),
        })))
    }

    #[inline]
    pub fn with_multiple(self, orders: Vec<&'a dyn Order<'a, Input, Output, Failure>>) -> Self {
        let multiple: &'a dyn Order<'a, Input, Output, Failure> =
            Box::leak(Box::new(Multiple { orders }));

        self.with_order(multiple)
    }

    #[inline]
    pub fn with_panic<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        self.with_order(Self::panic(emitter))
    }

    #[inline]
    pub fn with_pardon(self) -> Self {
        self.with_order(Box::leak(Box::new(Pardon)))
    }

    #[inline]
    pub fn with_perform<F>(self, executor: F) -> Self
    where
        F: Fn() + 'a,
    {
        self.with_order(Self::perform(executor))
    }

    #[inline]
    pub fn with_skip(self) -> Self {
        self.with_order(Box::leak(Box::new(Skip)))
    }

    #[inline]
    pub fn with_transform<T>(self, transform: T) -> Self
    where
        T: Fn(
            &mut Former<'_, 'a, Input, Output, Failure>,
            &mut Classifier<'a, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + 'a,
    {
        self.with_order(Self::transform(transform))
    }

    #[inline]
    pub fn with_fallback(self, order: &'a dyn Order<'a, Input, Output, Failure>) -> Self {
        self.with_branch(Self::perform(|| {}), order)
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
    pub fn transform<T>(transformer: T) -> &'a dyn Order<'a, Input, Output, Failure>
    where
        T: Fn(
            &mut Former<'_, 'a, Input, Output, Failure>,
            &mut Classifier<'a, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + 'a,
    {
        Box::leak(Box::new(Transform {
            transformer: Box::leak(Box::new(transformer)),
        }))
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> &'a dyn Order<'a, Input, Output, Failure>
    where
        T: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        Box::leak(Box::new(Fail {
            emitter: Box::leak(Box::new(emitter)),
        }))
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> &'a dyn Order<'a, Input, Output, Failure>
    where
        T: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        Box::leak(Box::new(Panic {
            emitter: Box::leak(Box::new(emitter)),
        }))
    }

    #[inline]
    pub fn ignore() -> &'a dyn Order<'a, Input, Output, Failure> {
        Box::leak(Box::new(Ignore))
    }

    pub fn inspect<I>(&self, inspector: I) -> Self
    where
        I: Fn(Classifier<'a, Input, Output, Failure>) -> &'a (dyn Order<'a, Input, Output, Failure> + 'a) + 'a,
    {
        let mut next = self.clone();
        next.order = Box::leak(Box::new(Inspect {
            inspector: Box::leak(Box::new(inspector)),
        }));
        next
    }

    #[inline]
    pub fn multiple(
        orders: Vec<&'a dyn Order<'a, Input, Output, Failure>>,
    ) -> &'a dyn Order<'a, Input, Output, Failure> {
        Box::leak(Box::new(Multiple { orders }))
    }

    #[inline]
    pub fn pardon() -> &'a dyn Order<'a, Input, Output, Failure> {
        Box::leak(Box::new(Pardon))
    }

    #[inline]
    pub fn perform<T>(executor: T) -> &'a dyn Order<'a, Input, Output, Failure>
    where
        T: Fn() + 'a,
    {
        Box::leak(Box::new(Perform {
            performer: Box::leak(Box::new(executor)),
        }))
    }

    #[inline]
    pub fn skip() -> &'a dyn Order<'a, Input, Output, Failure> {
        Box::leak(Box::new(Skip))
    }

    #[inline]
    pub fn branch(
        found: &'a dyn Order<'a, Input, Output, Failure>,
        missing: &'a dyn Order<'a, Input, Output, Failure>,
    ) -> &'a dyn Order<'a, Input, Output, Failure> {
        Box::leak(Box::new(Branch { found, missing }))
    }
}

#[derive(Clone)]
pub struct Literal<'a, Input> {
    pub value: &'a dyn PartialEq<Input>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Literal<'a, Input>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
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

#[derive(Clone)]
pub struct Negate<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Negate<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let form_used = former.consumed.len();
        let form_forms = former.forms.len();
        let class_used = classifier.consumed.len();
        let class_stack = classifier.stack.len();

        let mut child = classifier.create_child(self.classifier.order);
        former.build(&mut child);

        let record = child.record;

        classifier.consumed = child.consumed;
        classifier.stack = child.stack;
        classifier.consumed.truncate(class_used);
        classifier.stack.truncate(class_stack);

        former.consumed.truncate(form_used);
        former.forms.truncate(form_forms);

        if record == Record::Aligned {
            classifier.set_empty();
        } else if record == Record::Failed {
            classifier.set_align();
            classifier.form = 0;
        } else {
            classifier.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Predicate<'a, Input: Formable<'a>> {
    pub function: &'a dyn Fn(&Input) -> bool,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Predicate<'a, Input>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
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

#[derive(Clone)]
pub struct Alternative<
    'a,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
    const SIZE: Scale,
> {
    pub patterns: [Classifier<'a, Input, Output, Failure>; SIZE],
    pub targets: Vec<Record>,
    pub rejects: Vec<Record>,
}

impl<'a, Input, Output, Failure, const SIZE: Scale> Order<'a, Input, Output, Failure>
for Alternative<'a, Input, Output, Failure, SIZE>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let mut best: Option<Classifier<'a, Input, Output, Failure>> = None;

        let mut stack = take(&mut classifier.stack);
        let mut consumed = take(&mut classifier.consumed);
        let base_stack = stack.len();
        let base_consumed = consumed.len();

        let mut form_used = former.consumed.len();
        let mut form_forms = former.forms.len();

        for pattern in &self.patterns {
            let mut child = Classifier {
                order: pattern.order,
                marker: classifier.marker,
                position: classifier.position,
                consumed,
                record: Record::Blank,
                form: 0,
                stack,
                depth: classifier.depth + 1,
            };

            former.build(&mut child);

            if self.rejects.contains(&child.record) {
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
                    std::mem::swap(champion, &mut child);
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
                if self.targets.contains(&champion.record) {
                    break;
                }
            }
        }

        match best {
            Some(mut champion) => {
                classifier.record = champion.record;
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

#[derive(Clone)]
pub struct Deferred<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub function: &'a dyn Fn() -> Classifier<'a, Input, Output, Failure>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Deferred<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let mut target = (self.function)();
        target.marker = classifier.marker;
        target.position = classifier.position;
        target.depth = classifier.depth + 1;
        target.consumed = take(&mut classifier.consumed);
        target.stack = take(&mut classifier.stack);
        former.build(&mut target);

        classifier.marker = target.marker;
        classifier.position = target.position;
        classifier.consumed = target.consumed;
        classifier.record = target.record;
        classifier.form = target.form;
        classifier.stack = target.stack;
    }
}

#[derive(Clone)]
pub struct Optional<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Optional<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let form_used = former.consumed.len();
        let form_forms = former.forms.len();
        let class_used = classifier.consumed.len();
        let class_stack = classifier.stack.len();

        let mut child = classifier.create_child(self.classifier.order);
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

#[derive(Clone)]
pub struct Wrapper<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Wrapper<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let mut child = classifier.create_child(self.classifier.order);
        former.build(&mut child);

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.record = child.record;
        classifier.form = child.form;
        classifier.stack = child.stack;
    }
}

#[derive(Clone)]
pub struct Ranked<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
    pub precedence: i8,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Ranked<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let mut child = classifier.create_child(self.classifier.order);
        former.build(&mut child);

        let record = child.record;

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.form = child.form;
        classifier.stack = child.stack;

        if record == Record::Aligned {
            classifier.record = self.precedence.max(Record::Aligned.into()).into();
        } else if record == Record::Failed {
            classifier.record = self.precedence.min(Record::Failed.into()).into();
        } else {
            classifier.record = child.record;
        }
    }
}

#[derive(Clone)]
pub struct Sequence<
    'a,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
    const SIZE: Scale,
> {
    pub patterns: [Classifier<'a, Input, Output, Failure>; SIZE],
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>, const SIZE: Scale>
Order<'a, Input, Output, Failure> for Sequence<'a, Input, Output, Failure, SIZE>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
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

        for pattern in &self.patterns {
            let mut child = Classifier {
                order: pattern.order,
                marker: mark,
                position: pos,
                consumed,
                record: Record::Blank,
                form: 0,
                stack,
                depth: classifier.depth + 1,
            };

            former.build(&mut child);

            consumed = child.consumed;
            stack = child.stack;

            match child.record {
                Record::Aligned => {
                    classifier.record = child.record;
                    mark = child.marker;
                    pos = child.position;
                    forms.push(child.form);
                }
                Record::Panicked | Record::Failed => {
                    classifier.record = child.record;
                    mark = child.marker;
                    pos = child.position;
                    forms.push(child.form);
                    break;
                }
                Record::Ignored => {
                    mark = child.marker;
                    pos = child.position;
                }
                _ => {
                    classifier.record = child.record;
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
                    .map(|id| std::mem::replace(&mut former.forms[id], Form::Blank))
                    .collect(),
            );

            let form_id = former.forms.len();
            former.forms.push(group);
            classifier.form = form_id;
        }
    }
}

#[derive(Clone)]
pub struct Repetition<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Repetition<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
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

            let mut child = Classifier {
                order: self.classifier.order,
                marker: mark,
                position: pos,
                consumed,
                record: Record::Blank,
                form: 0,
                stack,
                depth: classifier.depth + 1,
            };

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
                match child.record {
                    Record::Panicked | Record::Aligned | Record::Failed => {
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                    }
                    Record::Ignored => {
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
                match child.record {
                    Record::Panicked | Record::Failed => {
                        classifier.record = child.record;
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                        break;
                    }
                    Record::Aligned => {
                        classifier.record = child.record;
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                    }
                    Record::Ignored => {
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
                if forms.len() >= max as usize {
                    break;
                }
            }
        }

        classifier.consumed = consumed;
        classifier.stack = stack;

        if forms.len() >= self.minimum as usize {
            if self.persist {
                classifier.set_align();
            }
            classifier.marker = mark;
            classifier.position = pos;

            let group = Form::multiple(
                forms
                    .into_iter()
                    .map(|id| std::mem::replace(&mut former.forms[id], Form::Blank))
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
