use {
    super::{
        order::Order,
        form::{Form},
        former::{Draft, Composer},
    },
    crate::{
        artifact::Artifact,
        hash::Hash,
        format::Debug,
        compiler::Context,
        thread::{Arc, Mutex},
        axo_cursor::{
            Spanned, Span,
        },
    },
};

pub trait Pattern<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>);
}

#[derive(Clone)]
pub struct Literal<Input> {
    pub value: Arc<dyn PartialEq<Input> + Send + Sync>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Literal<Input>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(draft.marker).cloned() {
            if self.value.eq(&peek) {
                draft.align();
                composer.source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.form = Form::input(peek);
            } else {
                draft.empty();
            }
        } else {
            draft.empty();
        }
    }
}

#[derive(Clone)]
pub struct Negate<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Box<Classifier<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Negate<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(draft.marker).cloned() {
            let mut child = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
            composer.build(&mut child);

            if !child.is_aligned() {
                draft.align();
                composer.source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.form = Form::input(peek);
            } else {
                draft.empty();
            }
        } else {
            draft.empty();
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if let Some(peek) = composer.source.get(draft.marker).cloned() {
            let predicate = (self.function)(&peek);

            if predicate {
                draft.align();
                composer.source.next(&mut draft.marker, &mut draft.position);
                draft.consumed.push(peek.clone());
                draft.form = Form::input(peek);
            } else {
                draft.empty();
            }
        } else {
            draft.empty();
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut best_child: Option<Draft<Input, Output, Failure>> = None;
        let mut best_score = i8::MIN;

        for pattern in &self.patterns {
            let mut child = Draft::new(draft.marker, draft.position, pattern.clone());
            composer.build(&mut child);

            if child.record > best_score {
                best_score = child.record;
                best_child = Some(child);

                if best_score == 1 {
                    break;
                }
            }
        }

        if let Some(best) = best_child {
            draft.record = best.record;
            draft.marker = best.marker;
            draft.position = best.position;
            draft.consumed = best.consumed;
            draft.form = best.form;
        } else {
            draft.empty();
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut child = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
        composer.build(&mut child);

        if child.is_effected() {
            draft.marker = child.marker;
            draft.position = child.position;
            draft.consumed = child.consumed;
            draft.form = child.form;
        }

        draft.align();
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut child = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
        composer.build(&mut child);

        draft.marker = child.marker;
        draft.position = child.position;
        draft.consumed = child.consumed;
        draft.record = child.record;
        draft.form = child.form;
    }
}

#[derive(Clone)]
pub struct Ranked<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub pattern: Box<Classifier<Input, Output, Failure>>,
    pub precedence: i8,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure> for Ranked<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut child = Draft::new(draft.marker, draft.position, self.pattern.as_ref().clone());
        composer.build(&mut child);

        draft.marker = child.marker;
        draft.position = child.position;
        draft.consumed = child.consumed.clone();
        draft.form = child.form.clone();

        if child.is_aligned() {
            draft.record = self.precedence.max(1);
        } else if child.is_failed() {
            draft.record = self.precedence.min(0);
        } else {
            draft.record = child.record;
        }
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut index = draft.marker;
        let mut position = draft.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::with_capacity(self.patterns.len());

        for pattern in &self.patterns {
            let mut child = Draft::new(index, position, pattern.clone());
            composer.build(&mut child);

            match child.record {
                1 => {
                    draft.align();
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                }
                0 => {
                    draft.fail();
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                    break;
                }
                -2 => {
                    index = child.marker;
                    position = child.position; 
                }
                _ => {
                    draft.empty();
                    break;
                }
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
    fn build(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let mut index = draft.marker;
        let mut position = draft.position;
        let mut consumed = Vec::new();
        let mut forms = Vec::new();

        while composer.source.peek_ahead(index).is_some() {
            let mut child = Draft::new(index, position, self.pattern.as_ref().clone());
            composer.build(&mut child);

            if child.marker == index {
                break;
            }

            match child.record {
                1 | 0 => {
                    index = child.marker;
                    position = child.position;
                    consumed.extend(child.consumed);
                    forms.push(child.form);
                }
                -2 => {
                    index = child.marker;
                    position = child.position;
                }
                _ => {
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
            draft.align();
            draft.marker = index;
            draft.position = position;
            draft.consumed = consumed;

            if forms.is_empty() {
                draft.form = Form::blank(Span::point(draft.position));
            } else {
                draft.form = Form::multiple(forms);
            }
        } else {
            draft.empty();
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
    pub fn ordered(
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
        Self::new(Arc::new(Literal {
            value: Arc::new(value),
        }))
    }

    pub fn negate(pattern: Self) -> Self {
        Self::new(Arc::new(Negate {
            pattern: Box::new(pattern),
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

    pub fn wrapper(pattern: Self) -> Self {
        Self::new(Arc::new(Wrapper {
            pattern: Box::new(pattern),
        }))
    }

    pub fn ranked(pattern: Self, precedence: i8) -> Self {
        Self::new(Arc::new(Ranked {
            pattern: Box::new(pattern),
            precedence,
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