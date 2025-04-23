use crate::axo_matcher::{SimilarityMetric, MatchType};
use std::marker::PhantomData;

/// Strategy for combining multiple metrics
pub enum CompositeStrategy {
    /// Use the maximum score from any metric
    Maximum,
    /// Use the average score from all metrics
    Average,
    /// Use the first metric that returns a score above the threshold
    Fallback(f64),
    /// Weight metrics by position (earlier metrics have higher priority)
    Weighted(Vec<f64>),
}

/// Composite metric that combines multiple metrics using a strategy
pub struct CompositeSimilarity<Q, C> {
    id: String,
    metrics: Vec<Box<dyn SimilarityMetric<Q, C>>>,
    strategy: CompositeStrategy,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q, C> CompositeSimilarity<Q, C> {
    pub fn new<S: Into<String>>(id: S, strategy: CompositeStrategy) -> Self {
        Self {
            id: id.into(),
            metrics: Vec::new(),
            strategy,
            _phantom: PhantomData,
        }
    }

    pub fn add_metric<M: SimilarityMetric<Q, C> + 'static>(&mut self, metric: M) -> &mut Self {
        self.metrics.push(Box::new(metric));
        self
    }

    pub fn with_metric<M: SimilarityMetric<Q, C> + 'static>(mut self, metric: M) -> Self {
        self.add_metric(metric);
        self
    }
}

impl<Q, C> SimilarityMetric<Q, C> for CompositeSimilarity<Q, C> {
    fn calculate(&self, query: &Q, candidate: &C) -> f64 {
        if self.metrics.is_empty() {
            return 0.0;
        }

        let scores: Vec<f64> = self.metrics.iter()
            .map(|metric| metric.calculate(query, candidate))
            .collect();

        match &self.strategy {
            CompositeStrategy::Maximum => {
                scores.iter().cloned().fold(0.0, f64::max)
            },
            CompositeStrategy::Average => {
                if scores.is_empty() {
                    0.0
                } else {
                    scores.iter().sum::<f64>() / scores.len() as f64
                }
            },
            CompositeStrategy::Fallback(threshold) => {
                for score in &scores {
                    if score >= threshold {
                        return *score;
                    }
                }
                scores.last().cloned().unwrap_or(0.0)
            },
            CompositeStrategy::Weighted(weights) => {
                let mut total_score = 0.0;
                let mut total_weight = 0.0;

                for (i, score) in scores.iter().enumerate() {
                    let weight = if i < weights.len() { weights[i] } else { 1.0 };
                    total_score += score * weight;
                    total_weight += weight;
                }

                if total_weight > 0.0 {
                    total_score / total_weight
                } else {
                    0.0
                }
            }
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn match_type(&self, query: &Q, candidate: &C) -> Option<MatchType> {
        // First check if any metric provides an exact match
        for metric in &self.metrics {
            if metric.is_exact_match(query, candidate) {
                return Some(MatchType::Exact);
            }
        }

        // Otherwise calculate composite score
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar(self.id().to_string()))
        } else {
            None
        }
    }
}