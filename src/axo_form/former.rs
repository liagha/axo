pub mod record {
    pub type Record = i8;

    pub const PANICKED: Record = Record::MAX;
    pub const ALIGNED: Record = 1;
    pub const FAILED: Record = 0;
    pub const BLANK: Record = -1;
    pub const IGNORED: Record = -2;
}

use {
    super::{
        Formable,
        Source,
        form::Form,
        classifier::Classifier,
    },
    crate::{
        axo_cursor::{
            Position,
        },
        marker::PhantomData,
    },
    record::*,
};

pub struct Composer<'c, Input: Formable, Output: Formable, Failure: Formable> {
    pub source: &'c mut dyn Source<Input>,
    pub _phantom: PhantomData<(Input, Output, Failure)>,
}

impl <'c, Input: Formable, Output: Formable, Failure: Formable> Composer<'c, Input, Output, Failure> {
    #[inline(always)]
    pub fn new(source: &'c mut (dyn Source<Input> + 'c)) -> Composer<'c, Input, Output, Failure> {
        Self {
            source,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn build(&mut self, draft: &mut Draft<Input, Output, Failure>) {
        let classifier = draft.classifier.order.clone();

        classifier.order(self, draft);
    }
}

#[derive(Clone, Debug)]
pub struct Draft<Input: Formable, Output: Formable, Failure: Formable> {
    pub marker: usize,
    pub position: Position,
    pub consumed: Vec<Input>,
    pub record: Record,
    pub classifier: Classifier<Input, Output, Failure>,
    pub form: Form<Input, Output, Failure>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Draft<Input, Output, Failure> {
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
    pub const fn set_panic(&mut self) {
        self.record = PANICKED;
    }

    #[inline(always)]
    pub const fn set_align(&mut self) {
        self.record = ALIGNED;
    }

    #[inline(always)]
    pub const fn set_fail(&mut self) {
        self.record = FAILED;
    }

    #[inline(always)]
    pub const fn set_empty(&mut self) {
        self.record = BLANK;
    }

    #[inline(always)]
    pub const fn set_ignore(&mut self) {
        self.record = IGNORED;
    }
}

pub trait Former<Input: Formable, Output: Formable, Failure: Formable>: Source<Input> {
    fn form(&mut self, classifier: Classifier<Input, Output, Failure>) -> Form<Input, Output, Failure>;
}

impl<Target, Input: Formable, Output: Formable, Failure: Formable> Former<Input, Output, Failure> for Target
where
    Target: Source<Input>,
{
    fn form(&mut self, classifier: Classifier<Input, Output, Failure>) -> Form<Input, Output, Failure> {
        let mut draft = Draft::new(0, self.position(), classifier);
        let mut composer = Composer::new(self);

        composer.build(&mut draft);

        if draft.is_effected() {
            self.set_index(draft.marker);
            self.set_position(draft.position);
        }

        draft.form
    }
}