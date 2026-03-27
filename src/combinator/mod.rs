// src/combinator/mod.rs
use crate::data::{sync::{AtomicUsize, Ordering}, memory::Rc, Identity, Scale, Boolean};

mod formation;
mod operation;

pub use formation::*;
pub use operation::*;

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(super) fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

use crate::{
    format::Show,
    internal::hash::Hash,
    data::memory::PhantomData,
    tracker::{Spanned},
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

pub struct Transform<'a, 'source, Host, State, Failure> {
    pub transformer: Rc<dyn Fn(
        &mut Host,
        &mut State,
    ) -> Result<(), Failure> + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Fail<'a, 'source, Host, State, Failure> {
    pub emitter: Rc<dyn Fn(
        &mut Host,
        State,
    ) -> Failure + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Panic<'a, 'source, Host, State, Failure> {
    pub emitter: Rc<dyn Fn(
        &mut Host,
        State,
    ) -> Failure + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Literal<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub value: Rc<dyn PartialEq<Input> + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Predicate<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub function: Rc<dyn Fn(&Input) -> bool + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Deferred<State> {
    pub factory: fn() -> State,
}

pub struct Optional<State> {
    pub state: Box<State>,
}

pub struct Alternative<State, const SIZE: Scale> {
    pub states: [State; SIZE],
}

pub struct Sequence<State, const SIZE: Scale> {
    pub states: [State; SIZE],
}

pub struct Repetition<State> {
    pub state: Box<State>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub persist: Boolean,
}

pub struct Command<'a, 'source, Host, State> {
    pub runner: Rc<dyn Fn(&mut Host, &mut State) + 'source>,
    pub phantom: PhantomData<&'a ()>,
}