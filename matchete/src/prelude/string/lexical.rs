use {
    crate::{
        assessor::{Resembler, Resemblance, Assessment},
    },
    core::cmp::max,
    hashish::{HashSet},
};

#[derive(PartialEq)]
pub struct Words {
    ignore_case: bool,
    min_word_len: usize,
    separators: Option<Vec<char>>,
    use_stemming: bool,
    stop_words: HashSet<String>,
}

impl Default for Words {
    fn default() -> Self {
        Self {
            ignore_case: true,
            min_word_len: 1,
            separators: None,
            use_stemming: false,
            stop_words: HashSet::new(),
        }
    }
}

impl Words {
    pub fn new(
        ignore_case: bool,
        min_word_len: usize,
        separators: Option<Vec<char>>,
        use_stemming: bool,
        stop_words: Option<Vec<&str>>,
    ) -> Self {
        Self {
            ignore_case,
            min_word_len,
            separators,
            use_stemming,
            stop_words: stop_words.map(|words| words.into_iter().map(String::from).collect()).unwrap_or_default(),
        }
    }

    pub fn with_case_sensitivity(ignore_case: bool) -> Self {
        Self { ignore_case, ..Default::default() }
    }

    fn extract_words(&self, text: &str) -> Vec<String> {
        let normalized = if self.ignore_case { text.to_lowercase() } else { text.to_string() };
        let mut words = Vec::new();
        let mut current = String::new();

        for c in normalized.chars() {
            let is_separator = c.is_whitespace() || self.separators.as_ref().map_or(false, |seps| seps.contains(&c));
            if is_separator {
                if !current.is_empty() {
                    self.process_word(&current, &mut words);
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() { self.process_word(&current, &mut words); }
        words
    }

    fn process_word(&self, word: &str, words: &mut Vec<String>) {
        if word.len() < self.min_word_len || self.stop_words.contains(word) { return; }
        let processed = if self.use_stemming { self.stem_word(word) } else { word.to_string() };
        words.push(processed);
    }

    fn stem_word(&self, word: &str) -> String {
        let mut result = word.to_string();
        for suffix in &["ing", "ed", "s", "es", "ies"] {
            if result.ends_with(suffix) && result.len() > suffix.len() + 2 {
                result.truncate(result.len() - suffix.len());
                break;
            }
        }
        result
    }

    fn weighted_jaccard(&self, query_words: &[String], candidate_words: &[String]) -> f64 {
        if query_words.is_empty() && candidate_words.is_empty() { return 1.0; }
        if query_words.is_empty() || candidate_words.is_empty() { return 0.0; }

        let mut common_weight = 0.0;
        for (i, q_word) in query_words.iter().enumerate() {
            for (j, c_word) in candidate_words.iter().enumerate() {
                if q_word == c_word {
                    let position_factor = 1.0 - (i as f64 - j as f64).abs() / max(query_words.len(), candidate_words.len()) as f64;
                    common_weight += 0.4 + 0.6 * position_factor;
                    break;
                }
            }
        }

        let union_size = query_words.len() + candidate_words.len() - common_weight as usize;
        common_weight / union_size as f64
    }
}

impl Resembler<String, String, ()> for Words {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        let query_words = self.extract_words(query);
        let candidate_words = self.extract_words(candidate);

        if query_words.is_empty() && candidate_words.is_empty() {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }
        if query_words.is_empty() || candidate_words.is_empty() {
            return Assessment { resemblance: Resemblance::Disparity, errors: vec![] };
        }

        let score = if query_words.len() <= 2 || candidate_words.len() <= 2 {
            let common_words = query_words.iter().filter(|w| candidate_words.contains(w)).count();
            let union_size = query_words.len() + candidate_words.len() - common_words;
            common_words as f64 / union_size as f64
        } else {
            self.weighted_jaccard(&query_words, &candidate_words)
        };

        let resemblance = if score >= 1.0 {
            Resemblance::Perfect
        } else if score > 0.0 {
            Resemblance::Partial(score)
        } else {
            Resemblance::Disparity
        };

        Assessment { resemblance, errors: vec![] }
    }
}