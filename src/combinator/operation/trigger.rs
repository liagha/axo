use {
    crate::{
        combinator::{Action, Formable, Operation, Operator},
        data::memory::Rc,
    },
    std::time::SystemTime,
};

#[derive(Clone)]
pub enum Condition {
    Always,
    Time(SystemTime),
    Evaluate(fn() -> bool),
}

pub struct Trigger<'a, 'source, Input, Output, Failure>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub condition: Condition,
    pub action: Rc<
        dyn Action<
            'a,
            Operator<'a, Input, Output, Failure>,
            Operation<'a, 'source, Input, Output, Failure>,
        > + 'source,
    >,
}
