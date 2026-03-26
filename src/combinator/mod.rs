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

pub struct Multiple<'a, 'src, Host, State> {
    pub actions: Vec<Rc<dyn Action<'a, Host, State> + 'src>>,
}

pub struct Ignore;

pub struct Skip;

pub struct Transform<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub transformer: Rc<dyn Fn(
        &mut Former<'a, 'src, Source, Input, Output, Failure>,
        &mut Classifier<'a, 'src, Source, Input, Output, Failure>,
    ) -> Result<(), Failure> + 'src>,
}

pub struct Fail<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub emitter: Rc<dyn Fn(
        &mut Former<'a, 'src, Source, Input, Output, Failure>,
        Classifier<'a, 'src, Source, Input, Output, Failure>,
    ) -> Failure + 'src>,
}

pub struct Panic<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub emitter: Rc<dyn Fn(
        &mut Former<'a, 'src, Source, Input, Output, Failure>,
        Classifier<'a, 'src, Source, Input, Output, Failure>,
    ) -> Failure + 'src>,
}

#[derive(Clone)]
pub struct Literal<'a, 'src, Input>
where
    Input: Formable<'a>,
{
    pub value: Rc<dyn PartialEq<Input> + 'src>,
    pub _marker: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Predicate<'a, 'src, Input>
where
    Input: Formable<'a>,
{
    pub function: Rc<dyn Fn(&Input) -> bool + 'src>,
    pub _marker: PhantomData<&'a ()>, 
}

pub struct Deferred<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub factory: fn() -> Classifier<'a, 'src, Source, Input, Output, Failure>,
}

pub struct Optional<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub classifier: Box<Classifier<'a, 'src, Source, Input, Output, Failure>>,
}

pub struct Alternative<
    'a,
    'src,
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
    pub patterns: [Classifier<'a, 'src, Source, Input, Output, Failure>; SIZE],
}

pub struct Sequence<
    'a,
    'src,
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
    pub patterns: [Classifier<'a, 'src, Source, Input, Output, Failure>; SIZE],
}

pub struct Repetition<'a, 'src, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input>,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub classifier: Box<Classifier<'a, 'src, Source, Input, Output, Failure>>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}
