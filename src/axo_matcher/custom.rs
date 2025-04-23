use crate::axo_matcher::SimilarityMetric;
use std::marker::PhantomData;

/// Closure-based similarity metric
pub struct ClosureMetric<Q, C, F>
where
    F: Fn(&Q, &C) -> f64,
{
    id: String,
    calculate_fn: F,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q, C, F> ClosureMetric<Q, C, F>
where
    F: Fn(&Q, &C) -> f64,
{
    pub fn new<S: Into<String>>(id: S, calculate_fn: F) -> Self {
        Self {
            id: id.into(),
            calculate_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Q, C, F> SimilarityMetric<Q, C> for ClosureMetric<Q, C, F>
where
    F: Fn(&Q, &C) -> f64,
{
    fn calculate(&self, query: &Q, candidate: &C) -> f64 {
        (self.calculate_fn)(query, candidate)
    }

    fn id(&self) -> &str {
        &self.id
    }
}