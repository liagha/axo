use crate::{
    combinator::Combinator,
    data::memory::Arc,
    internal::time::SystemTime,
};

use super::{Operation, Operator};

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
    pub combinator: Arc<dyn for<'op> Combinator<'static, (&'op mut Operator<Store>, &'op mut Operation<'source, Store>)> + Send + Sync + 'source>,
}