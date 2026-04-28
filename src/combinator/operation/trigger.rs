use crate::{
    combinator::{Combinator, Operation, Operator},
    data::memory::Arc,
    internal::time::SystemTime,
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
    pub combinator: Arc<
        dyn Combinator<'static, Operator<Store>, Operation<'source, Store>> + Send + Sync + 'source,
    >,
}
