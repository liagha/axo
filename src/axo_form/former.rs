pub mod record {
    pub const PANICKED: i8 = 120;
    pub const ALIGNED: i8 = 1;
    pub const FAILED: i8 = 0;
    pub const BLANK: i8 = -1;
    pub const IGNORED: i8 = -2;
}

use {
    super::{
        form::Form,
        helper::Source,
        pattern::Classifier,
    },
    crate::{
        axo_cursor::{
            Peekable, Position,
        },
        format::Debug,
        hash::Hash,
        marker::PhantomData,
    },
    record::*,
};
use crate::axo_internal::compiler::Marked;

pub struct Composer<'c, Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub source: &'c mut dyn Source<Input>,
    pub _phantom: PhantomData<(Input, Output, Failure)>,
}

impl <'c, Input, Output, Failure> Composer<'c, Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline(always)]
    pub fn new(source: &'c mut (dyn Source<Input> + 'c)) -> Composer<'c, Input, Output, Failure> {
        Self {
            source,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
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
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
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
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline(always)]
    pub const fn new(index: usize, position: Position, classifier: Classifier<Input, Output, Failure>) -> Self {
        Self {
            marker: index,
            position,
            consumed: Vec::new(),
            record: BLANK,
            classifier,
            form: Form::Blank,
        }
    }

    #[inline(always)]
    pub const fn is_panicked(&self) -> bool {
        self.record == PANICKED
    }

    #[inline(always)]
    pub const fn is_aligned(&self) -> bool {
        self.record == ALIGNED
    }

    #[inline(always)]
    pub const fn is_failed(&self) -> bool {
        self.record == FAILED
    }

    #[inline(always)]
    pub const fn is_effected(&self) -> bool {
        matches!(self.record, ALIGNED | FAILED)
    }

    #[inline(always)]
    pub const fn is_blank(&self) -> bool {
        self.record == BLANK
    }

    #[inline(always)]
    pub const fn is_ignored(&self) -> bool {
        self.record == IGNORED
    }

    #[inline(always)]
    pub const fn panic(&mut self) {
        self.record = PANICKED;
    }

    #[inline(always)]
    pub const fn align(&mut self) {
        self.record = ALIGNED;
    }

    #[inline(always)]
    pub const fn fail(&mut self) {
        self.record = FAILED;
    }

    #[inline(always)]
    pub const fn empty(&mut self) {
        self.record = BLANK;
    }

    #[inline(always)]
    pub const fn ignore(&mut self) {
        self.record = IGNORED;
    }
}

pub trait Former<Input, Output, Failure>: Peekable<Input> + Marked
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn strain(&mut self, pattern: Classifier<Input, Output, Failure>);
    fn form(&mut self, pattern: Classifier<Input, Output, Failure>) -> Form<Input, Output, Failure>;
}

impl<Source, Input, Output, Failure> Former<Input, Output, Failure> for Source
where
    Source: Peekable<Input> + Marked,
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn strain(&mut self, pattern: Classifier<Input, Output, Failure>) {
        let mut inputs = Vec::with_capacity(self.len());
        let mut index = 0;
        let mut position = self.position();
        let mut composer = Composer::new(self);

        loop {
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