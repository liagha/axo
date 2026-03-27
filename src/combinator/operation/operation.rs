use {
    crate::{
        combinator::{
            next_identity, Action, Alternative, Command, Condition, Multiple, Operator, Repetition,
            Sequence, Trigger,
        },
        data::{memory::Rc, Identity, Scale},
    },
    std::time::{Duration, SystemTime},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Resolved,
    Rejected,
}

pub struct Operation<'source> {
    pub identity: Identity,
    pub action: Rc<dyn Action<'static, Operator, Self> + 'source>,
    pub status: Status,
    pub depth: Scale,
    pub stack: Vec<Identity>,
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
        }
    }

    #[inline]
    pub fn create(
        action: Rc<dyn Action<'static, Operator, Self> + 'source>,
        status: Status,
        depth: Scale,
        stack: Vec<Identity>,
    ) -> Self {
        Self {
            identity: next_identity(),
            action,
            status,
            depth,
            stack,
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
        matches!(self.status, Status::Resolved)
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
    pub fn set_resolve(&mut self) {
        self.status = Status::Resolved;
    }

    #[inline]
    pub fn set_reject(&mut self) {
        self.status = Status::Rejected;
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
    pub fn sequence<const SIZE: Scale>(nodes: [Self; SIZE]) -> Self {
        Self::new(Rc::new(Sequence {
            states: nodes,
            halt: |state| state.is_rejected() || state.is_pending(),
            keep: |state| state.is_resolved(),
        }))
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(nodes: [Self; SIZE]) -> Self {
        Self::new(Rc::new(Alternative {
            states: nodes,
            halt: |state| state.is_resolved() || state.is_pending(),
            compare: |new, old| new.is_resolved() && old.is_rejected(),
        }))
    }

    #[inline]
    pub fn repetition(node: Self, minimum: Scale, maximum: Option<Scale>) -> Self {
        Self::new(Rc::new(Repetition {
            state: Box::new(node),
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
}
