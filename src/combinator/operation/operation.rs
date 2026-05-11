use crate::{
    combinator::{Combinator, Command, Condition, Trigger},
    data::{memory::Arc, Identity, Scale},
    internal::time::{Duration, SystemTime},
};

use super::{Operator, Plan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Resolved(Vec<u8>),
    Rejected,
}

type Combinator_<'source, Store> = dyn for<'op> Combinator<
'static,
(&'op mut Operator<Store>, &'op mut Operation<'source, Store>),
> + Send + Sync + 'source;

pub struct Mapper<'source, Store> {
    pub inner: Arc<Combinator_<'source, Store>>,
    pub transform: fn(Vec<u8>) -> Vec<u8>,
}

pub struct Operation<'source, Store = ()> {
    pub identity: Identity,
    pub combinator: Arc<Combinator_<'source, Store>>,
    pub status: Status,
    pub depth: Scale,
    pub stack: Vec<Identity>,
    pub payload: Vec<u8>,
    pub depends: Vec<Identity>,
}

impl<'source, Store> Operation<'source, Store> {
    #[inline]
    pub const fn is_pending(&self) -> bool {
        matches!(self.status, Status::Pending)
    }

    #[inline]
    pub const fn is_resolved(&self) -> bool {
        matches!(self.status, Status::Resolved(_))
    }

    #[inline]
    pub const fn is_rejected(&self) -> bool {
        matches!(self.status, Status::Rejected)
    }

    #[inline]
    pub fn set_pending(&mut self) {
        self.status = Status::Pending;
    }

    #[inline]
    pub fn set_resolve(&mut self, payload: Vec<u8>) {
        self.status = Status::Resolved(payload);
    }

    #[inline]
    pub fn set_reject(&mut self) {
        self.status = Status::Rejected;
    }

    #[inline]
    pub fn depend(mut self, identity: Identity) -> Self {
        self.depends.push(identity);
        self
    }
}

impl<'source, Store: Clone + Send + Sync + 'static> Operation<'source, Store> {
    #[inline]
    pub fn new(combinator: Arc<Combinator_<'source, Store>>) -> Self {
        Self {
            identity: crate::combinator::next_identity(),
            combinator,
            status: Status::Pending,
            depth: 0,
            stack: Vec::new(),
            payload: Vec::new(),
            depends: Vec::new(),
        }
    }

    #[inline]
    pub fn create(
        identity: Identity,
        combinator: Arc<Combinator_<'source, Store>>,
        status: Status,
        depth: Scale,
        stack: Vec<Identity>,
        payload: Vec<u8>,
        depends: Vec<Identity>,
    ) -> Self {
        Self {
            identity,
            combinator,
            status,
            depth,
            stack,
            payload,
            depends,
        }
    }

    #[inline]
    pub fn execute(&mut self, operator: &mut Operator<Store>) -> Status {
        operator.execute(self)
    }

    #[inline]
    pub fn delay(mut self, duration: Duration) -> Self {
        self.combinator = Arc::new(Trigger {
            condition: Condition::Time(SystemTime::now() + duration),
            combinator: self.combinator.clone(),
        });
        self
    }

    #[inline]
    pub fn wait(mut self, time: SystemTime) -> Self {
        self.combinator = Arc::new(Trigger {
            condition: Condition::Time(time),
            combinator: self.combinator.clone(),
        });
        self
    }

    #[inline]
    pub fn trigger(mut self, condition: Condition) -> Self {
        self.combinator = Arc::new(Trigger {
            condition,
            combinator: self.combinator.clone(),
        });
        self
    }

    #[inline]
    pub fn command(command: Command) -> Self {
        Self::new(Arc::new(command))
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(states: [Self; SIZE]) -> Self {
        Self::new(Arc::new(crate::combinator::Sequence {
            states,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(states: [Self; SIZE]) -> Self {
        Self::new(Arc::new(crate::combinator::Alternative {
            states,
            halt: |state| state.is_resolved() || state.is_pending(),
            compare: |new, old| new.is_resolved() && old.is_rejected(),
        }))
    }

    #[inline]
    pub fn repetition(state: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(Arc::new(crate::combinator::Repetition {
            state: Box::new(state),
            minimum,
            maximum,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn cycle(state: Self) -> Self {
        Self::new(Arc::new(crate::combinator::Cycle {
            state: Box::new(state),
            keep: |state| matches!(&state.status, Status::Resolved(data) if !data.is_empty()),
        }))
    }

    #[inline]
    pub fn multiple(steps: Vec<Arc<Combinator_<'source, Store>>>) -> Self {
        Self::new(Arc::new(super::combinator::Many { steps }))
    }

    #[inline]
    pub fn plan(states: Vec<Self>) -> Self {
        Self::new(Arc::new(Plan { states }))
    }

    #[inline]
    pub fn map(mut state: Self, transform: fn(Vec<u8>) -> Vec<u8>) -> Self {
        state.combinator = Arc::new(Mapper {
            inner: state.combinator.clone(),
            transform,
        });
        state
    }
}