#![allow(unused_variables, unused_mut)]

use axo_hash::HashMap;
use core::marker::PhantomData;
use core::fmt::Debug;
use crate::axo_matcher::{DetailedMatchResult, MatchResult, MatchType, MatcherConfig, MetricScore, SimilarityMetric};
use crate::axo_matcher::common::WeightedMetric;
use crate::axo_matcher::composite::{CompositeSimilarity, CompositeStrategy};

/// Core matcher implementation that combines multiple similarity metrics
pub struct Matcher<Q, C> {
    metrics: Vec<WeightedMetric<Q, C>>,
    config: MatcherConfig,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q: Clone + Debug, C: Clone + Debug> Default for Matcher<Q, C> {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            config: MatcherConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<Q: Clone + Debug, C: Clone + Debug> Matcher<Q, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metric<M: SimilarityMetric<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        self.metrics.push(WeightedMetric::new(metric, weight));
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.config.threshold = threshold;
        self
    }

    pub fn with_option(mut self, key: &str, value: &str) -> Self {
        if self.config.options.is_none() {
            self.config.options = Some(HashMap::new());
        }
        if let Some(options) = &mut self.config.options {
            options.insert(key.to_string(), value.to_string());
        }
        self
    }

    pub fn add_metric<M: SimilarityMetric<Q, C> + 'static>(&mut self, metric: M, weight: f64) -> &mut Self {
        self.metrics.push(WeightedMetric::new(metric, weight));
        self
    }

    pub fn set_threshold(&mut self, threshold: f64) -> &mut Self {
        self.config.threshold = threshold;
        self
    }

    pub fn add_option(&mut self, key: &str, value: &str) -> &mut Self {
        if self.config.options.is_none() {
            self.config.options = Some(HashMap::new());
        }
        if let Some(options) = &mut self.config.options {
            options.insert(key.to_string(), value.to_string());
        }
        self
    }

    /// Calculate scores from all metrics for a query-candidate pair
    pub fn get_metric_scores(&self, query: &Q, candidate: &C) -> Vec<MetricScore> {
        let mut scores = Vec::with_capacity(self.metrics.len());

        for weighted_metric in &self.metrics {
            let id = weighted_metric.metric.id().to_string();
            let raw_score = weighted_metric.metric.calculate(query, candidate);
            let weight = weighted_metric.weight;

            scores.push(MetricScore {
                id,
                raw_score,
                weight,
                weighted_score: raw_score * weight,
            });
        }

        scores.sort_by(|a, b| b.weighted_score.partial_cmp(&a.weighted_score).unwrap());
        scores
    }

    /// Calculate overall score from weighted metric scores
    fn calculate_overall_score(metric_scores: &[MetricScore]) -> f64 {
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for score in metric_scores {
            total_weighted_score += score.weighted_score;
            total_weight += score.weight;
        }

        if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        }
    }

    /// Get the best performing metric
    fn get_best_metric(metric_scores: &[MetricScore]) -> Option<String> {
        metric_scores.iter()
            .max_by(|a, b| a.raw_score.partial_cmp(&b.raw_score).unwrap())
            .map(|score| score.id.clone())
    }

    /// Analyze a single query-candidate pair with detailed results
    pub fn analyze(&self, query: &Q, candidate: &C) -> DetailedMatchResult<Q, C> {
        let metric_scores = self.get_metric_scores(query, candidate);

        let mut is_exact = false;
        for weighted_metric in &self.metrics {
            if weighted_metric.metric.is_exact_match(query, candidate) {
                is_exact = true;
                break;
            }
        }

        let overall_score = Self::calculate_overall_score(&metric_scores);
        let threshold = self.config.threshold;

        let match_type = if is_exact {
            MatchType::Exact
        } else if overall_score >= threshold {
            if let Some(best_metric) = Self::get_best_metric(&metric_scores) {
                MatchType::Similar(best_metric)
            } else {
                MatchType::NotFound
            }
        } else {
            MatchType::NotFound
        };

        DetailedMatchResult {
            query: query.clone(),
            candidate: candidate.clone(),
            score: overall_score,
            match_type,
            metric_scores,
            is_match: overall_score >= threshold,
        }
    }

    /// Find best match for query among candidates
    pub fn find_best_match(&self, query: &Q, candidates: &[C]) -> Option<MatchResult<Q, C>> {
        if candidates.is_empty() {
            return None;
        }

        candidates.iter()
            .map(|candidate| {
                let result = self.analyze(query, candidate);
                MatchResult {
                    score: result.score,
                    query: query.clone(),
                    candidate: candidate.clone(),
                    match_type: result.match_type,
                }
            })
            .filter(|info| info.score >= self.config.threshold)
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    /// Find all matches above threshold
    pub fn find_matches(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<MatchResult<Q, C>> {
        let mut matches = Vec::new();

        for candidate in candidates {
            let result = self.analyze(query, candidate);

            if result.is_match {
                matches.push(MatchResult {
                    score: result.score,
                    query: query.clone(),
                    candidate: candidate.clone(),
                    match_type: result.match_type,
                });
            }
        }

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        if limit > 0 && matches.len() > limit {
            matches.truncate(limit);
        }

        matches
    }

    /// Find matches above a specific threshold
    pub fn find_matches_by_threshold(&self, query: &Q, candidates: &[C], threshold: f64) -> Vec<MatchResult<Q, C>> {
        let actual_threshold = threshold.max(self.config.threshold);

        let mut matches = Vec::new();

        for candidate in candidates {
            let result = self.analyze(query, candidate);

            if result.score >= actual_threshold {
                matches.push(MatchResult {
                    score: result.score,
                    query: query.clone(),
                    candidate: candidate.clone(),
                    match_type: result.match_type,
                });
            }
        }

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        matches
    }

    /// Check if candidate matches query above threshold
    pub fn is_match(&self, query: &Q, candidate: &C) -> bool {
        let result = self.analyze(query, candidate);
        result.is_match
    }
}

/// Composite matcher that combines results from multiple matchers
pub struct MultiMatcher<Q, C> {
    matchers: Vec<Box<Matcher<Q, C>>>,
    threshold: f64,
}

impl<Q: Clone + Debug, C: Clone + Debug> Default for MultiMatcher<Q, C> {
    fn default() -> Self {
        Self {
            matchers: Vec::new(),
            threshold: 0.4,
        }
    }
}

impl<Q: Clone + Debug, C: Clone + Debug + PartialEq> MultiMatcher<Q, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_matcher(mut self, matcher: Matcher<Q, C>) -> Self {
        self.matchers.push(Box::new(matcher));
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn add_matcher(&mut self, matcher: Matcher<Q, C>) -> &mut Self {
        self.matchers.push(Box::new(matcher));
        self
    }

    pub fn set_threshold(&mut self, threshold: f64) -> &mut Self {
        self.threshold = threshold;
        self
    }

    /// Find best match across all matchers
    pub fn find_best_match(&self, query: &Q, candidates: &[C]) -> Option<MatchResult<Q, C>> {
        if candidates.is_empty() || self.matchers.is_empty() {
            return None;
        }

        let mut best_match = None;
        let mut best_score = self.threshold;

        for matcher in &self.matchers {
            if let Some(match_result) = matcher.find_best_match(query, candidates) {
                if match_result.score > best_score {
                    best_score = match_result.score;
                    best_match = Some(match_result);
                }
            }
        }

        best_match
    }

    /// Find all matches across all matchers
    pub fn find_matches(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<MatchResult<Q, C>> {
        if candidates.is_empty() || self.matchers.is_empty() {
            return Vec::new();
        }

        let mut all_matches = Vec::new();

        for matcher in &self.matchers {
            let matches = matcher.find_matches(query, candidates, 0);
            all_matches.extend(matches);
        }

        all_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Deduplicate matches by candidate
        all_matches.dedup_by(|a, b| a.candidate == b.candidate);

        if limit > 0 && all_matches.len() > limit {
            all_matches.truncate(limit);
        }

        all_matches
    }
}

// In matcher.rs, enhance the MatcherBuilder
pub struct MatcherBuilder<Q, C> {
    matcher: Matcher<Q, C>,
    metric_groups: Vec<(String, Vec<WeightedMetric<Q, C>>)>,
    current_group: Option<String>,
    conditional_thresholds: Vec<(Box<dyn Fn(&Q) -> bool>, f64)>,
}

impl<Q: Clone + Debug, C: Clone + Debug> MatcherBuilder<Q, C> {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
            metric_groups: Vec::new(),
            current_group: None,
            conditional_thresholds: Vec::new(),
        }
    }

    pub fn metric<M: SimilarityMetric<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        if let Some(group_name) = &self.current_group {
            // Add to current group
            let group = self.metric_groups.iter_mut()
                .find(|(name, _)| name == group_name)
                .unwrap();

            group.1.push(WeightedMetric::new(metric, weight));
        } else {
            // Add directly to matcher
            self.matcher.add_metric(metric, weight);
        }
        self
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        self.matcher.set_threshold(threshold);
        self
    }

    pub fn conditional_threshold<F>(mut self, condition: F, threshold: f64) -> Self
    where
        F: Fn(&Q) -> bool + 'static
    {
        self.conditional_thresholds.push((Box::new(condition), threshold));
        self
    }

    pub fn option(mut self, key: &str, value: &str) -> Self {
        self.matcher.add_option(key, value);
        self
    }

    pub fn begin_group(mut self, name: &str) -> Self {
        self.current_group = Some(name.to_string());
        // Create group if it doesn't exist
        if !self.metric_groups.iter().any(|(n, _)| n == name) {
            self.metric_groups.push((name.to_string(), Vec::new()));
        }
        self
    }

    pub fn end_group(mut self) -> Self {
        self.current_group = None;
        self
    }

    pub fn with_fallback_chain(mut self, threshold: f64) -> Self {
        if self.metric_groups.is_empty() {
            return self;
        }

        // Create a composite fallback chain from all groups
        let mut composite: CompositeSimilarity<Q, C> = CompositeSimilarity::new("fallback_chain", CompositeStrategy::Fallback(threshold));

        for (_, metrics) in &self.metric_groups {
            for weighted_metric in metrics {
                // Clone the trait object - this would require modifying WeightedMetric to support cloning
                // For now, we'd need to reconstruct the metrics when building a fallback chain
                // This is a simplification for the example
                // composite.add_metric(weighted_metric.metric.clone());
            }
        }

        // The actual implementation would add the composite metric
        // self.matcher.add_metric(composite, 1.0);

        self
    }

    pub fn priority_chain(mut self) -> Self {
        // Similar to fallback chain but using weighted strategy
        // Would implement similar logic as above with CompositeStrategy::Weighted
        self
    }

    pub fn dynamic_threshold<F>(mut self, threshold_fn: F) -> Self
    where
        F: Fn(&Q, &C) -> f64 + 'static
    {
        // Would store the function and modify the matcher to use it
        // This is conceptual since it would require deeper changes to Matcher
        self
    }

    pub fn build(self) -> Matcher<Q, C> {
        // Apply any conditional thresholds or other deferred configurations
        let mut matcher = self.matcher;

        // Add dynamic threshold resolution
        if !self.conditional_thresholds.is_empty() {
            matcher.add_option("has_conditional_thresholds", "true");
            // In practice, we'd store these conditions for runtime evaluation
        }

        matcher
    }
}