// src/combinator/mod.rs

use crate::data::{
    memory::Arc,
    sync::{AtomicUsize, Ordering},
    Identity, Scale,
};

mod formation;
mod operation;

pub use formation::*;
pub use operation::*;

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(super) fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

use crate::{data::memory::PhantomData, format::Show, internal::hash::Hash};

pub trait Formable<'a>: Clone + Eq + Hash + PartialEq + Show<'a> + 'a {}

impl<'a, T> Formable<'a> for T where T: Clone + Eq + Hash + PartialEq + Show<'a> + 'a {}

pub trait Combinator<'a, Host, State>: Send + Sync {
    fn combinator(&self, host: &mut Host, state: &mut State);
}

pub struct Multiple<'a, 'source, Host, State> {
    pub combinators: Vec<Arc<dyn Combinator<'a, Host, State> + Send + Sync + 'source>>,
}


pub struct Resolve;

pub struct Depend;

pub struct Pulse {
    pub idle: u64,
}
pub struct Ignore;

pub struct Skip;

pub struct Transform<'a, 'source, Host, State, Failure> {
    pub transformer:
        Arc<dyn Fn(&mut Host, &mut State) -> Result<(), Failure> + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Fail<'a, 'source, Host, State, Failure> {
    pub emitter: Arc<dyn Fn(&mut Host, State) -> Failure + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Panic<'a, 'source, Host, State, Failure> {
    pub emitter: Arc<dyn Fn(&mut Host, State) -> Failure + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Recover<'a, 'source, Host, State, Input, Failure> {
    pub sync: Arc<dyn Fn(&Input) -> bool + Send + Sync + 'source>,
    pub emitter: Arc<dyn Fn(&mut Host, State) -> Failure + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Literal<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub value: Arc<dyn PartialEq<Input> + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Predicate<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub function: Arc<dyn Fn(&Input) -> bool + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Deferred<State> {
    pub factory: fn() -> State,
}

pub struct Optional<State> {
    pub state: Box<State>,
}

pub struct Snapshot<State> {
    pub state: Box<State>,
}

pub struct Group<State> {
    pub state: Box<State>,
}

pub struct Alternative<State, const SIZE: Scale> {
    pub states: [State; SIZE],
    pub halt: fn(&State) -> bool,
    pub compare: fn(&State, &State) -> bool,
}

pub struct Sequence<State, const SIZE: Scale> {
    pub states: [State; SIZE],
    pub halt: fn(&State) -> bool,
    pub keep: fn(&State) -> bool,
}

pub struct Repetition<State> {
    pub state: Box<State>,
    pub minimum: Scale,
    pub maximum: Option<Scale>,
    pub halt: fn(&State) -> bool,
    pub keep: fn(&State) -> bool,
}

pub struct Cycle<State> {
    pub state: Box<State>,
    pub keep: fn(&State) -> bool,
}
