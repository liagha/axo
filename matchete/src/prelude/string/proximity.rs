use {
    hashish::HashMap,
    crate::{
        assessor::{Resembler, Resemblance, Assessment},
        prelude::string::utils::{edit_distance, keyboard::Layout},
    }
};
use core::cmp::{max, min};

#[derive(PartialEq)]
pub struct Keyboard {
    layout: HashMap<char, Vec<char>>,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            layout: Layout::Qwerty.get_layout(),
        }
    }
}

impl Keyboard {
    pub fn new(layout_type: Layout) -> Self {
        Self {
            layout: layout_type.get_layout(),
        }
    }
}

impl Resembler<String, String, ()> for Keyboard {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        let query_chars: Vec<char> = query.to_lowercase().chars().collect();
        let candidate_chars: Vec<char> = candidate.to_lowercase().chars().collect();

        if (query_chars.len() as isize - candidate_chars.len() as isize).abs() > 2 {
            return Assessment { resemblance: Resemblance::Disparity, errors: vec![] };
        }

        let distance = edit_distance(query, candidate);
        if distance > 3 {
            return Assessment { resemblance: Resemblance::Disparity, errors: vec![] };
        }

        let mut adjacent_count = 0;
        let max_comparisons = min(query_chars.len(), candidate_chars.len());
        for i in 0..max_comparisons {
            if query_chars[i] == candidate_chars[i] { continue; }
            if let Some(neighbors) = self.layout.get(&query_chars[i]) {
                if neighbors.contains(&candidate_chars[i]) { adjacent_count += 1; }
            }
        }

        let differing_chars = distance;
        if differing_chars == 0 { return Assessment { resemblance: Resemblance::Perfect, errors: vec![] }; }

        let keyboard_factor = adjacent_count as f64 / differing_chars as f64;
        let length_similarity = 1.0 - ((query_chars.len() as isize - candidate_chars.len() as isize).abs() as f64 / max(query_chars.len(), candidate_chars.len()) as f64);
        let base_score = 1.0 - (distance as f64 / max(query_chars.len(), candidate_chars.len()) as f64);
        let score = base_score * (1.0 + 0.5 * keyboard_factor) * length_similarity;

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