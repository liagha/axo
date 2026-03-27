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

use crate::{
    combinator::{Action, Classifier, Form, Formable},
    data::{
        memory::{replace, Arc},
        Identity, Offset,
    },
    internal::hash::Map,
    tracker::{Peekable, Position},
};
use crate::combinator::outcome::Outcome;

pub type Stash<'a, 'source, Source, Input, Output, Failure> = Vec<(
    usize,
    Arc<
        dyn Action<
                'a,
                Former<'a, 'source, Source, Input, Output, Failure>,
                Classifier<'a, 'source, Source, Input, Output, Failure>,
            > + Send + Sync + 'source,
    >,
)>;

pub struct Memo<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub outcome: Outcome,
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

pub struct Former<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub source: &'source mut Source,
    pub consumed: Vec<Input>,
    pub forms: Vec<Form<'a, Input, Output, Failure>>,
    pub stash: Stash<'a, 'source, Source, Input, Output, Failure>,
    pub memo: Map<(usize, Offset), Memo<'a, Input, Output, Failure>>,
}

impl<'a, 'source, Source, Input, Output, Failure>
    Former<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline(always)]
    pub fn new(source: &'source mut Source) -> Self {
        Self {
            source,
            consumed: Vec::new(),
            forms: Vec::new(),
            stash: Stash::new(),
            memo: Map::new(),
        }
    }

    #[inline(always)]
    pub fn build(
        &mut self,
        classifier: &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let action = classifier.action.clone();
        action.action(self, classifier);
    }

    #[inline(always)]
    pub fn form(
        &mut self,
        classifier: Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) -> Form<'a, Input, Output, Failure> {
        let initial = self.source.position();
        let mut active = Classifier::new(classifier.action.clone(), 0, initial);

        self.build(&mut active);

        if matches!(active.outcome, Outcome::Aligned | Outcome::Failed) {
            self.source.set_index(active.marker);
            self.source.set_position(active.position);
        }

        replace(&mut self.forms[active.form], Form::Blank)
    }
}
