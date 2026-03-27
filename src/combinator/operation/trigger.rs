use {
    crate::{
        combinator::{Action, Operation, Operator},
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

pub struct Trigger<'source> {
    pub condition: Condition,
    pub action: Rc<dyn Action<'static, Operator, Operation<'source>> + 'source>,
}
