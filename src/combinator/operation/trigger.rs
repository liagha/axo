use {
    crate::{
        combinator::{Action, Operation, Operator},
        data::memory::Arc,
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

pub struct Trigger<'source, Store = ()> {
    pub condition: Condition,
    pub action: Arc<dyn Action<'static, Operator<Store>, Operation<'source, Store>> + Send + Sync + 'source>,
}
