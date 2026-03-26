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
    tracker::{Peekable, Spanned},
};

pub trait Formable<'a>:
Clone + Eq + Hash + PartialEq + Show<'a> + Spanned<'a> + 'a {}

impl<'a, T> Formable<'a> for T where
    T: Clone + Eq + Hash + PartialEq + Show<'a> + Spanned<'a> + 'a {}

pub trait Source<'a, Input>: Peekable<'a, Input>
where
    Input: Formable<'a> {}

impl<'a, Target, Input> Source<'a, Input> for Target
where
    Target: Peekable<'a, Input>,
    Input: Formable<'a> {}

pub trait Action<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    fn action(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    );
}

pub struct Multiple<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub actions: Vec<Rc<dyn Action<'a, Input, Output, Failure> + 'a>>,
}

pub struct Ignore;

pub struct Skip;

pub struct Transform<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub transformer: Rc<dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        &mut Classifier<'a, Input, Output, Failure>,
    ) -> Result<(), Failure> + 'a>,
}

pub struct Fail<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub emitter: Rc<dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        Classifier<'a, Input, Output, Failure>,
    ) -> Failure + 'a>,
}

pub struct Panic<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub emitter: Rc<dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        Classifier<'a, Input, Output, Failure>,
    ) -> Failure + 'a>,
}

#[derive(Clone)]
pub struct Literal<'a, Input> {
    pub value: Rc<dyn PartialEq<Input> + 'a>,
}

#[derive(Clone)]
pub struct Predicate<'a, Input: Formable<'a>> {
    pub function: Rc<dyn Fn(&Input) -> bool + 'a>,
}

pub struct Deferred<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub factory: fn() -> Classifier<'a, Input, Output, Failure>,
}

#[derive(Clone)]
pub struct Optional<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
}

#[derive(Clone)]
pub struct Alternative<
    'a,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
    const SIZE: Scale,
> {
    pub patterns: [Classifier<'a, Input, Output, Failure>; SIZE],
}

#[derive(Clone)]
pub struct Sequence<
    'a,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
    const SIZE: Scale,
> {
    pub patterns: [Classifier<'a, Input, Output, Failure>; SIZE],
}

#[derive(Clone)]
pub struct Repetition<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub classifier: Box<Classifier<'a, Input, Output, Failure>>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}