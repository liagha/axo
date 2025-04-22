use crate::axo_matcher::MatchType;

pub trait SimilarityMetric<Q, C> {
    fn calculate(&self, query: &Q, candidate: &C) -> f64;
    fn name(&self) -> &str;

    fn is_exact_match(&self, query: &Q, candidate: &C) -> bool {
        self.calculate(query, candidate) >= 0.9999
    }

    fn match_type(&self, query: &Q, candidate: &C) -> Option<MatchType> {
        let score = self.calculate(query, candidate);

        if self.is_exact_match(query, candidate) {
            Some(MatchType::Exact)
        } else if score > 0.0 {
            Some(MatchType::Similar(self.name().to_string()))
        } else {
            None
        }
    }
}

pub struct WeightedMetric<Q, C> {
    pub metric: Box<dyn SimilarityMetric<Q, C>>,
    pub weight: f64,
}

impl<Q, C> WeightedMetric<Q, C> {
    pub fn new<M: SimilarityMetric<Q, C> + 'static>(metric: M, weight: f64) -> Self {
        WeightedMetric {
            metric: Box::new(metric),
            weight,
        }
    }
}