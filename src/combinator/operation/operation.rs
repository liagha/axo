use crate::{
    combinator::{
        next_identity, Alternative, Combinator, Command, Condition, Cycle, Multiple, Operator,
        Plan, Repetition, Sequence, Transform, Trigger,
    },
    data::{memory::Arc, memory::PhantomData, Identity, Scale},
    internal::time::{Duration, SystemTime},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Resolved(Vec<u8>),
    Rejected,
}

pub struct Operation<'source, Store = ()> {
    pub identity: Identity,
    pub combinator: Arc<dyn Combinator<'static, Operator<Store>, Self> + Send + Sync + 'source>,
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

impl<'source, Store: Clone + Send + Sync + 'source> Operation<'source, Store> {
    #[inline]
    pub fn new(
        combinator: Arc<dyn Combinator<'static, Operator<Store>, Self> + Send + Sync + 'source>,
    ) -> Self {
        Self {
            identity: next_identity(),
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
        combinator: Arc<dyn Combinator<'static, Operator<Store>, Self> + Send + Sync + 'source>,
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
        Self::new(Arc::new(Sequence {
            states,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(states: [Self; SIZE]) -> Self {
        Self::new(Arc::new(Alternative {
            states,
            halt: |state| state.is_resolved() || state.is_pending(),
            compare: |new, old| new.is_resolved() && old.is_rejected(),
        }))
    }

    #[inline]
    pub fn repetition(state: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(Arc::new(Repetition {
            state: Box::new(state),
            minimum,
            maximum,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn cycle(state: Self) -> Self {
        Self::new(Arc::new(Cycle {
            state: Box::new(state),
            keep: |state| matches!(&state.status, Status::Resolved(data) if !data.is_empty()),
        }))
    }

    #[inline]
    pub fn multiple(
        combinators: Vec<
            Arc<dyn Combinator<'static, Operator<Store>, Self> + Send + Sync + 'source>,
        >,
    ) -> Self {
        Self::new(Arc::new(Multiple { combinators }))
    }

    #[inline]
    pub fn plan(states: Vec<Self>) -> Self {
        Self::new(Arc::new(Plan { states }))
    }

    #[inline]
    pub fn map(mut state: Self, transform: fn(Vec<u8>) -> Vec<u8>) -> Self {
        let combinator = state.combinator.clone();
        state.combinator = Arc::new(Transform::<'static, 'source, Operator<Store>, Self, ()> {
            transformer: Arc::new(move |operator, operation| {
                combinator.combinator(operator, operation);
                if let Status::Resolved(data) = &operation.status {
                    operation.status = Status::Resolved(transform(data.clone()));
                }
                Ok(())
            }),
            phantom: PhantomData,
        });
        state
    }
}
