#![allow(dead_code)]

use core::cmp::{max, min};
use hashbrown::HashMap;

// Structure to hold similarity metrics and matching information
#[derive(Debug)]
pub struct MatchInfo {
    pub score: f64,            // Overall similarity score (0.0 to 1.0)
    pub name: String,          // The matched name
    pub match_type: MatchType, // Type of match found
}

#[derive(Debug, PartialEq)]
pub enum MatchType {
    Exact,              // Perfect match
    CaseInsensitive,    // Match ignoring case
    Prefix,             // Prefix match
    Suffix,             // Suffix match
    Substring,          // Substring match
    Acronym,            // Acronym match (e.g., "http_server" matches "hs")
    Similar,            // Similar based on combined metrics
    NotFound,           // No match found above threshold
}

pub struct Matcher {
    // Weights for different similarity components
    pub prefix_weight: f64,
    pub suffix_weight: f64,
    pub common_weight: f64,
    pub edit_dist_weight: f64,
    pub keyboard_dist_weight: f64,

    // Threshold below which matches are considered "not found"
    pub threshold: f64,

    // Optional keyboard layout for typo detection
    keyboard_layout: Option<HashMap<char, Vec<char>>>,
}

impl Default for Matcher {
    fn default() -> Self {
        Matcher {
            prefix_weight: 0.3,
            suffix_weight: 0.2,
            common_weight: 0.2,
            edit_dist_weight: 0.2,
            keyboard_dist_weight: 0.1,
            threshold: 0.4,
            keyboard_layout: Some(create_qwerty_layout()),
        }
    }
}

impl Matcher {
    pub fn new(
        prefix_weight: f64,
        suffix_weight: f64,
        common_subseq_weight: f64,
        edit_dist_weight: f64,
        keyboard_dist_weight: f64,
        threshold: f64,
    ) -> Self {
        Matcher {
            prefix_weight,
            suffix_weight,
            common_weight: common_subseq_weight,
            edit_dist_weight,
            keyboard_dist_weight,
            threshold,
            keyboard_layout: Some(create_qwerty_layout()),
        }
    }

    // Find the best match for a query string from a list of candidates
    pub fn find_best_match<'a>(&self, query: &str, candidates: &'a [String]) -> Option<MatchInfo> {
        if candidates.is_empty() {
            return None;
        }

        // Early exact matches
        for candidate in candidates {
            if query == candidate {
                return Some(MatchInfo {
                    score: 1.0,
                    name: candidate.clone(),
                    match_type: MatchType::Exact,
                });
            }
        }

        // Case insensitive match
        let query_lower = query.to_lowercase();
        for candidate in candidates {
            if query_lower == candidate.to_lowercase() {
                return Some(MatchInfo {
                    score: 0.95, // Not exactly 1.0 since it's not an exact match
                    name: candidate.clone(),
                    match_type: MatchType::CaseInsensitive,
                });
            }
        }

        // Check for prefix, suffix, substring matches
        for candidate in candidates {
            let candidate_lower = candidate.to_lowercase();

            // Prefix match
            if candidate_lower.starts_with(&query_lower) {
                let score = 0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0);
                if score > self.threshold {
                    return Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Prefix,
                    });
                }
            }

            // Suffix match
            if candidate_lower.ends_with(&query_lower) {
                let score = 0.85 * (query.len() as f64 / candidate.len() as f64).min(1.0);
                if score > self.threshold {
                    return Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Suffix,
                    });
                }
            }

            // Substring match
            if candidate_lower.contains(&query_lower) {
                let score = 0.8 * (query.len() as f64 / candidate.len() as f64).min(1.0);
                if score > self.threshold {
                    return Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Substring,
                    });
                }
            }
        }

        // Check for acronym matches
        if query.len() <= 5 {  // Only check acronyms for short queries
            for candidate in candidates {
                if self.is_acronym_match(query, candidate) {
                    return Some(MatchInfo {
                        score: 0.75,
                        name: candidate.clone(),
                        match_type: MatchType::Acronym,
                    });
                }
            }
        }

        // Detailed similarity calculation for all candidates
        let mut best_match: Option<MatchInfo> = None;
        let mut best_score = self.threshold;

        for candidate in candidates {
            let score = self.calculate_similarity(query, candidate);

            if score > best_score {
                best_score = score;
                best_match = Some(MatchInfo {
                    score,
                    name: candidate.clone(),
                    match_type: MatchType::Similar,
                });
            }
        }

        best_match
    }

    // Find all matches above a certain threshold, sorted by score
    pub fn find_all_matches(&self, query: &str, candidates: &[String], limit: usize) -> Vec<MatchInfo> {
        let mut matches: Vec<MatchInfo> = Vec::new();

        for candidate in candidates {
            if query == candidate {
                matches.push(MatchInfo {
                    score: 1.0,
                    name: candidate.clone(),
                    match_type: MatchType::Exact,
                });
                continue;
            }

            let query_lower = query.to_lowercase();
            let candidate_lower = candidate.to_lowercase();

            if query_lower == candidate_lower {
                matches.push(MatchInfo {
                    score: 0.95,
                    name: candidate.clone(),
                    match_type: MatchType::CaseInsensitive,
                });
                continue;
            }

            // Check for prefix, suffix, substring matches
            let mut match_info = None;

            if candidate_lower.starts_with(&query_lower) {
                let score = 0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0);
                if score > self.threshold {
                    match_info = Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Prefix,
                    });
                }
            } else if candidate_lower.ends_with(&query_lower) {
                let score = 0.85 * (query.len() as f64 / candidate.len() as f64).min(1.0);
                if score > self.threshold {
                    match_info = Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Suffix,
                    });
                }
            } else if candidate_lower.contains(&query_lower) {
                let score = 0.8 * (query.len() as f64 / candidate.len() as f64).min(1.0);
                if score > self.threshold {
                    match_info = Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Substring,
                    });
                }
            } else if query.len() <= 5 && self.is_acronym_match(query, candidate) {
                match_info = Some(MatchInfo {
                    score: 0.75,
                    name: candidate.clone(),
                    match_type: MatchType::Acronym,
                });
            } else {
                // Detailed similarity calculation
                let score = self.calculate_similarity(query, candidate);
                if score > self.threshold {
                    match_info = Some(MatchInfo {
                        score,
                        name: candidate.clone(),
                        match_type: MatchType::Similar,
                    });
                }
            }

            if let Some(info) = match_info {
                matches.push(info);
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

    // Calculate the combined similarity score between two strings
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Split strings on common code separators for better token comparison
        let s1_tokens: Vec<&str> = self.split_on_separators(&s1_lower);
        let s2_tokens: Vec<&str> = self.split_on_separators(&s2_lower);

        // Calculate common prefix length (normalized)
        let prefix_score = self.calculate_prefix_similarity(&s1_lower, &s2_lower);

        // Calculate common suffix length (normalized)
        let suffix_score = self.calculate_suffix_similarity(&s1_lower, &s2_lower);

        // Calculate longest common subsequence (normalized)
        let lcs_score = self.longest_common_subsequence(&s1_lower, &s2_lower) as f64 /
            max(s1_lower.len(), s2_lower.len()) as f64;

        // Calculate normalized edit distance
        let edit_dist_score = 1.0 - self.damerau_levenshtein_distance(&s1_lower, &s2_lower) as f64 /
            max(s1_lower.len(), s2_lower.len()) as f64;

        // Calculate token similarity score
        let token_score = self.token_similarity(&s1_tokens, &s2_tokens);

        // Calculate keyboard proximity for potential typos
        let keyboard_score = if let Some(layout) = &self.keyboard_layout {
            self.keyboard_proximity(&s1_lower, &s2_lower, layout)
        } else {
            0.0
        };

        // Combine scores with weights
        let combined_score =
            self.prefix_weight * prefix_score +
                self.suffix_weight * suffix_score +
                self.common_weight * lcs_score +
                self.edit_dist_weight * edit_dist_score +
                self.keyboard_dist_weight * keyboard_score;

        // Boost score for token matches
        let final_score = combined_score * (1.0 + 0.3 * token_score);

        // Normalize to 0.0-1.0 range
        final_score.min(1.0)
    }

    // Check if a query string is an acronym of a candidate
    // For example, "hs" could match "http_server"
    fn is_acronym_match(&self, query: &str, candidate: &str) -> bool {
        let query = query.to_lowercase();
        let candidate = candidate.to_lowercase();

        // Split candidate on common separators and check first letters
        let tokens = self.split_on_separators(&candidate);

        if tokens.len() < query.len() {
            return false;
        }

        // Get first letters of each token
        let first_letters: String = tokens.iter()
            .filter_map(|token| token.chars().next())
            .collect();

        first_letters.contains(&query)
    }

    // Split a string on common separators used in code
    fn split_on_separators<'a>(&self, s: &'a str) -> Vec<&'a str> {
        // First split on common separators
        let tokens: Vec<&str> = s.split(|c| c == '_' || c == '-' || c == '.' || c == ' ').collect();

        // Then handle camelCase and PascalCase
        let mut final_tokens = Vec::new();

        for token in tokens.clone() {
            if token.is_empty() {
                continue;
            }

            // Look for transitions from lowercase to uppercase
            let mut last_idx = 0;
            let chars: Vec<char> = token.chars().collect();

            for i in 1..chars.len() {
                if !chars[i-1].is_uppercase() && chars[i].is_uppercase() {
                    if last_idx < i {
                        final_tokens.push(&token[last_idx..i]);
                    }
                    last_idx = i;
                }
            }

            // Add the last token
            if last_idx < token.len() {
                final_tokens.push(&token[last_idx..]);
            }
        }

        if final_tokens.is_empty() {
            tokens
        } else {
            final_tokens
        }
    }

    // Calculate the similarity between two sets of tokens
    fn token_similarity(&self, tokens1: &[&str], tokens2: &[&str]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        let mut total_sim = 0.0;

        // Count exact token matches and calculate similarity for others
        for t1 in tokens1 {
            let mut best_sim: f64 = 0.0;

            for t2 in tokens2 {
                if t1 == t2 {
                    best_sim = 1.0;
                    break;
                }

                let token_sim = 1.0 - self.damerau_levenshtein_distance(t1, t2) as f64 /
                    max(t1.len(), t2.len()) as f64;
                best_sim = best_sim.max(token_sim);
            }

            total_sim += best_sim;
            if best_sim > 0.8 {
                matches += 1;
            }
        }

        // Normalize by the number of tokens
        let token_sim = total_sim / tokens1.len() as f64;

        // Boost for high number of token matches
        let match_ratio = matches as f64 / tokens1.len() as f64;

        token_sim * (1.0 + 0.5 * match_ratio)
    }

    // Calculate the prefix similarity between two strings
    fn calculate_prefix_similarity(&self, s1: &str, s2: &str) -> f64 {
        let min_len = min(s1.len(), s2.len());

        let mut common_len = 0;
        for (c1, c2) in s1.chars().zip(s2.chars()) {
            if c1 == c2 {
                common_len += 1;
            } else {
                break;
            }
        }

        if min_len == 0 {
            0.0
        } else {
            common_len as f64 / min_len as f64
        }
    }

    // Calculate the suffix similarity between two strings
    fn calculate_suffix_similarity(&self, s1: &str, s2: &str) -> f64 {
        let min_len = min(s1.len(), s2.len());

        let mut common_len = 0;
        for (c1, c2) in s1.chars().rev().zip(s2.chars().rev()) {
            if c1 == c2 {
                common_len += 1;
            } else {
                break;
            }
        }

        if min_len == 0 {
            0.0
        } else {
            common_len as f64 / min_len as f64
        }
    }

    // Calculate the longest common subsequence between two strings
    fn longest_common_subsequence(&self, s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        let m = s1_chars.len();
        let n = s2_chars.len();

        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if s1_chars[i - 1] == s2_chars[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = max(dp[i - 1][j], dp[i][j - 1]);
                }
            }
        }

        dp[m][n]
    }

    // Calculate the Damerau-Levenshtein distance between two strings
    // This is like Levenshtein but also accounts for transpositions (swapped characters)
    fn damerau_levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
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

    // Calculate keyboard proximity for potential typos
    fn keyboard_proximity(&self, s1: &str, s2: &str, layout: &HashMap<char, Vec<char>>) -> f64 {
        if s1.len() != s2.len() {
            return 0.0;  // Only consider strings of equal length for proximity
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        let mut adjacency_count = 0;

        for i in 0..s1_chars.len() {
            if s1_chars[i] == s2_chars[i] {
                continue;
            }

            // Check if characters are adjacent on keyboard
            if let Some(neighbors) = layout.get(&s1_chars[i]) {
                if neighbors.contains(&s2_chars[i]) {
                    adjacency_count += 1;
                }
            }
        }

        let diff_chars = s1_chars.iter().zip(s2_chars.iter()).filter(|(c1, c2)| c1 != c2).count();

        if diff_chars == 0 {
            1.0
        } else {
            adjacency_count as f64 / diff_chars as f64
        }
    }
}

// Create a QWERTY keyboard layout for English
fn create_qwerty_layout() -> HashMap<char, Vec<char>> {
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


// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let matcher = Matcher::default();
        let candidates = vec!["test_variable".to_string(), "other_var".to_string()];

        let result = matcher.find_best_match("test_variable", &candidates).unwrap();
        assert_eq!(result.match_type, MatchType::Exact);
        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn test_case_insensitive_match() {
        let matcher = Matcher::default();
        let candidates = vec!["TestVariable".to_string(), "other_var".to_string()];

        let result = matcher.find_best_match("testvariable", &candidates).unwrap();
        assert_eq!(result.match_type, MatchType::CaseInsensitive);
        assert!(result.score > 0.9);
    }

    #[test]
    fn test_split_on_separators() {
        let matcher = Matcher::default();

        let result = matcher.split_on_separators("user_name");
        assert_eq!(result, vec!["user", "name"]);

        let result = matcher.split_on_separators("userName");
        assert_eq!(result, vec!["user", "Name"]);

        let result = matcher.split_on_separators("HTTPServer");
        assert_eq!(result, vec!["HTTP", "Server"]);
    }

    #[test]
    fn test_acronym_match() {
        let matcher = Matcher::default();
        let candidates = vec!["http_server".to_string(), "html_parser".to_string()];

        let result = matcher.find_best_match("hs", &candidates).unwrap();
        assert_eq!(result.match_type, MatchType::Acronym);
        assert_eq!(result.name, "http_server");
    }

    #[test]
    fn test_typo_detection() {
        let matcher = Matcher::default();
        let candidates = vec!["username".to_string(), "user_id".to_string()];

        // 'n' is adjacent to 'm' on QWERTY keyboard
        let result = matcher.find_best_match("usermane", &candidates).unwrap();
        assert_eq!(result.name, "username");
        assert!(result.score > 0.8);
    }

    #[test]
    fn test_transposition() {
        let matcher = Matcher::default();
        let candidates = vec!["algorithm".to_string(), "logarithm".to_string()];

        // Swap 'a' and 'l' characters
        let result = matcher.find_best_match("algroithm", &candidates).unwrap();
        assert_eq!(result.name, "algorithm");
    }

    #[test]
    fn test_token_similarity() {
        let matcher = Matcher::default();

        let tokens1 = vec!["user", "account", "id"];
        let tokens2 = vec!["user", "profile", "id"];

        let similarity = matcher.token_similarity(&tokens1, &tokens2);
        assert!(similarity > 0.6);
    }

    #[test]
    fn test_empty_candidates() {
        let matcher = Matcher::default();
        let candidates: Vec<String> = vec![];

        let result = matcher.find_best_match("test", &candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_sort_by_score() {
        let matcher = Matcher::default();
        let candidates = vec![
            "user_name".to_string(),
            "user_id".to_string(),
            "username".to_string(),
        ];

        let results = matcher.find_all_matches("username", &candidates, 0);

        // Verify results are sorted by score in descending order
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score);
        }
    }
}