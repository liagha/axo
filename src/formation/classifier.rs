use {
    crate::{
        formation::{
            next_identity,
            form::Form,
            former::{status::Status, Former, Memo},
            helper::Formable,
            order::*,
        },
        data::{
            memory::{
                take, replace, swap,
                Rc
            },
            Boolean,
            Offset,
            Scale,
            Identity
        },
        tracker::{Location, Position},
    },
};

pub struct Classifier<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub identity: Identity,
    pub order: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
    pub marker: Offset,
    pub position: Position<'a>,
    pub consumed: Vec<Identity>,
    pub status: Status,
    pub form: Identity,
    pub stack: Vec<Identity>,
    pub depth: Scale,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Classifier<'a, Input, Output, Failure>
{
    #[inline]
    pub fn new(
        order: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
        marker: Offset,
        position: Position<'a>,
    ) -> Self {
        Self {
            identity: next_identity(),
            order,
            marker,
            position,
            consumed: Vec::new(),
            status: Status::Blank,
            form: 0,
            stack: Vec::new(),
            depth: 0,
        }
    }

    #[inline]
    fn create(
        order: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
        marker: Offset,
        position: Position<'a>,
        consumed: Vec<Identity>,
        status: Status,
        form: Identity,
        stack: Vec<Identity>,
        depth: Scale,
    ) -> Self {
        Self {
            identity: next_identity(),
            order,
            marker,
            position,
            consumed,
            status,
            form,
            stack,
            depth
        }
    }

    #[inline]
    fn create_child(&mut self, order: Rc<dyn Order<'a, Input, Output, Failure> + 'a>) -> Self {
        Self {
            identity: next_identity(),
            order,
            marker: self.marker,
            position: self.position,
            consumed: take(&mut self.consumed),
            status: Status::Blank,
            form: 0,
            stack: take(&mut self.stack),
            depth: self.depth + 1,
        }
    }

    #[inline]
    pub const fn is_panicked(&self) -> bool {
        matches!(self.status, Status::Panicked)
    }

    #[inline]
    pub const fn is_aligned(&self) -> bool {
        matches!(self.status, Status::Aligned)
    }

    #[inline]
    pub const fn is_failed(&self) -> bool {
        matches!(self.status, Status::Failed)
    }

    #[inline]
    pub const fn is_effected(&self) -> bool {
        matches!(self.status, Status::Aligned | Status::Failed)
    }

    #[inline]
    pub const fn is_blank(&self) -> bool {
        matches!(self.status, Status::Blank)
    }

    #[inline]
    pub const fn is_ignored(&self) -> bool {
        matches!(self.status, Status::Ignored)
    }

    #[inline]
    pub const fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    #[inline]
    pub const fn is_neutral(&self) -> bool {
        self.status.is_neutral()
    }

    #[inline]
    pub fn set_panic(&mut self) {
        self.status = Status::Panicked;
    }

    #[inline]
    pub fn set_align(&mut self) {
        self.status = Status::Aligned;
    }

    #[inline]
    pub fn set_fail(&mut self) {
        self.status = Status::Failed;
    }

    #[inline]
    pub fn set_empty(&mut self) {
        self.status = Status::Blank;
    }

    #[inline]
    pub fn set_ignore(&mut self) {
        self.status = Status::Ignored;
    }

    #[inline]
    pub fn escalate(&mut self, other: Status) {
        self.status = self.status.escalate(other);
    }

    #[inline]
    pub fn literal(value: impl PartialEq<Input> + 'a) -> Self {
        Self::new(
            Rc::new(Literal {
                value: Rc::new(value),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn negate(classifier: Self) -> Self {
        Self::new(
            Rc::new(Negate {
                classifier: Box::new(classifier),
            }),
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
            Rc::new(Predicate::<Input> {
                function: Rc::new(predicate),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Rc::new(Alternative {
                patterns,
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(patterns: [Self; SIZE]) -> Self {
        Self::new(
            Rc::new(Sequence { patterns }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn optional(classifier: Self) -> Self {
        Self::new(
            Rc::new(Optional {
                classifier: Box::new(classifier),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn persistence(classifier: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(
            Rc::new(Repetition {
                classifier: Box::new(classifier),
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
                classifier: Box::new(classifier),
                minimum,
                maximum,
                persist: false,
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn wrapper(classifier: Self) -> Self {
        Self::new(
            Rc::new(Wrapper {
                classifier: Box::new(classifier),
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn ranked(classifier: Self, precedence: i8) -> Self {
        Self::new(
            Rc::new(Ranked {
                classifier: Box::new(classifier),
                precedence,
            }),
            0,
            Position::new(Location::Void),
        )
    }

    #[inline]
    pub fn deferred(factory: fn() -> Self) -> Self {
        Self::new(
            Rc::new(Deferred {
                factory,
            }),
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
    pub fn with_order(mut self, order: Rc<dyn Order<'a, Input, Output, Failure> + 'a>) -> Self {
        let orders = vec![self.order.clone(), order];
        self.order = Rc::new(Multiple { orders });
        self
    }

    #[inline]
    pub fn with_align(self) -> Self {
        self.with_order(Rc::new(Align))
    }

    #[inline]
    pub fn with_branch(
        self,
        found: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
        missing: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
    ) -> Self {
        self.with_order(Rc::new(Branch { found, missing }))
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        self.with_order(Rc::new(Fail {
            emitter: Rc::new(emitter),
        }))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.with_order(Rc::new(Ignore))
    }

    #[inline]
    pub fn with_inspect<I>(self, inspector: I) -> Self
    where
        I: Fn(
            Classifier<'a, Input, Output, Failure>,
        ) -> &'a (dyn Order<'a, Input, Output, Failure>
        + 'a) + 'a,
    {
        self.with_order(Rc::new(Inspect {
            inspector: Rc::new(inspector) as Rc<dyn Fn(Classifier<'a, Input, Output, Failure>) -> &'a (dyn Order<'a, Input, Output, Failure> + 'a) + 'a>,
        }))
    }

    #[inline]
    pub fn with_multiple(self, orders: Vec<Rc<dyn Order<'a, Input, Output, Failure> + 'a>>) -> Self {
        self.with_order(Rc::new(Multiple { orders }))
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
        self.with_order(Rc::new(Pardon))
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
        self.with_order(Rc::new(Skip))
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
    pub fn with_fallback(self, order: Rc<dyn Order<'a, Input, Output, Failure> + 'a>) -> Self {
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
    pub fn transform<T>(transformer: T) -> Rc<dyn Order<'a, Input, Output, Failure> + 'a>
    where
        T: Fn(
            &mut Former<'_, 'a, Input, Output, Failure>,
            &mut Classifier<'a, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + 'a,
    {
        Rc::new(Transform {
            transformer: Rc::new(transformer),
        })
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> Rc<dyn Order<'a, Input, Output, Failure> + 'a>
    where
        T: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        Rc::new(Fail {
            emitter: Rc::new(emitter),
        })
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> Rc<dyn Order<'a, Input, Output, Failure> + 'a>
    where
        T: Fn(&mut Former<'_, 'a, Input, Output, Failure>, Classifier<'a, Input, Output, Failure>) -> Failure + 'a,
    {
        Rc::new(Panic {
            emitter: Rc::new(emitter),
        })
    }

    #[inline]
    pub fn ignore() -> Rc<dyn Order<'a, Input, Output, Failure> + 'a> {
        Rc::new(Ignore)
    }

    pub fn inspect<I>(&self, inspector: I) -> Self
    where
        I: Fn(Classifier<'a, Input, Output, Failure>) -> &'a (dyn Order<'a, Input, Output, Failure> + 'a) + 'a,
    {
        let mut next = self.clone();
        next.order = Rc::new(Inspect {
            inspector: Rc::new(inspector) as Rc<dyn Fn(Classifier<'a, Input, Output, Failure>) -> &'a (dyn Order<'a, Input, Output, Failure> + 'a) + 'a>,
        });
        next
    }

    #[inline]
    pub fn multiple(
        orders: Vec<Rc<dyn Order<'a, Input, Output, Failure> + 'a>>,
    ) -> Rc<dyn Order<'a, Input, Output, Failure> + 'a> {
        Rc::new(Multiple { orders })
    }

    #[inline]
    pub fn pardon() -> Rc<dyn Order<'a, Input, Output, Failure> + 'a> {
        Rc::new(Pardon)
    }

    #[inline]
    pub fn perform<T>(executor: T) -> Rc<dyn Order<'a, Input, Output, Failure> + 'a>
    where
        T: Fn() + 'a,
    {
        Rc::new(Perform {
            performer: Rc::new(executor) as Rc<dyn Fn() + 'a>,
        })
    }

    #[inline]
    pub fn skip() -> Rc<dyn Order<'a, Input, Output, Failure> + 'a> {
        Rc::new(Skip)
    }

    #[inline]
    pub fn branch(
        found: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
        missing: Rc<dyn Order<'a, Input, Output, Failure> + 'a>,
    ) -> Rc<dyn Order<'a, Input, Output, Failure> + 'a> {
        Rc::new(Branch { found, missing })
    }
}

#[derive(Clone)]
pub struct Literal<'a, Input> {
    pub value: Rc<dyn PartialEq<Input> + 'a>,
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

        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        let status = child.status;

        classifier.consumed = child.consumed;
        classifier.stack = child.stack;
        classifier.consumed.truncate(class_used);
        classifier.stack.truncate(class_stack);

        former.consumed.truncate(form_used);
        former.forms.truncate(form_forms);

        if status == Status::Aligned {
            classifier.set_empty();
        } else if status == Status::Failed {
            classifier.set_align();
            classifier.form = 0;
        } else {
            classifier.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Predicate<'a, Input: Formable<'a>> {
    pub function: Rc<dyn Fn(&Input) -> bool + 'a>,
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
        let mut child = Classifier::create(
            pattern.order.clone(),
            classifier.marker,
            classifier.position,
            consumed,
            Status::Blank,
            0,
            stack,
            classifier.depth + 1,
            );

            former.build(&mut child);

            if matches!(child.status, Status::Blank) {
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
                if matches!(champion.status, Status::Panicked | Status::Aligned) {
                    break;
                }
            }
        }

        match best {
            Some(mut champion) => {
                classifier.status = champion.status;
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

pub struct Deferred<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub factory: fn() -> Classifier<'a, Input, Output, Failure>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> Clone
for Deferred<'a, Input, Output, Failure>
{
    fn clone(&self) -> Self {
        Self {
            factory: self.factory,
        }
    }
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
                classifier.consumed.push((index as isize + input_offset) as Identity);
            }

            for &index in &entry.stack {
                let shifted = if index == 0 { 0 } else { (index as isize + form_offset) as Identity };
                classifier.stack.push(shifted);
            }

            classifier.marker = classifier.marker + entry.advance;
            classifier.position = entry.position;
            classifier.status = entry.status;
            classifier.form = if entry.form == 0 { 0 } else { (entry.form as isize + form_offset) as Identity };

            return;
        }

        let cached_order = match former.cache.iter().find(|(k, _)| *k == key) {
            Some((_, order)) => order.clone(),
            None => {
                let built = (self.factory)();
                let order = built.order;
                former.cache.push((key, order.clone()));
                order
            }
        };

        let class_consumed = classifier.consumed.len();
        let class_stack = classifier.stack.len();
        let form_base = former.forms.len();
        let input_base = former.consumed.len();
        let origin = classifier.marker;

        let mut child = Classifier::create(
            cached_order,
            classifier.marker,
            classifier.position,
            take(&mut classifier.consumed),
            Status::Blank,
            0,
            take(&mut classifier.stack),
            classifier.depth + 1,
        );

        former.build(&mut child);

        let forms: Vec<_> = former.forms[form_base..].iter().cloned().collect();
        let inputs: Vec<_> = former.consumed[input_base..].iter().cloned().collect();
        let consumed: Vec<_> = child.consumed[class_consumed..].to_vec();
        let stack: Vec<_> = child.stack[class_stack..].to_vec();

        former.memo.insert(memo_key, Memo {
            status: child.status,
            advance: child.marker - origin,
            position: child.position,
            forms,
            inputs,
            consumed,
            stack,
            form: child.form,
            form_base,
            input_base,
        });

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.status = child.status;
        classifier.form = child.form;
        classifier.stack = child.stack;
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

        let mut child = classifier.create_child(self.classifier.order.clone());
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
        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.status = child.status;
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
        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        let status = child.status;

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = child.consumed;
        classifier.form = child.form;
        classifier.stack = child.stack;

        if status == Status::Aligned {
            classifier.status = self.precedence.max(Status::Aligned.into()).into();
        } else if status == Status::Failed {
            classifier.status = self.precedence.min(Status::Failed.into()).into();
        } else {
            classifier.status = child.status;
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
            let mut child = Classifier::create(
                pattern.order.clone(),
                mark,
                pos,
                consumed,
                Status::Blank,
                0,
                stack,
                classifier.depth + 1,
            );

            former.build(&mut child);

            consumed = child.consumed;
            stack = child.stack;

            match child.status {
                Status::Aligned => {
                    classifier.status = child.status;
                    mark = child.marker;
                    pos = child.position;
                    forms.push(child.form);
                }
                Status::Panicked | Status::Failed => {
                    classifier.status = child.status;
                    mark = child.marker;
                    pos = child.position;
                    forms.push(child.form);
                    break;
                }
                Status::Ignored => {
                    mark = child.marker;
                    pos = child.position;
                }
                _ => {
                    classifier.status = child.status;
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

            let mut child = Classifier::create(
                self.classifier.order.clone(),
                mark,
                pos,
                consumed,
                Status::Blank,
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
                match child.status {
                    Status::Panicked | Status::Aligned | Status::Failed => {
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                    }
                    Status::Ignored => {
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
                match child.status {
                    Status::Panicked | Status::Failed => {
                        classifier.status = child.status;
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                        break;
                    }
                    Status::Aligned => {
                        classifier.status = child.status;
                        mark = child.marker;
                        pos = child.position;
                        forms.push(child.form);
                    }
                    Status::Ignored => {
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
