use core::fmt::Debug;
use axo_hash::HashMap;

/// Represents a match result with score and type information
#[derive(Debug, Clone)]
pub struct MatchResult<Q, C> {
    pub score: f64,
    pub query: Q,
    pub candidate: C,
    pub match_type: MatchType,
}

/// Type of match found between query and candidate
#[derive(Debug, PartialEq, Clone)]
pub enum MatchType {
    Exact,
    Similar(String),
    NotFound,
}

/// Score details for a single metric
#[derive(Debug, Clone)]
pub struct MetricScore {
    pub id: String,
    pub raw_score: f64,
    pub weight: f64,
    pub weighted_score: f64,
}

/// Detailed match analysis with all metric scores
#[derive(Debug, Clone)]
pub struct DetailedMatchResult<Q, C> {
    pub query: Q,
    pub candidate: C,
    pub score: f64,
    pub match_type: MatchType,
    pub metric_scores: Vec<MetricScore>,
    pub is_match: bool,
}

/// Configuration for matcher behavior
#[derive(Debug, Clone)]
pub struct MatcherConfig {
    pub threshold: f64,
    pub options: Option<HashMap<String, String>>,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self {
            threshold: 0.4,
            options: None,
        }
    }
}