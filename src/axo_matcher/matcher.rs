#![allow(dead_code)]

use {
    axo_hash::HashMap,
    core::fmt::Debug,
    core::marker::PhantomData,
    crate::{
        axo_matcher::{
            common::*,
            MatchInfo, MatchType,
        }
    }
};

pub struct Matcher<Q, C> {
    pub metrics: Vec<WeightedMetric<Q, C>>,
    pub threshold: f64,
    pub config: HashMap<String, String>,
    _phantom_q: PhantomData<Q>,
    _phantom_c: PhantomData<C>,
}

impl<Q: Clone + PartialEq + Debug, C: Clone + PartialEq + Debug> Default for Matcher<Q, C> {
    fn default() -> Self {
        Matcher {
            metrics: Vec::new(),
            threshold: 0.4,
            config: HashMap::new(),
            _phantom_q: PhantomData,
            _phantom_c: PhantomData,
        }
    }
}

impl<Q: Clone + PartialEq + Debug, C: Clone + PartialEq + Debug> Matcher<Q, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metric<M: SimilarityMetric<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        self.metrics.push(WeightedMetric::new(metric, weight));
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    pub fn add_metric<M: SimilarityMetric<Q, C> + 'static>(&mut self, metric: M, weight: f64) -> &mut Self {
        self.metrics.push(WeightedMetric::new(metric, weight));
        self
    }

    pub fn set_threshold(&mut self, threshold: f64) -> &mut Self {
        self.threshold = threshold;
        self
    }

    pub fn add_config(&mut self, key: &str, value: &str) -> &mut Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    pub fn find_best_match(&self, query: &Q, candidates: &[C]) -> Option<MatchInfo<Q, C>> {
        if candidates.is_empty() {
            return None;
        }

        for candidate in candidates {
            for weighted_metric in &self.metrics {
                if weighted_metric.metric.is_exact_match(query, candidate) {
                    return Some(MatchInfo {
                        score: 1.0,
                        query: query.clone(),
                        value: candidate.clone(),
                        match_type: MatchType::Exact,
                    });
                }
            }
        }

        let mut best_match: Option<MatchInfo<Q, C>> = None;
        let mut best_score = self.threshold;
        let mut _best_match_type = MatchType::NotFound;

        for candidate in candidates {
            let (score, match_type) = self.calculate_combined_similarity(query, candidate);

            if score > best_score {
                best_score = score;
                _best_match_type = match_type;
                best_match = Some(MatchInfo {
                    score,
                    query: query.clone(),
                    value: candidate.clone(),
                    match_type: _best_match_type.clone(),
                });
            }
        }

        best_match
    }

    pub fn find_all_matches(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<MatchInfo<Q, C>> {
        let mut matches: Vec<MatchInfo<Q, C>> = Vec::new();

        for candidate in candidates {
            let mut is_exact = false;
            for weighted_metric in &self.metrics {
                if weighted_metric.metric.is_exact_match(query, candidate) {
                    matches.push(MatchInfo {
                        score: 1.0,
                        query: query.clone(),
                        value: candidate.clone(),
                        match_type: MatchType::Exact,
                    });
                    is_exact = true;
                    break;
                }
            }

            if is_exact {
                continue;
            }

            let (score, match_type) = self.calculate_combined_similarity(query, candidate);

            if score > self.threshold {
                matches.push(MatchInfo {
                    score,
                    query: query.clone(),
                    value: candidate.clone(),
                    match_type,
                });
            }
        }

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        if limit > 0 && matches.len() > limit {
            matches.truncate(limit);
        }

        matches
    }

    pub fn find_matches_by_threshold(&self, query: &Q, candidates: &[C], min_threshold: f64) -> Vec<MatchInfo<Q, C>> {
        let mut matches: Vec<MatchInfo<Q, C>> = Vec::new();
        let actual_threshold = min_threshold.max(self.threshold);

        for candidate in candidates {
            let mut is_exact = false;
            for weighted_metric in &self.metrics {
                if weighted_metric.metric.is_exact_match(query, candidate) {
                    matches.push(MatchInfo {
                        score: 1.0,
                        query: query.clone(),
                        value: candidate.clone(),
                        match_type: MatchType::Exact,
                    });
                    is_exact = true;
                    break;
                }
            }

            if is_exact {
                continue;
            }

            let (score, match_type) = self.calculate_combined_similarity(query, candidate);

            if score > actual_threshold {
                matches.push(MatchInfo {
                    score,
                    query: query.clone(),
                    value: candidate.clone(),
                    match_type,
                });
            }
        }

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        matches
    }

    pub fn is_match(&self, query: &Q, candidate: &C) -> bool {
        for weighted_metric in &self.metrics {
            if weighted_metric.metric.is_exact_match(query, candidate) {
                return true;
            }
        }

        let (score, _) = self.calculate_combined_similarity(query, candidate);
        score > self.threshold
    }

    pub fn get_match_score(&self, query: &Q, candidate: &C) -> (f64, MatchType) {
        for weighted_metric in &self.metrics {
            if weighted_metric.metric.is_exact_match(query, candidate) {
                return (1.0, MatchType::Exact);
            }
        }

        self.calculate_combined_similarity(query, candidate)
    }

    fn calculate_combined_similarity(&self, query: &Q, candidate: &C) -> (f64, MatchType) {
        if self.metrics.is_empty() {
            return (0.0, MatchType::NotFound);
        }

        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        let mut best_metric_name = String::new();
        let mut best_metric_score = 0.0;

        for weighted_metric in &self.metrics {
            let score = weighted_metric.metric.calculate(query, candidate);
            total_score += score * weighted_metric.weight;
            total_weight += weighted_metric.weight;

            if score > best_metric_score {
                best_metric_score = score;
                best_metric_name = weighted_metric.metric.name().to_string();
            }
        }

        let final_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };

        let match_type = if final_score >= self.threshold {
            MatchType::Similar(best_metric_name)
        } else {
            MatchType::NotFound
        };

        (final_score, match_type)
    }
}

pub struct MultiMatcher<Q, C> {
    matchers: Vec<Box<Matcher<Q, C>>>,
    threshold: f64,
}

impl<Q: Clone + PartialEq + Debug, C: Clone + PartialEq + Debug> Default for MultiMatcher<Q, C> {
    fn default() -> Self {
        MultiMatcher {
            matchers: Vec::new(),
            threshold: 0.4,
        }
    }
}

impl<Q: Clone + PartialEq + Debug, C: Clone + PartialEq + Debug> MultiMatcher<Q, C> {
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

    pub fn find_best_match(&self, query: &Q, candidates: &[C]) -> Option<MatchInfo<Q, C>> {
        if candidates.is_empty() || self.matchers.is_empty() {
            return None;
        }

        let mut best_match: Option<MatchInfo<Q, C>> = None;
        let mut best_score = self.threshold;

        for matcher in &self.matchers {
            if let Some(match_info) = matcher.find_best_match(query, candidates) {
                if match_info.score > best_score {
                    best_score = match_info.score;
                    best_match = Some(match_info);
                }
            }
        }

        best_match
    }

    pub fn find_all_matches(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<MatchInfo<Q, C>> {
        if candidates.is_empty() || self.matchers.is_empty() {
            return Vec::new();
        }

        let mut all_matches: Vec<MatchInfo<Q, C>> = Vec::new();

        for matcher in &self.matchers {
            let matches = matcher.find_all_matches(query, candidates, 0);
            all_matches.extend(matches);
        }

        all_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        all_matches.dedup_by(|a, b| a.value == b.value);

        if limit > 0 && all_matches.len() > limit {
            all_matches.truncate(limit);
        }

        all_matches
    }
}

pub struct MatcherBuilder<Q, C> {
    matcher: Matcher<Q, C>,
}

impl<Q: Clone + PartialEq + Debug, C: Clone + PartialEq + Debug> MatcherBuilder<Q, C> {
    pub fn new() -> Self {
        MatcherBuilder {
            matcher: Matcher::new(),
        }
    }

    pub fn add_metric<M: SimilarityMetric<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        self.matcher.add_metric(metric, weight);
        self
    }

    pub fn set_threshold(mut self, threshold: f64) -> Self {
        self.matcher.set_threshold(threshold);
        self
    }

    pub fn add_config(mut self, key: &str, value: &str) -> Self {
        self.matcher.add_config(key, value);
        self
    }

    pub fn build(self) -> Matcher<Q, C> {
        self.matcher
    }
}