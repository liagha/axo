pub mod outcome {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Outcome {
        Panicked,
        Aligned,
        Failed,
        Blank,
        Ignored,
        Custom(i8),
    }

    impl Outcome {
        #[inline]
        pub const fn priority(self) -> i8 {
            match self {
                Outcome::Panicked => 4,
                Outcome::Failed => 3,
                Outcome::Aligned => 2,
                Outcome::Ignored => 1,
                Outcome::Blank => 0,
                Outcome::Custom(v) => v,
            }
        }

        #[inline]
        pub const fn is_productive(self) -> bool {
            matches!(self, Outcome::Aligned | Outcome::Failed)
        }

        #[inline]
        pub const fn is_terminal(self) -> bool {
            matches!(self, Outcome::Panicked | Outcome::Failed)
        }

        #[inline]
        pub const fn is_neutral(self) -> bool {
            matches!(self, Outcome::Blank | Outcome::Ignored)
        }

        #[inline]
        pub const fn is_success(self) -> bool {
            matches!(self, Outcome::Aligned)
        }

        #[inline]
        pub fn escalate(self, other: Outcome) -> Outcome {
            if other.priority() > self.priority() {
                other
            } else {
                self
            }
        }

        #[inline]
        pub fn demote(self) -> Outcome {
            match self {
                Outcome::Panicked => Outcome::Failed,
                Outcome::Aligned => Outcome::Ignored,
                other => other,
            }
        }
    }

    impl Into<i8> for Outcome {
        fn into(self) -> i8 {
            match self {
                Outcome::Panicked => 127,
                Outcome::Aligned => 1,
                Outcome::Failed => 0,
                Outcome::Blank => -1,
                Outcome::Ignored => -2,
                Outcome::Custom(value) => value,
            }
        }
    }

    impl From<i8> for Outcome {
        fn from(value: i8) -> Outcome {
            match value {
                127 => Outcome::Panicked,
                1 => Outcome::Aligned,
                0 => Outcome::Failed,
                -1 => Outcome::Blank,
                -2 => Outcome::Ignored,
                value => Outcome::Custom(value),
            }
        }
    }
}

use {
    crate::{
        formation::{
            Classifier,
            Form,
            Action,
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

pub type Cache<'a, Input, Output, Failure> = Vec<(usize, Rc<dyn Action<'a, Input, Output, Failure> + 'a>)>;

pub struct Memo<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub outcome: outcome::Outcome,
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
        let action = classifier.action.clone();
        action.action(self, classifier);
    }

    #[inline(always)]
    pub fn form(
        &mut self,
        classifier: Classifier<'a, Input, Output, Failure>,
    ) -> Form<'a, Input, Output, Failure> {
        let initial = self.source.position();
        let mut active = Classifier::new(classifier.action.clone(), 0, initial);

        self.build(&mut active);

        if active.is_effected() {
            self.source.set_index(active.marker);
            self.source.set_position(active.position);
        }

        replace(&mut self.forms[active.form], Form::Blank)
    }
}
