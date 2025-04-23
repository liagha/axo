use axo_hash::HashSet;
use {
    core::cmp::{max, min},
    axo_hash::HashMap,
    crate::{
        axo_matcher::{
            common::SimilarityMetric,
            utils::{
                damerau_levenshtein_distance, KeyboardLayoutType
            },
            MatchType,
        }
    }
};

/// Jaro-Winkler similarity for strings
pub struct JaroWinklerSimilarity {
    prefix_scale: f64,
}

impl Default for JaroWinklerSimilarity {
    fn default() -> Self {
        Self { prefix_scale: 0.1 } // Standard prefix scaling factor
    }
}

impl JaroWinklerSimilarity {
    pub fn new(prefix_scale: f64) -> Self {
        Self { prefix_scale }
    }

    fn jaro_distance(&self, s1: &str, s2: &str) -> f64 {
        let s1_len = s1.chars().count();
        let s2_len = s2.chars().count();

        if s1_len == 0 && s2_len == 0 {
            return 1.0;
        }

        if s1_len == 0 || s2_len == 0 {
            return 0.0;
        }

        // Maximum distance to consider characters as matching
        let match_distance = (s1_len.max(s2_len) / 2).max(1) - 1;

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        // Track matches
        let mut s1_matches = vec![false; s1_len];
        let mut s2_matches = vec![false; s2_len];

        // Count matching characters
        let mut matches = 0;
        for i in 0..s1_len {
            let start = i.saturating_sub(match_distance).max(0);
            let end = (i + match_distance + 1).min(s2_len);

            for j in start..end {
                if !s2_matches[j] && s1_chars[i] == s2_chars[j] {
                    s1_matches[i] = true;
                    s2_matches[j] = true;
                    matches += 1;
                    break;
                }
            }
        }

        if matches == 0 {
            return 0.0;
        }

        // Count transpositions
        let mut transpositions = 0;
        let mut k = 0;

        for i in 0..s1_len {
            if s1_matches[i] {
                while !s2_matches[k] {
                    k += 1;
                }

                if s1_chars[i] != s2_chars[k] {
                    transpositions += 1;
                }

                k += 1;
            }
        }

        // Calculate Jaro similarity
        let m = matches as f64;
        let t = transpositions as f64 / 2.0;

        if matches == 0 {
            0.0
        } else {
            (m / s1_len as f64 + m / s2_len as f64 + (m - t) / m) / 3.0
        }
    }

    fn get_common_prefix_length(&self, s1: &str, s2: &str) -> usize {
        let max_prefix_len = 4; // Standard prefix length for Jaro-Winkler

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        let min_len = s1_chars.len().min(s2_chars.len()).min(max_prefix_len);

        let mut prefix_len = 0;
        for i in 0..min_len {
            if s1_chars[i] == s2_chars[i] {
                prefix_len += 1;
            } else {
                break;
            }
        }

        prefix_len
    }
}

impl SimilarityMetric<String, String> for JaroWinklerSimilarity {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let jaro_dist = self.jaro_distance(query, candidate);

        // Apply Winkler modification (rewards strings with common prefixes)
        let prefix_len = self.get_common_prefix_length(query, candidate);

        jaro_dist + (prefix_len as f64 * self.prefix_scale * (1.0 - jaro_dist))
    }

    fn id(&self) -> &str {
        "jaro_winkler"
    }
}

/// Cosine similarity for strings using character n-grams
pub struct CosineSimilarity {
    n_gram_size: usize,
}

impl Default for CosineSimilarity {
    fn default() -> Self {
        Self { n_gram_size: 2 } // Default to bigrams
    }
}

impl CosineSimilarity {
    pub fn new(n_gram_size: usize) -> Self {
        Self { n_gram_size: n_gram_size.max(1) }
    }

    fn get_n_grams(&self, text: &str) -> HashMap<String, usize> {
        let mut n_grams = HashMap::new();

        if text.len() < self.n_gram_size {
            if !text.is_empty() {
                *n_grams.entry(text.to_string()).or_insert(0) += 1;
            }
            return n_grams;
        }

        let chars: Vec<char> = text.chars().collect();

        for i in 0..=(chars.len() - self.n_gram_size) {
            let n_gram: String = chars[i..(i + self.n_gram_size)].iter().collect();
            *n_grams.entry(n_gram).or_insert(0) += 1;
        }

        n_grams
    }
}

impl SimilarityMetric<String, String> for CosineSimilarity {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_n_grams = self.get_n_grams(query);
        let candidate_n_grams = self.get_n_grams(candidate);

        if query_n_grams.is_empty() || candidate_n_grams.is_empty() {
            return if query.is_empty() && candidate.is_empty() { 1.0 } else { 0.0 };
        }

        // Calculate dot product
        let mut dot_product = 0.0;
        for (n_gram, query_count) in &query_n_grams {
            if let Some(candidate_count) = candidate_n_grams.get(n_gram) {
                dot_product += (*query_count as f64) * (*candidate_count as f64);
            }
        }

        // Calculate magnitudes
        let query_magnitude: f64 = query_n_grams.values().map(|count| (*count as f64).powi(2)).sum::<f64>().sqrt();
        let candidate_magnitude: f64 = candidate_n_grams.values().map(|count| (*count as f64).powi(2)).sum::<f64>().sqrt();

        // Calculate cosine similarity
        if query_magnitude > 0.0 && candidate_magnitude > 0.0 {
            dot_product / (query_magnitude * candidate_magnitude)
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
        "cosine"
    }
}


pub struct ExactMatchMetric;

pub struct CaseInsensitiveMetric;

impl SimilarityMetric<String, String> for CaseInsensitiveMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn id(&self) -> &str {
        "CaseInsensitive"
    }
}

impl SimilarityMetric<&str, String> for CaseInsensitiveMetric {
    fn calculate(&self, query: &&str, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn id(&self) -> &str {
        "CaseInsensitive"
    }
}

impl SimilarityMetric<String, &str> for CaseInsensitiveMetric {
    fn calculate(&self, query: &String, candidate: &&str) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn id(&self) -> &str {
        "CaseInsensitive"
    }
}

pub struct PrefixMetric;

impl SimilarityMetric<String, String> for PrefixMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
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

impl SimilarityMetric<&str, String> for PrefixMetric {
    fn calculate(&self, query: &&str, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
        "Prefix"
    }
}

pub struct SuffixMetric;

impl SimilarityMetric<String, String> for SuffixMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.ends_with(&query_lower) {
            0.85 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
        "Suffix"
    }
}

pub struct SubstringMetric;

impl SimilarityMetric<String, String> for SubstringMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.contains(&query_lower) {
            0.8 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
        "Substring"
    }
}

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

    fn id(&self) -> &str {
        "EditDistance"
    }
}

pub struct TokenSimilarityMetric {
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

    fn id(&self) -> &str {
        "TokenSimilarity"
    }
}

impl TokenSimilarityMetric {
    pub fn new(separators: Vec<char>) -> Self {
        TokenSimilarityMetric { separators }
    }

    pub fn split_on_separators(&self, s: &str) -> Vec<String> {
        let mut tokens: Vec<String> = Vec::new();
        let mut current = String::new();

        for c in s.chars() {
            if self.separators.contains(&c) {
                if !current.is_empty() {
                    tokens.push(current);
                    current = String::new();
                }
            } else {
                if !current.is_empty() && current.chars().last().map_or(false, |last| !last.is_uppercase() && c.is_uppercase()) {
                    tokens.push(current);
                    current = String::new();
                }
                current.push(c);
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

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

        let token_sim = if !tokens1.is_empty() {
            total_sim / tokens1.len() as f64
        } else {
            0.0
        };

        let match_ratio = if !tokens1.is_empty() {
            matches as f64 / tokens1.len() as f64
        } else {
            0.0
        };

        token_sim * (1.0 + 0.5 * match_ratio)
    }
}

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

        let tokens = self.token_metric.split_on_separators(&candidate_lower);

        if tokens.len() < query_lower.len() {
            return 0.0;
        }

        let first_letters: String = tokens.iter()
            .filter_map(|token| token.chars().next())
            .collect();

        if first_letters.contains(&query_lower) {
            return 0.75;
        }

        0.0
    }

    fn id(&self) -> &str {
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

pub struct KeyboardProximityMetric {
    pub keyboard_layout: HashMap<char, Vec<char>>,
    pub layout_type: KeyboardLayoutType,
}

impl Default for KeyboardProximityMetric {
    fn default() -> Self {
        KeyboardProximityMetric {
            keyboard_layout: KeyboardLayoutType::Qwerty.get_layout(),
            layout_type: KeyboardLayoutType::Qwerty,
        }
    }
}

impl KeyboardProximityMetric {
    pub fn new(layout_type: KeyboardLayoutType) -> Self {
        KeyboardProximityMetric {
            keyboard_layout: layout_type.get_layout(),
            layout_type,
        }
    }
}

impl SimilarityMetric<String, String> for KeyboardProximityMetric {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        if (s1_lower.len() as isize - s2_lower.len() as isize).abs() > 2 {
            return 0.0;
        }

        let s1_chars: Vec<char> = s1_lower.chars().collect();
        let s2_chars: Vec<char> = s2_lower.chars().collect();

        let edit_distance = damerau_levenshtein_distance(&s1_lower, &s2_lower);

        if edit_distance > 3 {
            return 0.0;
        }

        let mut adjacency_count = 0;
        let max_comparisons = min(s1_chars.len(), s2_chars.len());

        for i in 0..max_comparisons {
            if s1_chars[i] == s2_chars[i] {
                continue;
            }

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

            let base_similarity = 1.0 - (edit_distance as f64 / max(s1_chars.len(), s2_chars.len()) as f64);
            base_similarity * (1.0 + 0.3 * keyboard_factor) * length_similarity
        }
    }

    fn id(&self) -> &str {
        match self.layout_type {
            KeyboardLayoutType::Qwerty => "QwertyProximity",
            KeyboardLayoutType::Dvorak => "DvorakProximity",
            KeyboardLayoutType::Custom(_) => "CustomKeyboardProximity",
        }
    }
}

pub struct FuzzySearchMetric {
    pub token_metric: TokenSimilarityMetric,
    pub min_token_similarity: f64,
}

impl Default for FuzzySearchMetric {
    fn default() -> Self {
        FuzzySearchMetric {
            token_metric: TokenSimilarityMetric::default(),
            min_token_similarity: 0.7,
        }
    }
}

impl SimilarityMetric<String, String> for FuzzySearchMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        let query_tokens = self.token_metric.split_on_separators(&query_lower);
        let candidate_tokens = self.token_metric.split_on_separators(&candidate_lower);

        if query_tokens.is_empty() || candidate_tokens.is_empty() {
            return 0.0;
        }

        let mut matched_tokens = 0;
        let mut total_similarity = 0.0;

        for q_token in &query_tokens {
            let mut best_match = 0.0;

            for c_token in &candidate_tokens {
                let edit_sim = 1.0 - (damerau_levenshtein_distance(q_token, c_token) as f64
                    / max(q_token.len(), c_token.len()) as f64);

                if edit_sim > best_match {
                    best_match = edit_sim;
                }

                if c_token.contains(q_token) {
                    let contain_score = q_token.len() as f64 / c_token.len() as f64 * 0.9;
                    best_match = best_match.max(contain_score);
                }
            }

            total_similarity += best_match;
            if best_match >= self.min_token_similarity {
                matched_tokens += 1;
            }
        }

        let coverage = matched_tokens as f64 / query_tokens.len() as f64;
        let avg_similarity = total_similarity / query_tokens.len() as f64;

        coverage * avg_similarity * (0.7 + 0.3 * coverage)
    }

    fn id(&self) -> &str {
        "FuzzySearch"
    }
}

pub struct PhoneticMetric {
    pub mode: PhoneticMode,
}

pub enum PhoneticMode {
    Soundex,
    DoubleMetaphone,
}

impl Default for PhoneticMetric {
    fn default() -> Self {
        PhoneticMetric {
            mode: PhoneticMode::Soundex,
        }
    }
}

impl SimilarityMetric<String, String> for PhoneticMetric {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        match self.mode {
            PhoneticMode::Soundex => {
                let s1_code = self.soundex(s1);
                let s2_code = self.soundex(s2);

                if s1_code == s2_code {
                    0.85
                } else {
                    let common_prefix_len = s1_code.chars().zip(s2_code.chars())
                        .take_while(|(c1, c2)| c1 == c2)
                        .count();

                    if common_prefix_len > 0 {
                        0.6 * (common_prefix_len as f64 / 4.0)
                    } else {
                        0.0
                    }
                }
            },
            PhoneticMode::DoubleMetaphone => {
                // Simplified double metaphone implementation
                if s1.to_lowercase() == s2.to_lowercase() {
                    return 1.0;
                }

                // Just use soundex as fallback
                let s1_code = self.soundex(s1);
                let s2_code = self.soundex(s2);

                if s1_code == s2_code {
                    0.8
                } else {
                    0.0
                }
            }
        }
    }
    fn id(&self) -> &str {
        match self.mode {
            PhoneticMode::Soundex => "Soundex",
            PhoneticMode::DoubleMetaphone => "DoubleMetaphone",
        }
    }
}

impl PhoneticMetric {
    pub fn new(mode: PhoneticMode) -> Self {
        PhoneticMetric { mode }
    }

    fn soundex(&self, s: &str) -> String {
        if s.is_empty() {
            return "0000".to_string();
        }

        let mut result = String::new();
        let mut prev_code = 0;

        for (i, c) in s.to_lowercase().chars().enumerate() {
            let code = match c {
                'b' | 'f' | 'p' | 'v' => 1,
                'c' | 'g' | 'j' | 'k' | 'q' | 's' | 'x' | 'z' => 2,
                'd' | 't' => 3,
                'l' => 4,
                'm' | 'n' => 5,
                'r' => 6,
                _ => 0,
            };

            if i == 0 {
                result.push(c.to_ascii_uppercase());
            } else if code != 0 && code != prev_code {
                result.push(char::from_digit(code, 10).unwrap());
            }

            prev_code = code;

            if result.len() >= 4 {
                break;
            }
        }

        while result.len() < 4 {
            result.push('0');
        }

        result
    }
}

pub struct NGramMetric {
    pub n: usize,
}

impl Default for NGramMetric {
    fn default() -> Self {
        NGramMetric { n: 2 }
    }
}

impl SimilarityMetric<String, String> for NGramMetric {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        if s1.is_empty() || s2.is_empty() {
            return if s1.is_empty() && s2.is_empty() { 1.0 } else { 0.0 };
        }

        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        let s1_ngrams = self.generate_ngrams(&s1_lower);
        let s2_ngrams = self.generate_ngrams(&s2_lower);

        if s1_ngrams.is_empty() || s2_ngrams.is_empty() {
            return 0.0;
        }

        let mut intersection = 0;

        for ngram in &s1_ngrams {
            if s2_ngrams.contains(ngram) {
                intersection += 1;
            }
        }

        (2.0 * intersection as f64) / (s1_ngrams.len() + s2_ngrams.len()) as f64
    }

    fn id(&self) -> &str {
        "NGram"
    }
}

impl NGramMetric {
    pub fn new(n: usize) -> Self {
        NGramMetric { n }
    }

    fn generate_ngrams(&self, s: &str) -> Vec<String> {
        if s.len() < self.n {
            return vec![s.to_string()];
        }

        let chars: Vec<char> = s.chars().collect();
        let mut ngrams = Vec::new();

        for i in 0..=chars.len() - self.n {
            let ngram: String = chars[i..i + self.n].iter().collect();
            ngrams.push(ngram);
        }

        ngrams
    }
}

/// Word overlap similarity using Jaccard similarity with customizable tokenization
pub struct WordOverlapSimilarity {
    /// Whether to ignore case when comparing words
    ignore_case: bool,
    /// Minimum length of words to consider
    min_word_length: usize,
    /// Custom tokenization characters (in addition to whitespace)
    custom_separators: Option<Vec<char>>,
    /// Whether to use stemming for word comparison
    use_stemming: bool,
    /// Stopwords to ignore in comparison
    stopwords: HashSet<String>,
}

impl Default for WordOverlapSimilarity {
    fn default() -> Self {
        Self {
            ignore_case: true,
            min_word_length: 1,
            custom_separators: None,
            use_stemming: false,
            stopwords: HashSet::new(),
        }
    }
}

impl WordOverlapSimilarity {
    /// Create a new WordOverlapSimilarity with custom settings
    pub fn new(
        ignore_case: bool,
        min_word_length: usize,
        custom_separators: Option<Vec<char>>,
        use_stemming: bool,
        stopwords: Option<Vec<&str>>,
    ) -> Self {
        Self {
            ignore_case,
            min_word_length,
            custom_separators,
            use_stemming,
            stopwords: stopwords
                .map(|words| words.into_iter().map(String::from).collect())
                .unwrap_or_default(),
        }
    }

    /// Create a simple WordOverlapSimilarity with just case sensitivity setting
    pub fn with_case_sensitivity(ignore_case: bool) -> Self {
        Self {
            ignore_case,
            ..Default::default()
        }
    }

    /// Tokenize text into words based on configuration
    fn get_words(&self, text: &str) -> Vec<String> {
        let normalized = if self.ignore_case {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        let mut result = Vec::new();

        // Split by whitespace and custom separators if provided
        let mut current_word = String::new();

        for c in normalized.chars() {
            let is_separator = c.is_whitespace() ||
                self.custom_separators.as_ref()
                    .map_or(false, |seps| seps.contains(&c));

            if is_separator {
                if !current_word.is_empty() {
                    self.add_processed_word(&current_word, &mut result);
                    current_word.clear();
                }
            } else {
                current_word.push(c);
            }
        }

        // Don't forget the last word
        if !current_word.is_empty() {
            self.add_processed_word(&current_word, &mut result);
        }

        result
    }

    /// Process and add a word to the result if it meets criteria
    fn add_processed_word(&self, word: &str, result: &mut Vec<String>) {
        // Skip words that are too short
        if word.len() < self.min_word_length {
            return;
        }

        // Skip stopwords
        if self.stopwords.contains(word) {
            return;
        }

        // Apply stemming if enabled
        let processed = if self.use_stemming {
            self.apply_stemming(word)
        } else {
            word.to_string()
        };

        result.push(processed);
    }

    /// Apply basic stemming (very simplified Porter stemming)
    fn apply_stemming(&self, word: &str) -> String {
        // This is a very simplified version of stemming
        // In a real implementation, you would use a proper stemming algorithm

        let mut result = word.to_string();

        // Remove common English suffixes
        for suffix in &["ing", "ed", "s", "es", "ies"] {
            if result.ends_with(suffix) && result.len() > suffix.len() + 2 {
                result.truncate(result.len() - suffix.len());
                break;
            }
        }

        result
    }

    /// Calculate weighted Jaccard similarity with position awareness
    fn weighted_jaccard(&self, query_words: &[String], candidate_words: &[String]) -> f64 {
        if query_words.is_empty() && candidate_words.is_empty() {
            return 1.0;
        }

        if query_words.is_empty() || candidate_words.is_empty() {
            return 0.0;
        }

        // Count common words with position-based weighting
        let mut common_weight = 0.0;

        for (i, q_word) in query_words.iter().enumerate() {
            for (j, c_word) in candidate_words.iter().enumerate() {
                if q_word == c_word {
                    // Words that appear in similar positions get higher weight
                    let position_factor = 1.0 - (i as f64 - j as f64).abs() /
                        (query_words.len().max(candidate_words.len()) as f64);

                    common_weight += 1.0 * (0.5 + 0.5 * position_factor);
                    break;
                }
            }
        }

        // Modified Jaccard similarity: weighted_intersection / union
        let union_size = query_words.len() + candidate_words.len() - common_weight as usize;
        common_weight / union_size as f64
    }
}

impl SimilarityMetric<String, String> for WordOverlapSimilarity {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_words = self.get_words(query);
        let candidate_words = self.get_words(candidate);

        if query_words.is_empty() && candidate_words.is_empty() {
            return 1.0;
        }

        if query_words.is_empty() || candidate_words.is_empty() {
            return 0.0;
        }

        // Use standard Jaccard similarity for simple cases
        if query_words.len() <= 2 || candidate_words.len() <= 2 {
            // Count common words
            let mut common_words = 0;
            for q_word in &query_words {
                if candidate_words.contains(q_word) {
                    common_words += 1;
                }
            }

            // Jaccard similarity: |A ∩ B| / |A ∪ B|
            let union_size = query_words.len() + candidate_words.len() - common_words;
            common_words as f64 / union_size as f64
        } else {
            // Use position-aware weighted Jaccard for longer texts
            self.weighted_jaccard(&query_words, &candidate_words)
        }
    }

    fn id(&self) -> &str {
        "word_overlap"
    }
}

// Support for string references
impl SimilarityMetric<&str, String> for WordOverlapSimilarity {
    fn calculate(&self, query: &&str, candidate: &String) -> f64 {
        let query_str = query.to_string();
        self.calculate(&query_str, candidate)
    }

    fn id(&self) -> &str {
        "word_overlap"
    }
}

impl SimilarityMetric<String, &str> for WordOverlapSimilarity {
    fn calculate(&self, query: &String, candidate: &&str) -> f64 {
        let candidate_str = candidate.to_string();
        self.calculate(query, &candidate_str)
    }

    fn id(&self) -> &str {
        "word_overlap"
    }
}

/// Levenshtein distance-based similarity for strings
pub struct LevenshteinSimilarity;

impl LevenshteinSimilarity {
    pub fn new() -> Self {
        Self {}
    }

    /// Calculates the Levenshtein edit distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        if s1.is_empty() {
            return s2.len();
        }
        if s2.is_empty() {
            return s1.len();
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let s1_len = s1_chars.len();
        let s2_len = s2_chars.len();

        // Initialize the matrix
        let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

        // Fill the first row and column
        for i in 0..=s1_len {
            matrix[i][0] = i;
        }
        for j in 0..=s2_len {
            matrix[0][j] = j;
        }

        // Fill the rest of the matrix
        for i in 1..=s1_len {
            for j in 1..=s2_len {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };

                matrix[i][j] = *[
                    matrix[i - 1][j] + 1,           // deletion
                    matrix[i][j - 1] + 1,           // insertion
                    matrix[i - 1][j - 1] + cost,    // substitution
                ].iter().min().unwrap();
            }
        }

        matrix[s1_len][s2_len]
    }
}

impl Default for LevenshteinSimilarity {
    fn default() -> Self {
        Self {}
    }
}

impl SimilarityMetric<String, String> for LevenshteinSimilarity {
    fn calculate(&self, s1: &String, s2: &String) -> f64 {
        let distance = self.levenshtein_distance(s1, s2);
        let max_len = core::cmp::max(s1.len(), s2.len());

        if max_len == 0 {
            return 1.0; // Both strings are empty, they're identical
        }

        // Convert distance to similarity score (0.0 to 1.0)
        // where 1.0 means identical and 0.0 means completely different
        1.0 - (distance as f64 / max_len as f64)
    }

    fn id(&self) -> &str {
        "levenshtein"
    }

    fn match_type(&self, query: &String, candidate: &String) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.7 {
            Some(MatchType::Similar("Levenshtein".to_string()))
        } else {
            None
        }
    }
}

// Support for &str references
impl SimilarityMetric<&str, String> for LevenshteinSimilarity {
    fn calculate(&self, query: &&str, candidate: &String) -> f64 {
        let distance = self.levenshtein_distance(query, candidate);
        let max_len = core::cmp::max(query.len(), candidate.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    fn id(&self) -> &str {
        "levenshtein"
    }
}

impl SimilarityMetric<String, &str> for LevenshteinSimilarity {
    fn calculate(&self, query: &String, candidate: &&str) -> f64 {
        let distance = self.levenshtein_distance(query, candidate);
        let max_len = core::cmp::max(query.len(), candidate.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    fn id(&self) -> &str {
        "levenshtein"
    }
}

impl SimilarityMetric<String, String> for ExactMatchMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        if query == candidate {
            1.0
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
        "ExactMatch"
    }
}