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

pub struct CaseInsensitiveMetric;

impl SimilarityMetric<String, String> for CaseInsensitiveMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "CaseInsensitive"
    }
}

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

    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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

    fn name(&self) -> &str {
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