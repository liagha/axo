use {
    super::{
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
use crate::axo_form::pattern::Source;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Record {
    Aligned,
    Skipped,
    Failed,
    Blank,
}

impl Record {
    #[inline]
    pub fn is_aligned(&self) -> bool {
        matches!(self, &Record::Aligned)
    }

    #[inline]
    pub fn is_skipped(&self) -> bool {
        matches!(self, &Record::Skipped)
    }

    #[inline]
    pub fn is_failed(&self) -> bool {
        matches!(self, Record::Failed)
    }

    #[inline]
    pub fn is_effected(&self) -> bool {
        matches!(self, &Record::Aligned | &Record::Failed)
    }

    #[inline]
    pub fn is_blank(&self) -> bool {
        matches!(self, &Record::Blank)
    }

    #[inline]
    pub fn align(&mut self) {
        *self = Record::Aligned;
    }

    #[inline]
    pub fn skip(&mut self) {
        *self = Record::Skipped;
    }

    #[inline]
    pub fn fail(&mut self) {
        *self = Record::Failed;
    }

    #[inline]
    pub fn empty(&mut self) {
        *self = Record::Blank;
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
    pub record: Record,
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
            record: Record::Blank,
            classifier: pattern,
            form: Form::new(FormKind::Blank, Span::point(position)),
        }
    }

    pub fn build(&mut self, source: &mut dyn Source<Input>) {
        let pattern = self.classifier.pattern.clone();
        let order = self.classifier.order.clone();

        pattern.build(source, self);

        if let Some(order) = order {
            order.execute(source, self);
        }
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

        while self.get(index).is_some() {
            let mut draft = Draft::new(index, position, pattern.clone());
            draft.build(self);

            if draft.record.is_aligned() {
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

        draft.build(self);

        if draft.record.is_effected() {
            self.set_index(draft.marker);
            self.set_position(draft.position);
        }

        draft.form
    }
}