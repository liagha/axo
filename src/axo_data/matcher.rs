#![allow(dead_code)]

use core::cmp::{max, min};
use core::hash::Hash;
use std::fmt::Debug;
use hashbrown::HashMap;
use std::marker::PhantomData;

// Match information structure - now supports different types for Query and Value
#[derive(Debug)]
pub struct MatchInfo<Q: Clone, V: Clone> {
    pub score: f64,            // Overall similarity score (0.0 to 1.0)
    pub query: Q,              // The original query
    pub value: V,              // The matched value
    pub match_type: MatchType, // Type of match found
}

#[derive(Debug, PartialEq, Clone)]
pub enum MatchType {
    Exact,              // Perfect match
    Similar(String),    // Similar match with reason
    NotFound,           // No match found above threshold
}

// Trait for a similarity metric that can be applied to different data types
pub trait SimilarityMetric<Q, C> {
    // Calculate similarity between query and candidate, returning a score between 0.0 and 1.0
    fn calculate(&self, query: &Q, candidate: &C) -> f64;

    // Name of the metric for debugging and configuration
    fn name(&self) -> &str;

    // Optional method to determine if this metric produced an exact match
    fn is_exact_match(&self, query: &Q, candidate: &C) -> bool {
        self.calculate(query, candidate) >= 0.9999
    }

    // Optional method to determine match type for this metric
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

// A weighted metric combines a metric with its weight
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

// The main Matcher struct, now generic over two types Q (query) and C (candidate)
pub struct Matcher<Q, C> {
    // Vector of metrics with their associated weights
    pub metrics: Vec<WeightedMetric<Q, C>>,

    // Threshold below which matches are considered "not found"
    pub threshold: f64,

    // Any additional configuration parameters can be added here
    pub config: HashMap<String, String>,

    // Phantom data to help with type inference
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

    // Add a new metric with a weight
    pub fn with_metric<M: SimilarityMetric<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        self.metrics.push(WeightedMetric::new(metric, weight));
        self
    }

    // Set the threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    // Add a configuration parameter
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    // Add a metric to an existing matcher
    pub fn add_metric<M: SimilarityMetric<Q, C> + 'static>(&mut self, metric: M, weight: f64) -> &mut Self {
        self.metrics.push(WeightedMetric::new(metric, weight));
        self
    }

    // Find the best match for a query item from a list of candidates
    pub fn find_best_match(&self, query: &Q, candidates: &[C]) -> Option<MatchInfo<Q, C>> {
        if candidates.is_empty() {
            return None;
        }

        // First check for exact matches
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

        // Calculate weighted similarity for all candidates
        let mut best_match: Option<MatchInfo<Q, C>> = None;
        let mut best_score = self.threshold;
        let mut best_match_type = MatchType::NotFound;

        for candidate in candidates {
            let (score, match_type) = self.calculate_combined_similarity(query, candidate);

            if score > best_score {
                best_score = score;
                best_match_type = match_type;
                best_match = Some(MatchInfo {
                    score,
                    query: query.clone(),
                    value: candidate.clone(),
                    match_type: best_match_type.clone(),
                });
            }
        }

        best_match
    }

    // Find all matches above threshold, sorted by score
    pub fn find_all_matches(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<MatchInfo<Q, C>> {
        let mut matches: Vec<MatchInfo<Q, C>> = Vec::new();

        for candidate in candidates {
            // Check for exact matches
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

            // Calculate similarity
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

        // Sort by score descending
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Limit results if needed
        if limit > 0 && matches.len() > limit {
            matches.truncate(limit);
        }

        matches
    }

    // Calculate combined similarity with all metrics
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

            // Track which metric gave the highest score for match_type
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

// Implementations for string-specific metrics
// These can be used as examples for creating metrics for other data types

// Exact match metric - now works with different types if they implement PartialEq
pub struct ExactMatchMetric;

impl<Q, C> SimilarityMetric<Q, C> for ExactMatchMetric
where
    Q: PartialEq<C>,
{
    fn calculate(&self, query: &Q, candidate: &C) -> f64 {
        if query == candidate { 1.0 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "ExactMatch"
    }
}

// Case-insensitive match for strings
pub struct CaseInsensitiveMetric;

impl SimilarityMetric<String, String> for CaseInsensitiveMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "CaseInsensitive"
    }
}

// Also support &str with String
impl SimilarityMetric<&str, String> for CaseInsensitiveMetric {
    fn calculate(&self, query: &&str, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "CaseInsensitive"
    }
}

impl SimilarityMetric<String, &str> for CaseInsensitiveMetric {
    fn calculate(&self, query: &String, candidate: &&str) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "CaseInsensitive"
    }
}

// Prefix match metric
pub struct PrefixMetric;

impl SimilarityMetric<String, String> for PrefixMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            return 0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0);
        }
        0.0
    }

    fn name(&self) -> &str {
        "Prefix"
    }

    fn match_type(&self, query: &String, candidate: &String) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Prefix".to_string()))
        } else {
            None
        }
    }
}

// Also support &str with String for PrefixMetric
impl SimilarityMetric<&str, String> for PrefixMetric {
    fn calculate(&self, query: &&str, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            return 0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0);
        }
        0.0
    }

    fn name(&self) -> &str {
        "Prefix"
    }
}

// Suffix match metric
pub struct SuffixMetric;

impl SimilarityMetric<String, String> for SuffixMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.ends_with(&query_lower) {
            return 0.85 * (query.len() as f64 / candidate.len() as f64).min(1.0);
        }
        0.0
    }

    fn name(&self) -> &str {
        "Suffix"
    }
}

// Substring match metric
pub struct SubstringMetric;

impl SimilarityMetric<String, String> for SubstringMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.contains(&query_lower) {
            return 0.8 * (query.len() as f64 / candidate.len() as f64).min(1.0);
        }
        0.0
    }

    fn name(&self) -> &str {
        "Substring"
    }
}

// Edit distance metric using Damerau-Levenshtein
pub struct EditDistanceMetric;

impl SimilarityMetric<String, String> for EditDistanceMetric {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        let distance = damerau_levenshtein_distance(s1, s2);
        let max_len = max(s1.len(), s2.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    fn name(&self) -> &str {
        "EditDistance"
    }
}

// Token-based similarity metric for strings
pub struct TokenSimilarityMetric {
    // Configuration for token splitting
    pub separators: Vec<char>,
}

impl Default for TokenSimilarityMetric {
    fn default() -> Self {
        TokenSimilarityMetric {
            separators: vec!['_', '-', '.', ' '],
        }
    }
}

impl SimilarityMetric<String, String> for TokenSimilarityMetric {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        let s1_tokens = self.split_on_separators(&s1_lower);
        let s2_tokens = self.split_on_separators(&s2_lower);

        self.token_similarity(&s1_tokens, &s2_tokens)
    }

    fn name(&self) -> &str {
        "TokenSimilarity"
    }
}

impl TokenSimilarityMetric {
    pub fn new(separators: Vec<char>) -> Self {
        TokenSimilarityMetric { separators }
    }

    // Split a string on separators and handle camelCase
    pub fn split_on_separators(&self, s: &str) -> Vec<String> {
        // First split on common separators
        let mut tokens: Vec<String> = Vec::new();
        let mut current = String::new();

        for c in s.chars() {
            if self.separators.contains(&c) {
                if !current.is_empty() {
                    tokens.push(current);
                    current = String::new();
                }
            } else {
                // Handle camelCase: transition from lowercase to uppercase
                if !current.is_empty() && !c.is_uppercase() && c.is_uppercase() {
                    tokens.push(current);
                    current = String::new();
                }
                current.push(c);
            }
        }

        // Add the last token
        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    // Calculate similarity between token sets
    pub fn token_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let mut total_sim = 0.0;
        let mut matches = 0;

        for t1 in tokens1 {
            let mut best_sim : f64 = 0.0;

            for t2 in tokens2 {
                if t1 == t2 {
                    best_sim = 1.0;
                    break;
                }

                let edit_distance = damerau_levenshtein_distance(t1, t2);
                let max_len = max(t1.len(), t2.len());
                let token_sim = if max_len > 0 {
                    1.0 - (edit_distance as f64 / max_len as f64)
                } else {
                    0.0
                };

                best_sim = best_sim.max(token_sim);
            }

            total_sim += best_sim;
            if best_sim > 0.8 {
                matches += 1;
            }
        }

        // Normalize by number of tokens
        let token_sim = if !tokens1.is_empty() {
            total_sim / tokens1.len() as f64
        } else {
            0.0
        };

        // Boost for high number of matches
        let match_ratio = if !tokens1.is_empty() {
            matches as f64 / tokens1.len() as f64
        } else {
            0.0
        };

        token_sim * (1.0 + 0.5 * match_ratio)
    }
}

// Acronym matching metric
pub struct AcronymMetric {
    pub token_metric: TokenSimilarityMetric,
    pub max_acronym_length: usize,
}

impl Default for AcronymMetric {
    fn default() -> Self {
        AcronymMetric {
            token_metric: TokenSimilarityMetric::default(),
            max_acronym_length: 5,
        }
    }
}

impl SimilarityMetric<String, String> for AcronymMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        if query.len() > self.max_acronym_length {
            return 0.0;
        }

        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        // Split candidate on separators
        let tokens = self.token_metric.split_on_separators(&candidate_lower);

        if tokens.len() < query_lower.len() {
            return 0.0;
        }

        // Get first letters of each token
        let first_letters: String = tokens.iter()
            .filter_map(|token| token.chars().next())
            .collect();

        if first_letters.contains(&query_lower) {
            return 0.75;
        }

        0.0
    }

    fn name(&self) -> &str {
        "Acronym"
    }

    fn match_type(&self, query: &String, candidate: &String) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Acronym".to_string()))
        } else {
            None
        }
    }
}

// Keyboard proximity metric for typo detection
pub struct KeyboardProximityMetric {
    pub keyboard_layout: HashMap<char, Vec<char>>,
}

impl Default for KeyboardProximityMetric {
    fn default() -> Self {
        KeyboardProximityMetric {
            keyboard_layout: create_qwerty_layout(),
        }
    }
}

impl SimilarityMetric<String, String> for KeyboardProximityMetric {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Only consider strings of similar length
        if (s1_lower.len() as isize - s2_lower.len() as isize).abs() > 2 {
            return 0.0;
        }

        let s1_chars: Vec<char> = s1_lower.chars().collect();
        let s2_chars: Vec<char> = s2_lower.chars().collect();

        // Use Damerau-Levenshtein distance first
        let edit_distance = damerau_levenshtein_distance(&s1_lower, &s2_lower);

        // If edit distance is too large, return low similarity
        if edit_distance > 3 {
            return 0.0;
        }

        // Calculate keyboard adjacency for characters that differ
        let mut adjacency_count = 0;
        let mut max_comparisons = min(s1_chars.len(), s2_chars.len());

        for i in 0..max_comparisons {
            if s1_chars[i] == s2_chars[i] {
                continue;
            }

            // Check if characters are adjacent on keyboard
            if let Some(neighbors) = self.keyboard_layout.get(&s1_chars[i]) {
                if neighbors.contains(&s2_chars[i]) {
                    adjacency_count += 1;
                }
            }
        }

        let differing_chars = edit_distance;

        if differing_chars == 0 {
            1.0
        } else {
            let keyboard_factor = adjacency_count as f64 / differing_chars as f64;
            let length_similarity = 1.0 - ((s1_chars.len() as isize - s2_chars.len() as isize).abs() as f64 / max(s1_chars.len(), s2_chars.len()) as f64);

            // Combine keyboard proximity with edit distance normalization
            let base_similarity = 1.0 - (edit_distance as f64 / max(s1_chars.len(), s2_chars.len()) as f64);
            base_similarity * (1.0 + 0.3 * keyboard_factor) * length_similarity
        }
    }

    fn name(&self) -> &str {
        "KeyboardProximity"
    }
}

// Damerau-Levenshtein edit distance implementation
pub fn damerau_levenshtein_distance(s1: &str, s2: &str) -> usize {
    if s1 == s2 {
        return 0;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len_s1 = s1_chars.len();
    let len_s2 = s2_chars.len();

    // Handle edge cases
    if len_s1 == 0 {
        return len_s2;
    }
    if len_s2 == 0 {
        return len_s1;
    }

    // Initialize the distance matrix
    let mut matrix = vec![vec![0; len_s2 + 1]; len_s1 + 1];

    // Fill the first row and column
    for i in 0..=len_s1 {
        matrix[i][0] = i;
    }
    for j in 0..=len_s2 {
        matrix[0][j] = j;
    }

    // Fill the matrix using dynamic programming
    for i in 1..=len_s1 {
        for j in 1..=len_s2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };

            matrix[i][j] = min(
                matrix[i - 1][j] + 1,                // deletion
                min(
                    matrix[i][j - 1] + 1,            // insertion
                    matrix[i - 1][j - 1] + cost      // substitution
                )
            );

            // Check for transposition
            if i > 1 && j > 1 && s1_chars[i - 1] == s2_chars[j - 2] && s1_chars[i - 2] == s2_chars[j - 1] {
                matrix[i][j] = min(
                    matrix[i][j],
                    matrix[i - 2][j - 2] + cost      // transposition
                );
            }
        }
    }

    matrix[len_s1][len_s2]
}

// Create a QWERTY keyboard layout for English
pub fn create_qwerty_layout() -> HashMap<char, Vec<char>> {
    let mut layout = HashMap::new();

    // Define keyboard adjacency
    let adjacency_map = [
        ('q', vec!['w', 'a']),
        ('w', vec!['q', 'e', 'a', 's']),
        ('e', vec!['w', 'r', 's', 'd']),
        ('r', vec!['e', 't', 'd', 'f']),
        ('t', vec!['r', 'y', 'f', 'g']),
        ('y', vec!['t', 'u', 'g', 'h']),
        ('u', vec!['y', 'i', 'h', 'j']),
        ('i', vec!['u', 'o', 'j', 'k']),
        ('o', vec!['i', 'p', 'k', 'l']),
        ('p', vec!['o', 'l']),
        ('a', vec!['q', 'w', 's', 'z']),
        ('s', vec!['w', 'e', 'a', 'd', 'z', 'x']),
        ('d', vec!['e', 'r', 's', 'f', 'x', 'c']),
        ('f', vec!['r', 't', 'd', 'g', 'c', 'v']),
        ('g', vec!['t', 'y', 'f', 'h', 'v', 'b']),
        ('h', vec!['y', 'u', 'g', 'j', 'b', 'n']),
        ('j', vec!['u', 'i', 'h', 'k', 'n', 'm']),
        ('k', vec!['i', 'o', 'j', 'l', 'm']),
        ('l', vec!['o', 'p', 'k']),
        ('z', vec!['a', 's', 'x']),
        ('x', vec!['z', 's', 'd', 'c']),
        ('c', vec!['x', 'd', 'f', 'v']),
        ('v', vec!['c', 'f', 'g', 'b']),
        ('b', vec!['v', 'g', 'h', 'n']),
        ('n', vec!['b', 'h', 'j', 'm']),
        ('m', vec!['n', 'j', 'k']),
        ('1', vec!['2', '`']),
        ('2', vec!['1', '3', 'q']),
        ('3', vec!['2', '4', 'w']),
        ('4', vec!['3', '5', 'e']),
        ('5', vec!['4', '6', 'r']),
        ('6', vec!['5', '7', 't']),
        ('7', vec!['6', '8', 'y']),
        ('8', vec!['7', '9', 'u']),
        ('9', vec!['8', '0', 'i']),
        ('0', vec!['9', '-', 'o']),
        ('-', vec!['0', '=', 'p']),
        ('=', vec!['-']),
    ];

    for (key, adjacent) in adjacency_map {
        layout.insert(key, adjacent);
    }

    layout
}

// Implementations for numeric types

// Numeric proximity metric for integers
pub struct NumericProximityMetric {
    pub normalization_factor: f64,
}

impl Default for NumericProximityMetric {
    fn default() -> Self {
        NumericProximityMetric {
            normalization_factor: 10.0,
        }
    }
}

// Allow comparing with different numeric types
impl<T, U> SimilarityMetric<T, U> for NumericProximityMetric
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn calculate(&self, a: &T, b: &U) -> f64 {
        let a_val: f64 = (*a).into();
        let b_val: f64 = (*b).into();

        let difference = (a_val - b_val).abs();
        let normalized_diff = difference / self.normalization_factor;

        (-normalized_diff).exp()
    }

    fn name(&self) -> &str {
        "NumericProximity"
    }
}