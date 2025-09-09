pub mod record {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Record {
        Panicked,
        Aligned,
        Failed,
        Blank,
        Ignored,
        Custom(i8),
    }

    impl Into<i8> for Record {
        fn into(self) -> i8 {
            match self {
                Record::Panicked => 127,
                Record::Aligned => 1,
                Record::Failed => 0,
                Record::Blank => -1,
                Record::Ignored => -2,
                Record::Custom(value) => value,
            }
        }
    }

    impl From<i8> for Record {
        fn from(value: i8) -> Record {
            match value {
                127 => Record::Panicked,
                1 => Record::Aligned,
                0 => Record::Failed,
                -1 => Record::Blank,
                -2 => Record::Ignored,
                value => Record::Custom(value),
            }
        }
    }
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