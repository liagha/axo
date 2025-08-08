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
        tracker::{
            Position,
        },
        data::PhantomData,
    },
    record::*,
};

pub struct Composer<'instance, 'composer, Input: Formable<'composer>, Output: Formable<'composer>, Failure: Formable<'composer>> {
    pub source: &'instance mut dyn Source<'composer, Input>,
    pub _phantom: PhantomData<(Input, Output, Failure)>,
}

impl<'a, 'composer, Input: Formable<'composer>, Output: Formable<'composer>, Failure: Formable<'composer>> Composer<'a, 'composer, Input, Output, Failure> {
    #[inline(always)]
    pub fn new(source: &'a mut dyn Source<'composer, Input>) -> Self {
        Self {
            source,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn build(&mut self, draft: &mut Draft<'composer, Input, Output, Failure>) {
        let classifier = draft.classifier.order.clone();
        classifier.order(self, draft);
    }
}

#[derive(Clone, Debug)]
pub struct Draft<'draft, Input: Formable<'draft>, Output: Formable<'draft>, Failure: Formable<'draft>> {
    pub marker: usize,
    pub position: Position<'draft>,
    pub consumed: Vec<Input>,
    pub record: Record,
    pub classifier: Classifier<'draft, Input, Output, Failure>,
    pub form: Form<'draft, Input, Output, Failure>,
}

impl<'draft, Input: Formable<'draft>, Output: Formable<'draft>, Failure: Formable<'draft>> Draft<'draft, Input, Output, Failure> {
    #[inline(always)]
    pub const fn new(index: usize, position: Position<'draft>, classifier: Classifier<'draft, Input, Output, Failure>) -> Self {
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

pub trait Former<'former, Input: Formable<'former>, Output: Formable<'former>, Failure: Formable<'former>> {
    fn form(&mut self, classifier: Classifier<'former, Input, Output, Failure>) -> Form<'former, Input, Output, Failure>
    where
        Self: Source<'former, Input>;
}

impl<'former, Target, Input: Formable<'former>, Output: Formable<'former>, Failure: Formable<'former>> Former<'former, Input, Output, Failure> for Target
{
    fn form(&mut self, classifier: Classifier<'former, Input, Output, Failure>) -> Form<'former, Input, Output, Failure>
    where
        Self: Source<'former, Input>,
    {
        let initial = self.position();
        let mut draft = Draft::new(0, initial, classifier);

        let mut composer: Composer<'_, 'former, Input, Output, Failure> = Composer::new(self as &mut dyn Source<'former, Input>);
        composer.build(&mut draft);

        if draft.is_effected() {
            composer.source.set_index(draft.marker);
            composer.source.set_position(draft.position);
        }

        draft.form
    }
}