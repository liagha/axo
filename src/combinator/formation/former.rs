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

    impl From<Outcome> for i8 {
        fn from(val: Outcome) -> i8 {
            match val {
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

use crate::combinator::outcome::Outcome;
use crate::{
    combinator::{Combinator, Form, Formable, Formation},
    data::{
        memory::{replace, Arc},
        Offset,
    },
    internal::hash::Map,
    tracker::Peekable,
};

use super::memo::Memo;

pub type Stash<'a, 'source, Source, Input, Output, Failure> = Vec<(
    usize,
    Arc<
    dyn Combinator<
    'a,
    Former<'a, 'source, Source, Input, Output, Failure>,
    Formation<'a, 'source, Source, Input, Output, Failure>,
> + Send
+ Sync
+ 'source,
>,
)>;

pub struct Former<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub source: &'source mut Source,
    pub consumed: Vec<Input>,
    pub forms: Vec<Form<'a, Input, Output, Failure>>,
    pub stash: Stash<'a, 'source, Source, Input, Output, Failure>,
    pub memo: Map<(usize, Offset), Memo<'a, Source, Input, Output, Failure>>,
}

impl<'a, 'source, Source, Input, Output, Failure>
Former<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Source::State: Default,
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
    pub fn push(
        &mut self,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
        input: Input,
    ) {
        self.source
            .next(&mut formation.marker, &mut formation.state);

        let consumed = self.consumed.len();
        let form = self.forms.len();

        self.consumed.push(input.clone());
        self.forms.push(Form::input(input));

        formation.consumed.push(consumed);
        formation.form = form;
        formation.stack.push(form);
    }

    #[inline(always)]
    pub fn build(
        &mut self,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        let combinator = formation.combinator.clone();
        combinator.combinator(self, formation);
    }

    #[inline(always)]
    pub fn form(
        &mut self,
        formation: Formation<'a, 'source, Source, Input, Output, Failure>,
    ) -> Form<'a, Input, Output, Failure> {
        let mut active = Formation::new(formation.combinator.clone(), 0, self.source.origin());
        self.build(&mut active);

        if matches!(active.outcome, Outcome::Aligned | Outcome::Failed) {
            self.source.set_index(active.marker);
            self.source.set_state(active.state);
        }

        replace(&mut self.forms[active.form], Form::Blank)
    }
}