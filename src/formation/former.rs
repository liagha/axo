pub mod status {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Status {
        Panicked,
        Aligned,
        Failed,
        Blank,
        Ignored,
        Custom(i8),
    }

    impl Into<i8> for Status {
        fn into(self) -> i8 {
            match self {
                Status::Panicked => 127,
                Status::Aligned => 1,
                Status::Failed => 0,
                Status::Blank => -1,
                Status::Ignored => -2,
                Status::Custom(value) => value,
            }
        }
    }

    impl From<i8> for Status {
        fn from(value: i8) -> Status {
            match value {
                127 => Status::Panicked,
                1 => Status::Aligned,
                0 => Status::Failed,
                -1 => Status::Blank,
                -2 => Status::Ignored,
                value => Status::Custom(value),
            }
        }
    }
}

use {
    crate::{
        formation::{
            Classifier,
            Form,
            Order,
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

pub type Cache<'a, Input, Output, Failure> = Vec<(usize, Rc<dyn Order<'a, Input, Output, Failure> + 'a>)>;

pub struct Memo<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub status: status::Status,
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
