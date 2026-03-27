use {
    crate::{
        combinator::{Action, Operation, Operator},
        data::memory::Rc,
        internal::time::SystemTime,
    },
};

#[derive(Clone)]
pub enum Condition {
    Always,
    Time(SystemTime),
    Evaluate(fn() -> bool),
    Outdated(String, String),
    Missing(String),
}

pub struct Trigger<'source> {
    pub condition: Condition,
    pub action: Rc<dyn Action<'static, Operator, Operation<'source>> + 'source>,
}
