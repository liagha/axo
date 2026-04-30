use crate::{
    combinator::{Consume, Formable, Formation},
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
        Consume::run(former, formation, input);
    }
}
