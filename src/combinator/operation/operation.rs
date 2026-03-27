use {
    crate::{
        combinator::{
            next_identity, Action, Alternative, Command, Condition, Plan, Multiple, Operator,
            Repetition, Sequence, Transform, Trigger,
        },
        data::{memory::PhantomData, memory::Rc, Identity, Scale},
        internal::time::{Duration, SystemTime},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Resolved(Vec<u8>),
    Rejected,
}

pub struct Operation<'source> {
    pub identity: Identity,
    pub action: Rc<dyn Action<'static, Operator, Self> + 'source>,
    pub status: Status,
    pub depth: Scale,
    pub stack: Vec<Identity>,
    pub payload: Vec<u8>,
    pub depends: Vec<Identity>,
}

impl<'source> Operation<'source> {
    #[inline]
    pub fn new(action: Rc<dyn Action<'static, Operator, Self> + 'source>) -> Self {
        Self {
            identity: next_identity(),
            action,
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
        action: Rc<dyn Action<'static, Operator, Self> + 'source>,
        status: Status,
        depth: Scale,
        stack: Vec<Identity>,
        payload: Vec<u8>,
        depends: Vec<Identity>,
    ) -> Self {
        Self {
            identity,
            action,
            status,
            depth,
            stack,
            payload,
            depends,
        }
    }

    #[inline]
    pub fn execute(&mut self, operator: &mut Operator) -> Status {
        operator.execute(self)
    }

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

    #[inline]
    pub fn delay(mut self, duration: Duration) -> Self {
        self.action = Rc::new(Trigger {
            condition: Condition::Time(SystemTime::now() + duration),
            action: self.action.clone(),
        });
        self
    }

    #[inline]
    pub fn wait(mut self, time: SystemTime) -> Self {
        self.action = Rc::new(Trigger {
            condition: Condition::Time(time),
            action: self.action.clone(),
        });
        self
    }

    #[inline]
    pub fn trigger(mut self, condition: Condition) -> Self {
        self.action = Rc::new(Trigger {
            condition,
            action: self.action.clone(),
        });
        self
    }

    #[inline]
    pub fn command(command: Command) -> Self {
        Self::new(Rc::new(command))
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(states: [Self; SIZE]) -> Self {
        Self::new(Rc::new(Sequence {
            states,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(states: [Self; SIZE]) -> Self {
        Self::new(Rc::new(Alternative {
            states,
            halt: |state| state.is_resolved() || state.is_pending(),
            compare: |new, old| new.is_resolved() && old.is_rejected(),
        }))
    }

    #[inline]
    pub fn repetition(state: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(Rc::new(Repetition {
            state: Box::new(state),
            minimum,
            maximum,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn multiple(actions: Vec<Rc<dyn Action<'static, Operator, Self> + 'source>>) -> Self {
        Self::new(Rc::new(Multiple { actions }))
    }

    #[inline]
    pub fn plan(states: Vec<Self>) -> Self {
        Self::new(Rc::new(Plan { states }))
    }

    #[inline]
    pub fn map(mut state: Self, transform: fn(Vec<u8>) -> Vec<u8>) -> Self {
        let action = state.action.clone();
        state.action = Rc::new(Transform::<'static, 'source, Operator, Self, ()> {
            transformer: Rc::new(move |operator, operation| {
                action.action(operator, operation);
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