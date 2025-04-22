use core::fmt::Debug;

#[derive(Debug)]
pub struct MatchInfo<Q: Clone, V: Clone> {
    pub score: f64,
    pub query: Q,
    pub value: V,
    pub match_type: MatchType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MatchType {
    Exact,
    Similar(String),
    NotFound,
}