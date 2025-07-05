use {
    super::{
        order::Order,
        form::{Form},
        former::Draft,
    },
    crate::{
        artifact::Artifact,
        hash::Hash,
        format::Debug,
        compiler::Context,
        thread::{Arc, Mutex},
        axo_cursor::{
            Spanned, Peekable, Span,
        },
        compiler::Marked,
    },
};

// Create a combined trait for Source requirements
pub trait Source<Input>: Peekable<Input> + Marked
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
}

// Blanket implementation for any type that satisfies both traits
impl<T, Input> Source<Input> for T
where
    T: Peekable<Input> + Marked,
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
}

// Solution 2: Use boxed trait object for source
pub trait Pattern<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>);
}

#[derive(Clone)]
pub struct Identical<Input> {
    pub value: Arc<dyn PartialEq<Input> + Send + Sync>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Identical<Input>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        if let Some(peek) = source.get(draft.marker).cloned() {
            if self.value.eq(&peek) {
                source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.record.align();
                draft.form = Form::input(peek);
            } else {
                draft.record.empty();
            }
        } else {
            draft.record.empty();
        }
    }
}

#[derive(Clone)]
pub struct Reject<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Box<Classifier<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Reject<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        if let Some(peek) = source.get(draft.marker).cloned() {
            let mut inner_draft = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
            inner_draft.build(source);

            if !inner_draft.record.is_aligned() {
                source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.record.align();
                draft.form = Form::input(peek);
            } else {
                draft.record.empty();
            }
        } else {
            draft.record.empty();
        }
    }
}

#[derive(Clone)]
pub struct Predicate<Input> {
    pub function: Arc<dyn Fn(&Input) -> bool + Send + Sync>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Predicate<Input>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        if let Some(peek) = source.get(draft.marker).cloned() {
            let predicate = (self.function)(&peek);

            if predicate {
                source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.record.align();
                draft.form = Form::input(peek);
            } else {
                draft.record.empty();
            }
        } else {
            draft.record.empty();
        }
    }
}

#[derive(Clone)]
pub struct Alternative<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub patterns: Vec<Classifier<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Alternative<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        let mut fallback = None;

        for pattern in &self.patterns {
            let mut inner_draft = Draft::new(draft.marker, draft.position, pattern.clone());
            inner_draft.build(source);

            match inner_draft.record {
                super::former::Record::Aligned => {
                    draft.marker = inner_draft.marker;
                    draft.position = inner_draft.position;
                    draft.consumed = inner_draft.consumed;
                    draft.record.align();
                    draft.form = inner_draft.form;
                    return;
                }
                super::former::Record::Skipped => {
                    draft.marker = inner_draft.marker;
                    draft.position = inner_draft.position;
                }
                super::former::Record::Failed => {
                    if fallback.is_none() {
                        fallback = Some(inner_draft);
                    }
                }
                super::former::Record::Blank => {
                    continue;
                }
            }
        }

        if let Some(fallback) = fallback {
            draft.marker = fallback.marker;
            draft.position = fallback.position;
            draft.consumed = fallback.consumed;
            draft.record.fail();
            draft.form = fallback.form;
        } else {
            draft.record.empty();
        }
    }
}

#[derive(Clone)]
pub struct Deferred<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub function: Arc<dyn Fn() -> Classifier<Input, Output, Failure> + Send + Sync>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Deferred<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        let resolved = (self.function)();
        let mut inner_draft = Draft::new(draft.marker, draft.position, resolved);
        inner_draft.build(source);

        draft.marker = inner_draft.marker;
        draft.position = inner_draft.position;
        draft.consumed = inner_draft.consumed;
        draft.record = inner_draft.record;
        draft.form = inner_draft.form;
    }
}

#[derive(Clone)]
pub struct Optional<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Box<Classifier<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Optional<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        let mut inner_draft = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
        inner_draft.build(source);

        if inner_draft.record.is_effected() {
            draft.marker = inner_draft.marker;
            draft.position = inner_draft.position;
            draft.consumed = inner_draft.consumed;
            draft.form = inner_draft.form;
        }

        draft.record.align();
    }
}

#[derive(Clone)]
pub struct Wrapper<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Box<Classifier<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Wrapper<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        let mut inner_draft = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
        inner_draft.build(source);

        draft.marker = inner_draft.marker;
        draft.position = inner_draft.position;
        draft.consumed = inner_draft.consumed;
        draft.record = inner_draft.record;
        draft.form = inner_draft.form;
    }
}

#[derive(Clone)]
pub struct Sequence<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub patterns: Vec<Classifier<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Sequence<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        let mut index = draft.marker;
        let mut position = draft.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::with_capacity(self.patterns.len());

        for pattern in &self.patterns {
            let mut child = Draft::new(index, position, pattern.clone());
            child.build(source);

            match child.record {
                super::former::Record::Aligned => {
                    draft.record.align();
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                }
                super::former::Record::Failed => {
                    draft.record.fail();
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                    break;
                }
                super::former::Record::Blank => {
                    draft.record.empty();
                    break;
                }
                super::former::Record::Skipped => {}
            }
        }

        draft.marker = index;
        draft.position = position;

        if forms.is_empty() {
            draft.consumed.clear();
            draft.form = Form::blank(Span::point(draft.position));
        } else {
            draft.consumed = consumed;
            draft.form = Form::multiple(forms);
        }
    }
}

#[derive(Clone)]
pub struct Repetition<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Box<Classifier<Input, Output, Failure>>,
    pub minimum: usize,
    pub maximum: Option<usize>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Repetition<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, source: &mut dyn Source<Input>, draft: &mut Draft<Input, Output, Failure>) {
        let mut index = draft.marker;
        let mut position = draft.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::new();

        while source.peek_ahead(index).is_some() {
            let mut child = Draft::new(index, position, self.pattern.as_ref().clone());
            child.build(source);

            if child.marker == index {
                break;
            }

            match child.record {
                super::former::Record::Aligned | super::former::Record::Failed => {
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                }
                super::former::Record::Skipped => {}
                super::former::Record::Blank => {
                    break;
                }
            }

            if let Some(max) = self.maximum {
                if forms.len() >= max {
                    break;
                }
            }
        }

        if forms.len() >= self.minimum {
            draft.marker = index;
            draft.position = position;
            draft.consumed = consumed;
            draft.record.align();

            if forms.is_empty() {
                draft.form = Form::blank(Span::point(draft.position));
            } else {
                draft.form = Form::multiple(forms);
            }
        } else {
            draft.record.empty();
        }
    }
}

#[derive(Clone)]
pub struct Classifier<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Arc<dyn Pattern<Input, Output, Failure>>,
    pub order: Option<Order<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Classifier<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn new(pattern: Arc<dyn Pattern<Input, Output, Failure>>) -> Self {
        Self {
            pattern,
            order: None,
        }
    }

    #[inline]
    pub fn order(
        classifier: impl Into<Box<Classifier<Input, Output, Failure>>>,
        order: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            pattern: Arc::new(Wrapper {
                pattern: classifier.into()
            }),
            order: Some(order),
        }
    }

    pub fn with_order(mut self, order: Order<Input, Output, Failure>) -> Self {
        self.order = Some(order);
        self
    }

    pub fn literal(value: impl PartialEq<Input> + Send + Sync + 'static) -> Self {
        Self::new(Arc::new(Identical {
            value: Arc::new(value),
        }))
    }

    pub fn alternative(patterns: impl Into<Vec<Self>>) -> Self {
        Self::new(Arc::new(Alternative { patterns: patterns.into() }))
    }

    pub fn sequence(patterns: impl Into<Vec<Self>>) -> Self {
        Self::new(Arc::new(Sequence { patterns: patterns.into() }))
    }

    pub fn optional(pattern: Self) -> Self {
        Self::new(Arc::new(Optional {
            pattern: Box::new(pattern),
        }))
    }

    pub fn repeat(pattern: Self, minimum: usize, maximum: Option<usize>) -> Self {
        Self::new(Arc::new(Repetition {
            pattern: Box::new(pattern),
            minimum,
            maximum,
        }))
    }

    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + Send + Sync + 'static,
    {
        Self::new(Arc::new(Predicate {
            function: Arc::new(predicate),
        }))
    }

    pub fn negate(pattern: Self) -> Self {
        Self::new(Arc::new(Reject {
            pattern: Box::new(pattern),
        }))
    }

    pub fn wrapper(pattern: Self) -> Self {
        Self::new(Arc::new(Wrapper {
            pattern: Box::new(pattern),
        }))
    }

    pub fn lazy<F>(factory: F) -> Self
    where
        F: Fn() -> Self + Send + Sync + 'static,
    {
        Self::new(Arc::new(Deferred {
            function: Arc::new(factory),
        }))
    }

    pub fn anything() -> Self {
        Self::predicate(|_| true)
    }

    pub fn nothing() -> Self {
        Self::predicate(|_| false)
    }

    pub fn capture(self, identifier: Artifact) -> Self {
        self.with_order(Order::Capture(identifier))
    }

    pub fn transform<T>(self, transform: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure>
        + Send
        + Sync
        + 'static,
    {
        self.with_order(Order::Convert(Arc::new(Mutex::new(transform))))
    }

    pub fn conditional(
        self,
        found: Order<Input, Output, Failure>,
        missing: Order<Input, Output, Failure>,
    ) -> Self {
        self.with_order(Order::Trigger {
            found: Box::new(found),
            missing: Box::new(missing),
        })
    }

    pub fn required(self, order: Order<Input, Output, Failure>) -> Self {
        self.conditional(Order::perform(|| {}), order)
    }

    pub fn as_optional(&self) -> Self {
        Self::optional(self.clone())
    }

    pub fn as_repeat(&self, min: usize, max: Option<usize>) -> Self {
        Self::repeat(self.clone(), min, max)
    }

    #[inline]
    pub fn with_action(mut self, order: Order<Input, Output, Failure>) -> Self {
        self.order = Some(order);
        self
    }

    #[inline]
    pub fn with_ignore(mut self) -> Self {
        self.order = Some(Order::Ignore);
        self
    }


    #[inline]
    pub fn with_conditional(
        mut self,
        found: Order<Input, Output, Failure>,
        missing: Order<Input, Output, Failure>,
    ) -> Self {
        self.order = Some(Order::Trigger {
            found: Box::new(found),
            missing: Box::new(missing),
        });
        self
    }

    #[inline]
    pub fn with_transform<T>(mut self, transform: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure>
        + Send
        + Sync
        + 'static,
    {
        self.order = Some(Order::Convert(Arc::new(Mutex::new(transform))));
        self
    }
}