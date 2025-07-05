use std::marker::PhantomData;
use {
    super::{
        helper::Source,
        pattern::Classifier,
        form::{Form, FormKind},
    },
    crate::{
        axo_cursor::{
            Position, Peekable,
            Span, Spanned,
        },
        compiler::Marked,
        format::Debug,
        hash::Hash,
    },
};

pub struct Composer<'c, Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub source: &'c mut dyn Source<Input>,
    pub _phantom: PhantomData<(Input, Output, Failure)>,
}

impl <'c, Input, Output, Failure> Composer<'c, Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn new(source: &'c mut (dyn Source<Input> + 'c)) -> Composer<'c, Input, Output, Failure> {
        Self {
            source,
            _phantom: PhantomData,
        }
    }

    pub fn build(&mut self, draft: &mut Draft<Input, Output, Failure>) {
        let pattern = draft.classifier.pattern.clone();
        let order = draft.classifier.order.clone();

        pattern.build(self, draft);

        if let Some(order) = order {
            order.execute(self.source, draft);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Draft<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub marker: usize,
    pub position: Position,
    pub consumed: Vec<Input>,
    pub record: i8,
    pub classifier: Classifier<Input, Output, Failure>,
    pub form: Form<Input, Output, Failure>,
}

impl<Input, Output, Failure> Draft<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn new(index: usize, position: Position, pattern: Classifier<Input, Output, Failure>) -> Self {
        Self {
            marker: index,
            position,
            consumed: Vec::new(),
            record: -1,
            classifier: pattern,
            form: Form::new(FormKind::Blank, Span::point(position)),
        }
    }

    /// Aligned = 1
    /// Failed = 0
    /// Blank = -1
    #[inline]
    pub fn is_aligned(&self) -> bool {
        matches!(self.record, 1)
    }

    #[inline]
    pub fn is_failed(&self) -> bool {
        matches!(self.record, 0)
    }

    #[inline]
    pub fn is_effected(&self) -> bool {
        matches!(self.record, 1 | 0)
    }

    #[inline]
    pub fn is_blank(&self) -> bool {
        matches!(self.record, -1)
    }

    #[inline]
    pub fn align(&mut self) {
        self.record = 1;
    }

    #[inline]
    pub fn fail(&mut self) {
        self.record = 0;
    }

    #[inline]
    pub fn empty(&mut self) {
        self.record = -1;
    }
}

pub trait Former<Input, Output, Failure>: Peekable<Input> + Marked
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn strain(&mut self, pattern: Classifier<Input, Output, Failure>);
    fn form(&mut self, pattern: Classifier<Input, Output, Failure>) -> Form<Input, Output, Failure>;
}

impl<Source, Input, Output, Failure> Former<Input, Output, Failure> for Source
where
    Source: Peekable<Input> + Marked,
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn strain(&mut self, pattern: Classifier<Input, Output, Failure>) {
        let mut inputs = Vec::with_capacity(self.len());
        let mut index = 0;
        let mut position = self.position();

        loop {
            let mut composer = Composer::new(self);

            // Check if we have more items to process
            if composer.source.get(index).is_none() {
                break;
            }

            let mut draft = Draft::new(index, position, pattern.clone());
            composer.build(&mut draft);

            if draft.is_aligned() {
                index = draft.marker + 1;
                position = draft.position;
                inputs.extend(draft.consumed);
            } else {
                index = draft.marker + 1;
                position = draft.position;
            }
        }

        *self.input_mut() = inputs;
    }

    fn form(&mut self, pattern: Classifier<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let mut draft = Draft::new(0, self.position(), pattern);
        let mut composer = Composer::new(self);

        composer.build(&mut draft);

        if draft.is_effected() {
            self.set_index(draft.marker);
            self.set_position(draft.position);
        }

        draft.form
    }
}