use crate::assessor::{Resembler, Resemblance, Assessment};

#[derive(PartialEq)]
pub struct Phonetic {
    mode: PhoneticMode,
}

#[derive(PartialEq)]
pub enum PhoneticMode {
    Soundex,
    DoubleMetaphone,
}

impl Default for Phonetic {
    fn default() -> Self {
        Self { mode: PhoneticMode::Soundex }
    }
}

impl Phonetic {
    pub fn new(mode: PhoneticMode) -> Self {
        Self { mode }
    }

    fn compute_soundex(&self, text: &str) -> String {
        if text.is_empty() { return "0000".to_string(); }

        let mut result = String::new();
        let mut prev_code = 0;
        for (i, c) in text.to_lowercase().chars().enumerate() {
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
            if result.len() >= 4 { break; }
        }

        while result.len() < 4 {
            result.push('0');
        }
        result
    }

    fn compute_double_metaphone(&self, text: &str) -> (String, String) {
        let primary = self.compute_soundex(text);
        let mut secondary_text = text.to_lowercase().replace("th", "d").replace("ph", "f").replace("ck", "k").replace("gn", "n").replace("wr", "r");
        if secondary_text == text.to_lowercase() {
            secondary_text = text.to_lowercase().replace("s", "z").replace("c", "k");
        }
        let secondary = self.compute_soundex(&secondary_text);
        (primary, secondary)
    }
}

impl Resembler<String, String, ()> for Phonetic {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        if query == candidate {
            return Assessment { resemblance: Resemblance::Perfect, errors: vec![] };
        }

        let resemblance = match self.mode {
            PhoneticMode::Soundex => {
                let query_code = self.compute_soundex(query);
                let candidate_code = self.compute_soundex(candidate);
                if query_code == candidate_code {
                    Resemblance::Partial(0.85)
                } else {
                    let common_prefix_len = query_code.chars().zip(candidate_code.chars())
                        .take_while(|(c1, c2)| c1 == c2)
                        .count();
                    if common_prefix_len > 0 {
                        Resemblance::Partial(0.6 * (common_prefix_len as f64 / 4.0))
                    } else {
                        Resemblance::Disparity
                    }
                }
            }
            PhoneticMode::DoubleMetaphone => {
                let (query_primary, query_secondary) = self.compute_double_metaphone(query);
                let (candidate_primary, candidate_secondary) = self.compute_double_metaphone(candidate);
                if query_primary == candidate_primary || query_primary == candidate_secondary ||
                    query_secondary == candidate_primary || query_secondary == candidate_secondary {
                    Resemblance::Partial(0.9)
                } else {
                    let common_prefix_len = query_primary.chars().zip(candidate_primary.chars())
                        .take_while(|(c1, c2)| c1 == c2)
                        .count();
                    if common_prefix_len > 0 {
                        Resemblance::Partial(0.7 * (common_prefix_len as f64 / 4.0))
                    } else {
                        Resemblance::Disparity
                    }
                }
            }
        };

        Assessment { resemblance, errors: vec![] }
    }
}