use crate::{
    combinator::{Combinator, Form, Formable, Formation, Former, Outcome},
    data::memory::Arc,
    tracker::Peekable,
};

use super::Joint;

pub struct Consume;

impl Consume {
    #[inline(always)]
    pub fn run<'a, 'source, Source, Input, Output, Failure>(
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
        input: Input,
    ) where
        Source: Peekable<'a, Input> + Clone,
        Source::State: Default,
        Input: Formable<'a>,
        Output: Formable<'a>,
        Failure: Formable<'a>,
    {
        former
            .source
            .next(&mut formation.marker, &mut formation.state);

        let consumed = former.consumed.len();
        let form = former.forms.len();

        former.consumed.push(input.clone());
        former.forms.push(Form::input(input));

        formation.consumed.push(consumed);
        formation.form = form;
        formation.stack.push(form);
    }
}

pub struct Commit;

impl Commit {
    #[inline(always)]
    pub fn run<'a, 'source, Source, Input, Output, Failure>(
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        active: &Formation<'a, 'source, Source, Input, Output, Failure>,
    ) where
        Source: Peekable<'a, Input> + Clone,
        Source::State: Default,
        Input: Formable<'a>,
        Output: Formable<'a>,
        Failure: Formable<'a>,
    {
        if matches!(active.outcome, Outcome::Aligned | Outcome::Failed) {
            former.source.set_index(active.marker);
            former.source.set_state(active.state);
        }
    }
}

pub struct Build;

impl Build {
    #[inline(always)]
    pub fn run<'a, 'source, Source, Input, Output, Failure>(
        former: &mut Former<'a, 'source, Source, Input, Output, Failure>,
        formation: &mut Formation<'a, 'source, Source, Input, Output, Failure>,
    ) where
        Source: Peekable<'a, Input> + Clone,
        Source::State: Default,
        Input: Formable<'a>,
        Output: Formable<'a>,
        Failure: Formable<'a>,
    {
        let combinator: Arc<
            dyn Combinator<'a, Joint<'a, 'source, Source, Input, Output, Failure>>
            + Send
            + Sync
            + 'source,
        > = formation.combinator.clone();

        let mut joint: Joint<'a, 'source, Source, Input, Output, Failure> = unsafe {
            std::mem::transmute_copy(&(former, formation))
        };
        combinator.combinator(&mut joint);
    }
}