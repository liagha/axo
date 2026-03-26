use crate::data::{sync::{AtomicUsize, Ordering}, memory::Rc, Identity, Scale, Boolean};

mod formation;
mod operation;

pub use formation::*;

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(super) fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

use crate::{
    format::Show,
    internal::hash::Hash,
    data::memory::PhantomData,
    tracker::{Spanned, Peekable},
};

pub trait Formable<'a>:
Clone + Eq + Hash + PartialEq + Show<'a> + Spanned<'a> + 'a {}

impl<'a, T> Formable<'a> for T where
    T: Clone + Eq + Hash + PartialEq + Show<'a> + Spanned<'a> + 'a {}

pub trait Action<'a, Host, State> {
    fn action(
        &self,
        host: &mut Host,
        state: &mut State,
    );
}

pub struct Multiple<'a, 'source, Host, State> {
    pub actions: Vec<Rc<dyn Action<'a, Host, State> + 'source>>,
}

pub struct Ignore;

pub struct Skip;

pub struct Transform<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub transformer: Rc<dyn Fn(
        &mut Former<'a, 'source, Source, Input, Output, Failure>,
        &mut Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) -> Result<(), Failure> + 'source>,
}

pub struct Fail<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub emitter: Rc<dyn Fn(
        &mut Former<'a, 'source, Source, Input, Output, Failure>,
        Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) -> Failure + 'source>,
}

pub struct Panic<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub emitter: Rc<dyn Fn(
        &mut Former<'a, 'source, Source, Input, Output, Failure>,
        Classifier<'a, 'source, Source, Input, Output, Failure>,
    ) -> Failure + 'source>,
}

#[derive(Clone)]
pub struct Literal<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub value: Rc<dyn PartialEq<Input> + 'source>,
    pub _marker: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Predicate<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub function: Rc<dyn Fn(&Input) -> bool + 'source>,
    pub _marker: PhantomData<&'a ()>, 
}

pub struct Deferred<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub factory: fn() -> Classifier<'a, 'source, Source, Input, Output, Failure>,
}

pub struct Optional<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub classifier: Box<Classifier<'a, 'source, Source, Input, Output, Failure>>,
}

pub struct Alternative<
    'a,
    'source,
    Source,
    Input,
    Output,
    Failure,
    const SIZE: Scale,
>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub patterns: [Classifier<'a, 'source, Source, Input, Output, Failure>; SIZE],
}

pub struct Sequence<
    'a,
    'source,
    Source,
    Input,
    Output,
    Failure,
    const SIZE: Scale,
>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub patterns: [Classifier<'a, 'source, Source, Input, Output, Failure>; SIZE],
}

pub struct Repetition<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub classifier: Box<Classifier<'a, 'source, Source, Input, Output, Failure>>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}
