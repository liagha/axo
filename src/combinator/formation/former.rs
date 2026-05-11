use crate::{
    combinator::{Build, Combinator, Commit, Form, Formable, Formation},
    data::{
        memory::{replace, Arc},
        Offset,
    },
    internal::hash::Map,
    tracker::Peekable,
};

use super::{Joint, memo::Memo, Sink};

pub type Stash<'a, 'source, Source, Input, Output, Failure> = Vec<(
    usize,
    Arc<
        dyn Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
        + Send
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
        Sink::push(self, formation, input);
    }

    #[inline(always)]
    pub fn build(
        &mut self,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) {
        Build::run(self, formation);
    }

    #[inline(always)]
    pub fn form(
        &mut self,
        formation: Formation<'a, 'source, Source, Input, Output, Failure>,
    ) -> Form<'a, Input, Output, Failure> {
        let mut active = Formation::new(formation.combinator.clone(), 0, self.source.origin());
        self.build(&mut active);

        Commit::run(self, &active);

        replace(&mut self.forms[active.form], Form::Blank)
    }
}