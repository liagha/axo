use {
    super::{
        form::Form,
        former::{record::Record, Former},
        helper::Formable,
        order::*,
    },
    crate::{
        data::{
            memory::take,
            sync::Rc,
            Boolean, Offset, Scale,
        },
        tracker::{Location, Position},
    },
};

pub struct Classifier<
    'classifier,
    Input: Formable<'classifier>,
    Output: Formable<'classifier>,
    Failure: Formable<'classifier>,
> {
    pub order: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    pub marker: Offset,
    pub position: Position<'classifier>,
    pub consumed: Vec<usize>,
    pub record: Record,
    pub form: usize,
    pub stack: Vec<usize>,
    pub depth: Scale,
}

impl<
    'classifier,
    Input: Formable<'classifier>,
    Output: Formable<'classifier>,
    Failure: Formable<'classifier>,
> Classifier<'classifier, Input, Output, Failure>
{
    #[inline]
    pub fn new(
        classifier: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
        marker: Offset,
        position: Position<'classifier>,
    ) -> Self {
        Self {
            order: classifier,
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
        classifier: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
        marker: Offset,
        position: Position<'classifier>,
        depth: Scale,
    ) -> Self {
        Self {
            order: classifier,
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
    fn create_child(
        &mut self,
        order: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    ) -> Self {
        Self {
            order,
            marker: self.marker,
            position: self.position,
            consumed: Vec::new(),
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
    pub fn literal(value: impl PartialEq<Input> + 'classifier) -> Self {
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
        F: Fn(&Input) -> bool + 'classifier,
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
                perfection: vec![Record::Panicked, Record::Aligned],
                blacklist: vec![Record::Blank],
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
    pub fn deferred<F>(factory: F) -> Self
    where
        F: Fn() -> Self + 'classifier,
    {
        Self::new(
            Rc::new(Deferred {
                function: Rc::new(factory),
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
    pub fn with_order(
        mut self,
        order: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    ) -> Self {
        let orders = vec![self.order.clone(), order];
        let multiple: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> =
            Rc::new(Multiple { orders });

        self.order = multiple;
        self
    }

    #[inline]
    pub fn with_align(self) -> Self {
        self.with_order(Rc::new(Align))
    }

    #[inline]
    pub fn with_branch(
        self,
        found: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
        missing: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    ) -> Self {
        let branch: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> =
            Rc::new(Branch { found, missing });

        self.with_order(branch)
    }

    #[inline]
    pub fn with_fail<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Former<'_, 'classifier, Input, Output, Failure>, Classifier<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
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
            Classifier<'classifier, Input, Output, Failure>,
        ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
        + 'classifier,
    {
        self.with_order(Rc::new(Inspect {
            inspector: Rc::new(inspector),
        }))
    }

    #[inline]
    pub fn with_multiple(
        self,
        orders: Vec<Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>>,
    ) -> Self {
        let multiple: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> =
            Rc::new(Multiple { orders });

        self.with_order(multiple)
    }

    #[inline]
    pub fn with_panic<F>(self, emitter: F) -> Self
    where
        F: Fn(&mut Former<'_, 'classifier, Input, Output, Failure>, Classifier<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
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
        F: Fn() + 'classifier,
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
            &mut Former<'_, 'classifier, Input, Output, Failure>,
            &mut Classifier<'classifier, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + 'classifier,
    {
        self.with_order(Self::transform(transform))
    }

    #[inline]
    pub fn with_fallback(
        self,
        order: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    ) -> Self {
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
    pub fn transform<T>(
        transformer: T,
    ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(
            &mut Former<'_, 'classifier, Input, Output, Failure>,
            &mut Classifier<'classifier, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + 'classifier,
    {
        Rc::new(Transform {
            transformer: Rc::new(transformer),
        })
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(&mut Former<'_, 'classifier, Input, Output, Failure>, Classifier<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
    {
        Rc::new(Fail {
            emitter: Rc::new(emitter),
        })
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(&mut Former<'_, 'classifier, Input, Output, Failure>, Classifier<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
    {
        Rc::new(Panic {
            emitter: Rc::new(emitter),
        })
    }

    #[inline]
    pub fn ignore() -> Rc<dyn Order<'classifier, Input, Output, Failure>> {
        Rc::new(Ignore)
    }

    #[inline]
    pub fn inspect<T>(
        inspector: T,
    ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(
            Classifier<'classifier, Input, Output, Failure>,
        ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
        + 'classifier,
    {
        Rc::new(Inspect {
            inspector: Rc::new(inspector),
        })
    }

    #[inline]
    pub fn multiple(
        orders: Vec<Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>>,
    ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Rc::new(Multiple { orders })
    }

    #[inline]
    pub fn pardon() -> Rc<dyn Order<'classifier, Input, Output, Failure>> {
        Rc::new(Pardon)
    }

    #[inline]
    pub fn perform<T>(
        executor: T,
    ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn() + 'classifier,
    {
        Rc::new(Perform {
            performer: Rc::new(executor),
        })
    }

    #[inline]
    pub fn skip() -> Rc<dyn Order<'classifier, Input, Output, Failure>> {
        Rc::new(Skip)
    }

    #[inline]
    pub fn branch(
        found: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
        missing: Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>,
    ) -> Rc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Rc::new(Branch { found, missing })
    }
}

#[derive(Clone)]
pub struct Literal<'literal, Input> {
    pub value: Rc<dyn PartialEq<Input> + 'literal>,
}

impl<
    'literal,
    Input: Formable<'literal>,
    Output: Formable<'literal>,
    Failure: Formable<'literal>,
> Order<'literal, Input, Output, Failure> for Literal<'literal, Input>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'literal, Input, Output, Failure>,
        classifier: &mut Classifier<'literal, Input, Output, Failure>,
    ) {
        if let Some(peek) = former.source.get(classifier.marker).cloned() {
            if self.value.eq(&peek) {
                classifier.set_align();
                former
                    .source
                    .next(&mut classifier.marker, &mut classifier.position);

                let consumed_id = former.consumed.len();
                former.consumed.push(peek.clone());
                classifier.consumed.push(consumed_id);

                let form_id = former.forms.len();
                former.forms.push(Form::input(peek));
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
pub struct Negate<
    'negate,
    Input: Formable<'negate>,
    Output: Formable<'negate>,
    Failure: Formable<'negate>,
> {
    pub classifier: Box<Classifier<'negate, Input, Output, Failure>>,
}

impl<'negate, Input: Formable<'negate>, Output: Formable<'negate>, Failure: Formable<'negate>>
Order<'negate, Input, Output, Failure> for Negate<'negate, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'negate, Input, Output, Failure>,
        classifier: &mut Classifier<'negate, Input, Output, Failure>,
    ) {
        let checkpoint_consumed = former.consumed.len();
        let checkpoint_forms = former.forms.len();

        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        // Negate is a lookahead assertion; completely rollback all arena usage.
        former.consumed.truncate(checkpoint_consumed);
        former.forms.truncate(checkpoint_forms);

        if child.is_aligned() {
            classifier.set_empty();
        } else if child.is_effected() {
            classifier.set_align();
            classifier.form = 0;
        } else {
            classifier.set_empty();
        }
    }
}

#[derive(Clone)]
pub struct Predicate<'predicate, Input: Formable<'predicate>> {
    pub function: Rc<dyn Fn(&Input) -> bool + 'predicate>,
}

impl<
    'predicate,
    Input: Formable<'predicate>,
    Output: Formable<'predicate>,
    Failure: Formable<'predicate>,
> Order<'predicate, Input, Output, Failure> for Predicate<'predicate, Input>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'predicate, Input, Output, Failure>,
        classifier: &mut Classifier<'predicate, Input, Output, Failure>,
    ) {
        if let Some(peek) = former.source.get(classifier.marker) {
            if (self.function)(peek) {
                let value = peek.clone();
                classifier.set_align();
                former
                    .source
                    .next(&mut classifier.marker, &mut classifier.position);

                let consumed_id = former.consumed.len();
                former.consumed.push(value.clone());
                classifier.consumed.push(consumed_id);

                let form_id = former.forms.len();
                former.forms.push(Form::input(value));
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
    'alternative,
    Input: Formable<'alternative>,
    Output: Formable<'alternative>,
    Failure: Formable<'alternative>,
    const SIZE: Scale,
> {
    pub patterns: [Classifier<'alternative, Input, Output, Failure>; SIZE],
    pub perfection: Vec<Record>,
    pub blacklist: Vec<Record>,
}

impl<'alternative, Input, Output, Failure, const SIZE: Scale> Order<'alternative, Input, Output, Failure>
for Alternative<'alternative, Input, Output, Failure, SIZE>
where
    Input: Formable<'alternative>,
    Output: Formable<'alternative>,
    Failure: Formable<'alternative>,
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'alternative, Input, Output, Failure>,
        classifier: &mut Classifier<'alternative, Input, Output, Failure>,
    ) {
        let mut best: Option<Classifier<'alternative, Input, Output, Failure>> = None;
        let initial_stack_len = classifier.stack.len();
        let mut current_stack = take(&mut classifier.stack);

        let mut current_consumed_len = former.consumed.len();
        let mut current_forms_len = former.forms.len();

        for pattern in &self.patterns {
            let mut child = Classifier {
                order: pattern.order.clone(),
                marker: classifier.marker,
                position: classifier.position,
                consumed: Vec::new(),
                record: Record::Blank,
                form: 0,
                stack: current_stack,
                depth: classifier.depth + 1,
            };

            former.build(&mut child);

            if self.blacklist.contains(&child.record) {
                current_stack = take(&mut child.stack);
                current_stack.truncate(initial_stack_len);
                former.consumed.truncate(current_consumed_len);
                former.forms.truncate(current_forms_len);
                continue;
            }

            if let Some(ref mut champion) = best {
                if child.is_aligned() && (champion.is_failed() || child.marker > champion.marker) {
                    *champion = child;
                    current_stack = take(&mut champion.stack);
                    current_consumed_len = former.consumed.len();
                    current_forms_len = former.forms.len();
                } else {
                    current_stack = take(&mut child.stack);
                    current_stack.truncate(initial_stack_len);
                    former.consumed.truncate(current_consumed_len);
                    former.forms.truncate(current_forms_len);
                }
            } else {
                best = Some(child);
                current_stack = take(&mut best.as_mut().unwrap().stack);
                current_consumed_len = former.consumed.len();
                current_forms_len = former.forms.len();
            }

            if let Some(ref champion) = best {
                if self.perfection.contains(&champion.record) {
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
                classifier.stack = current_stack;
            }
            None => {
                classifier.set_empty();
                classifier.stack = current_stack;
            }
        }
    }
}

#[derive(Clone)]
pub struct Deferred<
    'deferred,
    Input: Formable<'deferred>,
    Output: Formable<'deferred>,
    Failure: Formable<'deferred>,
> {
    pub function: Rc<dyn Fn() -> Classifier<'deferred, Input, Output, Failure> + 'deferred>,
}

impl<
    'deferred,
    Input: Formable<'deferred>,
    Output: Formable<'deferred>,
    Failure: Formable<'deferred>,
> Order<'deferred, Input, Output, Failure> for Deferred<'deferred, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'deferred, Input, Output, Failure>,
        classifier: &mut Classifier<'deferred, Input, Output, Failure>,
    ) {
        let mut resolved = (self.function)();
        resolved.marker = classifier.marker;
        resolved.position = classifier.position;
        resolved.depth = classifier.depth + 1;
        resolved.stack = take(&mut classifier.stack);
        former.build(&mut resolved);

        classifier.marker = resolved.marker;
        classifier.position = resolved.position;
        classifier.consumed = take(&mut resolved.consumed);
        classifier.record = resolved.record;
        classifier.form = resolved.form;
        classifier.stack = take(&mut resolved.stack);
    }
}

#[derive(Clone)]
pub struct Optional<
    'optional,
    Input: Formable<'optional>,
    Output: Formable<'optional>,
    Failure: Formable<'optional>,
> {
    pub classifier: Box<Classifier<'optional, Input, Output, Failure>>,
}

impl<
    'optional,
    Input: Formable<'optional>,
    Output: Formable<'optional>,
    Failure: Formable<'optional>,
> Order<'optional, Input, Output, Failure> for Optional<'optional, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'optional, Input, Output, Failure>,
        classifier: &mut Classifier<'optional, Input, Output, Failure>,
    ) {
        let checkpoint_consumed = former.consumed.len();
        let checkpoint_forms = former.forms.len();

        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        if child.is_effected() {
            classifier.marker = child.marker;
            classifier.position = child.position;
            classifier.consumed = take(&mut child.consumed);
            classifier.form = child.form;
            classifier.stack = take(&mut child.stack);
            classifier.set_align();
        } else {
            former.consumed.truncate(checkpoint_consumed);
            former.forms.truncate(checkpoint_forms);
            classifier.set_ignore();
        }
    }
}

#[derive(Clone)]
pub struct Wrapper<
    'wrapper,
    Input: Formable<'wrapper>,
    Output: Formable<'wrapper>,
    Failure: Formable<'wrapper>,
> {
    pub classifier: Box<Classifier<'wrapper, Input, Output, Failure>>,
}

impl<
    'wrapper,
    Input: Formable<'wrapper>,
    Output: Formable<'wrapper>,
    Failure: Formable<'wrapper>,
> Order<'wrapper, Input, Output, Failure> for Wrapper<'wrapper, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'wrapper, Input, Output, Failure>,
        classifier: &mut Classifier<'wrapper, Input, Output, Failure>,
    ) {
        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = take(&mut child.consumed);
        classifier.record = child.record;
        classifier.form = child.form;
        classifier.stack = take(&mut child.stack);
    }
}

#[derive(Clone)]
pub struct Ranked<
    'ranked,
    Input: Formable<'ranked>,
    Output: Formable<'ranked>,
    Failure: Formable<'ranked>,
> {
    pub classifier: Box<Classifier<'ranked, Input, Output, Failure>>,
    pub precedence: i8,
}

impl<'ranked, Input: Formable<'ranked>, Output: Formable<'ranked>, Failure: Formable<'ranked>>
Order<'ranked, Input, Output, Failure> for Ranked<'ranked, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'ranked, Input, Output, Failure>,
        classifier: &mut Classifier<'ranked, Input, Output, Failure>,
    ) {
        let mut child = classifier.create_child(self.classifier.order.clone());
        former.build(&mut child);

        classifier.marker = child.marker;
        classifier.position = child.position;
        classifier.consumed = take(&mut child.consumed);
        classifier.form = child.form;
        classifier.stack = take(&mut child.stack);

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
pub struct Sequence<
    'sequence,
    Input: Formable<'sequence>,
    Output: Formable<'sequence>,
    Failure: Formable<'sequence>,
    const SIZE: Scale,
> {
    pub patterns: [Classifier<'sequence, Input, Output, Failure>; SIZE],
}

impl<
    'sequence,
    Input: Formable<'sequence>,
    Output: Formable<'sequence>,
    Failure: Formable<'sequence>,
    const SIZE: Scale,
> Order<'sequence, Input, Output, Failure>
for Sequence<'sequence, Input, Output, Failure, SIZE>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'sequence, Input, Output, Failure>,
        classifier: &mut Classifier<'sequence, Input, Output, Failure>,
    ) {
        let mut index = classifier.marker;
        let mut position = classifier.position;
        let mut consumed = take(&mut classifier.consumed);
        let mut forms = Vec::with_capacity(SIZE);

        let initial_consumed_len = former.consumed.len();
        let initial_forms_len = former.forms.len();

        let mut stack = take(&mut classifier.stack);
        let mut broke_on_blank = false;

        for pattern in &self.patterns {
            let mut child = Classifier {
                order: pattern.order.clone(),
                marker: index,
                position,
                consumed: Vec::new(),
                record: Record::Blank,
                form: 0,
                stack,
                depth: classifier.depth + 1,
            };

            former.build(&mut child);

            match child.record {
                Record::Aligned => {
                    classifier.record = child.record;
                    index = child.marker;
                    position = child.position;
                    consumed.extend(take(&mut child.consumed));
                    forms.push(child.form);
                    stack = take(&mut child.stack);
                }
                Record::Panicked | Record::Failed => {
                    classifier.record = child.record;
                    index = child.marker;
                    position = child.position;
                    consumed.extend(take(&mut child.consumed));
                    forms.push(child.form);
                    stack = take(&mut child.stack);
                    break;
                }
                Record::Ignored => {
                    index = child.marker;
                    position = child.position;
                    stack = take(&mut child.stack);
                }
                _ => {
                    classifier.record = child.record;
                    stack = take(&mut child.stack);
                    broke_on_blank = true;
                    break;
                }
            }
        }

        if broke_on_blank {
            // Unsuccessful sequence rollback cleans up partial progress efficiently
            former.consumed.truncate(initial_consumed_len);
            former.forms.truncate(initial_forms_len);
            classifier.stack = stack;
        } else {
            classifier.marker = index;
            classifier.position = position;
            classifier.consumed = consumed;

            let multi_form = Form::multiple(forms.into_iter().map(|id| former.forms[id].clone()).collect());
            let form_id = former.forms.len();
            former.forms.push(multi_form);
            classifier.form = form_id;

            classifier.stack = stack;
        }
    }
}

#[derive(Clone)]
pub struct Repetition<
    'repetition,
    Input: Formable<'repetition>,
    Output: Formable<'repetition>,
    Failure: Formable<'repetition>,
> {
    pub classifier: Box<Classifier<'repetition, Input, Output, Failure>>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}

impl<
    'repetition,
    Input: Formable<'repetition>,
    Output: Formable<'repetition>,
    Failure: Formable<'repetition>,
> Order<'repetition, Input, Output, Failure>
for Repetition<'repetition, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'repetition, Input, Output, Failure>,
        classifier: &mut Classifier<'repetition, Input, Output, Failure>,
    ) {
        let mut index = classifier.marker;
        let mut position = classifier.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::new();

        let initial_consumed_len = former.consumed.len();
        let initial_forms_len = former.forms.len();

        let mut stack = take(&mut classifier.stack);

        while former.source.peek_ahead(index).is_some() {
            let loop_consumed_len = former.consumed.len();
            let loop_forms_len = former.forms.len();

            let mut child = Classifier {
                order: self.classifier.order.clone(),
                marker: index,
                position,
                consumed: Vec::new(),
                record: Record::Blank,
                form: 0,
                stack,
                depth: classifier.depth + 1,
            };

            former.build(&mut child);

            if child.marker == index {
                former.consumed.truncate(loop_consumed_len);
                former.forms.truncate(loop_forms_len);
                stack = take(&mut child.stack);
                break;
            }

            if self.persist {
                match child.record {
                    Record::Panicked | Record::Aligned | Record::Failed => {
                        index = child.marker;
                        position = child.position;
                        consumed.extend(take(&mut child.consumed));
                        forms.push(child.form);
                        stack = take(&mut child.stack);
                    }
                    Record::Ignored => {
                        former.consumed.truncate(loop_consumed_len);
                        former.forms.truncate(loop_forms_len);
                        index = child.marker;
                        position = child.position;
                        stack = take(&mut child.stack);
                    }
                    _ => {
                        former.consumed.truncate(loop_consumed_len);
                        former.forms.truncate(loop_forms_len);
                        stack = take(&mut child.stack);
                    }
                }
            } else {
                match child.record {
                    Record::Panicked | Record::Failed => {
                        classifier.record = child.record;
                        index = child.marker;
                        position = child.position;
                        consumed.extend(take(&mut child.consumed));
                        forms.push(child.form);
                        stack = take(&mut child.stack);
                        break;
                    }
                    Record::Aligned => {
                        classifier.record = child.record;
                        index = child.marker;
                        position = child.position;
                        consumed.extend(take(&mut child.consumed));
                        forms.push(child.form);
                        stack = take(&mut child.stack);
                    }
                    Record::Ignored => {
                        former.consumed.truncate(loop_consumed_len);
                        former.forms.truncate(loop_forms_len);
                        index = child.marker;
                        position = child.position;
                        stack = take(&mut child.stack);
                    }
                    _ => {
                        former.consumed.truncate(loop_consumed_len);
                        former.forms.truncate(loop_forms_len);
                        stack = take(&mut child.stack);
                    }
                }
            }

            if let Some(max) = self.maximum {
                if forms.len() >= max as usize {
                    break;
                }
            }
        }

        if forms.len() >= self.minimum as usize {
            if self.persist {
                classifier.set_align();
            }
            classifier.marker = index;
            classifier.position = position;
            classifier.consumed = consumed;

            let multi_form = Form::multiple(forms.into_iter().map(|id| former.forms[id].clone()).collect());
            let form_id = former.forms.len();
            former.forms.push(multi_form);
            classifier.form = form_id;

            classifier.stack = stack;
        } else {
            // Failed the minimum requirement, rollback ALL progress made inside loop
            former.consumed.truncate(initial_consumed_len);
            former.forms.truncate(initial_forms_len);

            if self.persist {
                classifier.set_empty();
            }
            classifier.stack = stack;
        }
    }
}
