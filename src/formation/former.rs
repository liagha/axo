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
    crate::{
        formation::{
            classifier::Classifier,
            form::Form,
            helper::{Formable, Source},
        },
        data::{
            memory::{
                replace, Rc
            },
            Identity, Offset
        },
        internal::hash::Map,
        tracker::Position,
    },
};

pub type Cache<'a, Input, Output, Failure> = Vec<(usize, Rc<dyn super::order::Order<'a, Input, Output, Failure> + 'a>)>;

pub struct Memo<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub record: record::Record,
    pub advance: Offset,
    pub position: Position<'a>,
    pub forms: Vec<Form<'a, Input, Output, Failure>>,
    pub inputs: Vec<Input>,
    pub consumed: Vec<Identity>,
    pub stack: Vec<Identity>,
    pub form: Identity,
    pub form_base: Offset,
    pub input_base: Offset,
}

pub struct Former<'b, 'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub source: &'b mut dyn Source<'a, Input>,
    pub consumed: Vec<Input>,
    pub forms: Vec<Form<'a, Input, Output, Failure>>,
    pub cache: Cache<'a, Input, Output, Failure>,
    pub memo: Map<(usize, Offset), Memo<'a, Input, Output, Failure>>,
}

impl<'b, 'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Former<'b, 'a, Input, Output, Failure>
{
    #[inline(always)]
    pub fn new(source: &'b mut dyn Source<'a, Input>) -> Self {
        Self {
            source,
            consumed: Vec::with_capacity(2048),
            forms: {
                let mut forms = Vec::with_capacity(2048);
                forms.push(Form::Blank);

                forms
            },
            cache: Vec::with_capacity(32),
            memo: Map::with_capacity(512),
        }
    }

    #[inline(always)]
    pub fn build(&mut self, classifier: &mut Classifier<'a, Input, Output, Failure>) {
        let order = classifier.order.clone();
        order.order(self, classifier);
    }

    #[inline(always)]
    pub fn form(
        &mut self,
        classifier: Classifier<'a, Input, Output, Failure>,
    ) -> Form<'a, Input, Output, Failure> {
        let initial = self.source.position();
        let mut active = Classifier::new(classifier.order.clone(), 0, initial);

        self.build(&mut active);

        if active.is_effected() {
            self.source.set_index(active.marker);
            self.source.set_position(active.position);
        }

        replace(&mut self.forms[active.form], Form::Blank)
    }
}
