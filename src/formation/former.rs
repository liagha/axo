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
        classifier::Classifier,
        form::Form,
        helper::{Formable, Source},
    },
    crate::{
        data::{
            memory::PhantomData,
            Offset,
        },
        tracker::{
            Position,
        },
    },
    record::*,
};

pub struct Former<'instance, 'former, Input: Formable<'former>, Output: Formable<'former>, Failure: Formable<'former>> {
    pub source: &'instance mut dyn Source<'former, Input>,
    pub _phantom: PhantomData<(Input, Output, Failure)>,
}

impl<'instance, 'former, Input: Formable<'former>, Output: Formable<'former>, Failure: Formable<'former>> Former<'instance, 'former, Input, Output, Failure> {
    #[inline(always)]
    pub fn new(source: &'instance mut dyn Source<'former, Input>) -> Self {
        Self {
            source,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn build(&mut self, draft: &mut Draft<'former, Input, Output, Failure>) {
        let classifier = draft.classifier.order.clone();
        classifier.order(self, draft);
    }

    #[inline(always)]
    pub fn form(&mut self, classifier: Classifier<'former, Input, Output, Failure>) -> Form<'former, Input, Output, Failure> {
        let initial = self.source.position();
        let mut draft = Draft::new(0, initial, classifier);

        self.build(&mut draft);

        if draft.is_effected() {
            self.source.set_index(draft.marker);
            self.source.set_position(draft.position);
        }

        draft.form
    }
}

#[derive(Clone, Debug)]
pub struct Draft<'draft, Input: Formable<'draft>, Output: Formable<'draft>, Failure: Formable<'draft>> {
    pub marker: Offset,
    pub position: Position<'draft>,
    pub consumed: Vec<Input>,
    pub record: Record,
    pub classifier: Classifier<'draft, Input, Output, Failure>,
    pub form: Form<'draft, Input, Output, Failure>,
}

impl<'draft, Input: Formable<'draft>, Output: Formable<'draft>, Failure: Formable<'draft>> Draft<'draft, Input, Output, Failure> {
    #[inline(always)]
    pub const fn new(marker: Offset, position: Position<'draft>, classifier: Classifier<'draft, Input, Output, Failure>) -> Self {
        Self {
            marker,
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