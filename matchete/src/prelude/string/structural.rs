use crate::{
    assessor::{Resembler, Resemblance, Assessment},
};

#[derive(PartialEq)]
pub struct Prefix;

impl Resembler<String, String, ()> for Prefix {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        let resemblance = if candidate.to_lowercase().starts_with(&query.to_lowercase()) {
            let score = 0.9 * f64::min(query.len() as f64 / candidate.len() as f64, 1.0);
            Resemblance::Partial(score)
        } else {
            Resemblance::Disparity
        };
        Assessment { resemblance, errors: vec![] }
    }
}

#[derive(PartialEq)]
pub struct Suffix;

impl Resembler<String, String, ()> for Suffix {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        let resemblance = if candidate.to_lowercase().ends_with(&query.to_lowercase()) {
            let score = 0.85 * f64::min(query.len() as f64 / candidate.len() as f64, 1.0);
            Resemblance::Partial(score)
        } else {
            Resemblance::Disparity
        };
        Assessment { resemblance, errors: vec![] }
    }
}

#[derive(PartialEq)]
pub struct Contains;

impl Resembler<String, String, ()> for Contains {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        let resemblance = if candidate.to_lowercase().contains(&query.to_lowercase()) {
            let score = 0.8 * f64::min(query.len() as f64 / candidate.len() as f64, 1.0);
            Resemblance::Partial(score)
        } else {
            Resemblance::Disparity
        };
        Assessment { resemblance, errors: vec![] }
    }
}

#[derive(PartialEq)]
pub struct Sequential {
    size: usize,
}

impl Default for Sequential {
    fn default() -> Self {
        Self { size: 2 }
    }
}

impl Sequential {
    pub fn new(size: usize) -> Self {
        Self { size }
    }

    fn generate_ngrams(&self, text: &str) -> Vec<String> {
        if text.len() < self.size { return vec![text.to_string()]; }

        let chars: Vec<char> = text.chars().collect();
        (0..=chars.len() - self.size)
            .map(|i| chars[i..i + self.size].iter().collect())
            .collect()
    }
}

impl Resembler<String, String, ()> for Sequential {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        if query.is_empty() && candidate.is_empty() {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }
        if query.is_empty() || candidate.is_empty() {
            return Assessment { resemblance: Resemblance::Disparity, errors: vec![] };
        }

        let query_ngrams = self.generate_ngrams(&query.to_lowercase());
        let candidate_ngrams = self.generate_ngrams(&candidate.to_lowercase());

        if query_ngrams.is_empty() || candidate_ngrams.is_empty() {
            return Assessment { resemblance: Resemblance::Disparity, errors: vec![] };
        }

        let intersection = query_ngrams.iter().filter(|ngram| candidate_ngrams.contains(ngram)).count();
        let score = 2.0 * intersection as f64 / (query_ngrams.len() + candidate_ngrams.len()) as f64;

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