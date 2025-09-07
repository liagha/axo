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
        },
    },
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
    pub fn build(&mut self, classifier: &mut Classifier<'former, Input, Output, Failure>) {
        classifier.order.clone().order(self, classifier);
    }

    #[inline(always)]
    pub fn form(&mut self, classifier: Classifier<'former, Input, Output, Failure>) -> Form<'former, Input, Output, Failure> {
        let initial = self.source.position();
        let mut classifier = Classifier::new(classifier.order, 0, initial);

        self.build(&mut classifier);

        if classifier.is_effected() {
            self.source.set_index(classifier.marker);
            self.source.set_position(classifier.position);
        }

        classifier.form
    }
}