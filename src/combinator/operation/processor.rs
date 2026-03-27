use crate::{
    combinator::{
        Action, Operator, Command, Formable, next_identity, Multiple, Sequence, Alternative,
    },
    data::{
        memory::{Rc},
        Identity, Scale,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Active,
    Resolved,
    Rejected,
}

pub struct Processor<'a: 'source, 'source, Data, Output, Failure>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub identity: Identity,
    pub action: Rc<dyn Action<'a, Operator<'a, Data, Output, Failure>, Self> + 'source>,
    pub status: Status,
    pub depth: Scale,
    pub stack: Vec<Identity>,
}

impl<'a: 'source, 'source, Data, Output, Failure> Processor<'a, 'source, Data, Output, Failure>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn new(
        action: Rc<dyn Action<'a, Operator<'a, Data, Output, Failure>, Self> + 'source>,
    ) -> Self {
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
        action: Rc<dyn Action<'a, Operator<'a, Data, Output, Failure>, Self> + 'source>,
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
    pub fn command<F>(runner: F) -> Self
    where
        F: Fn(&mut Operator<'a, Data, Output, Failure>, &mut Self) + 'source,
    {
        Self::new(Rc::new(Command {
            runner: Rc::new(runner),
            phantom: Default::default(),
        }))
    }

    #[inline]
    pub fn sequence<const SIZE: Scale>(nodes: [Self; SIZE]) -> Self {
        Self::new(Rc::new(Sequence { states: nodes }))
    }

    #[inline]
    pub fn alternative<const SIZE: Scale>(nodes: [Self; SIZE]) -> Self {
        Self::new(Rc::new(Alternative { states: nodes }))
    }

    #[inline]
    pub fn multiple(
        actions: Vec<Rc<dyn Action<'a, Operator<'a, Data, Output, Failure>, Self> + 'source>>,
    ) -> Self {
        Self::new(Rc::new(Multiple { actions }))
    }

    #[inline]
    pub fn resolve(&mut self) {
        self.status = Status::Resolved;
    }

    #[inline]
    pub fn reject(&mut self) {
        self.status = Status::Rejected;
    }
}
