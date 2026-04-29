use crate::{
    combinator::{Form, Formable, Formation},
    tracker::Peekable,
};

use super::former::Former;

pub struct Sink;

impl Sink {
    #[inline(always)]
    pub fn push<'a, 'source, Source, Input, Output, Failure>(
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
