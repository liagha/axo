use {
    crate::compare::{
        max, min
    },
    matchete::{
        damerau_levenshtein_distance,
        AcronymMetric, CaseInsensitiveMetric, EditDistanceMetric,
        ExactMatchMetric, KeyboardProximityMetric, MatchType, Matcher,
        MatcherBuilder, PrefixMetric, SimilarityMetric, SubstringMetric,
        SuffixMetric, TokenSimilarityMetric
    },
    crate::{
        axo_lexer::{
            Token, TokenKind
        },
        axo_parser::{
            Element, ElementKind,
            Item, ItemKind,
        },
    }
};

impl SimilarityMetric<Token, Token> for CaseInsensitiveMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        if query.to_string().to_lowercase() == candidate.to_string().to_lowercase() { 0.95 } else { 0.0 }
    }

}

impl SimilarityMetric<Token, Token> for PrefixMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            return 0.9 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0);
        } else if query_lower.starts_with(&candidate_lower) {
            return 0.9 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
        }

        0.0
    }


    fn match_type(&self, query: &Token, candidate: &Token) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Prefix".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Token, Token> for SubstringMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        if candidate_lower.contains(&query_lower) {
            return 0.8 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0);
        } else if query_lower.contains(&candidate_lower) {
            return 0.8 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
        }

        0.0
    }

}

impl SimilarityMetric<Token, Token> for EditDistanceMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        let distance = damerau_levenshtein_distance(&*s1.to_string(), &*s2.to_string());
        let max_len = max(s1.to_string().len(), s2.to_string().len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

}

impl SimilarityMetric<Token, Token> for TokenSimilarityMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        let s1_lower = s1.to_string().to_lowercase();
        let s2_lower = s2.to_string().to_lowercase();

        let s1_tokens = self.split_on_separators(&s1_lower);
        let s2_tokens = self.split_on_separators(&s2_lower);

        self.token_similarity(&s1_tokens, &s2_tokens)
    }

}

impl SimilarityMetric<Token, Token> for AcronymMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        if query.to_string().len() > self.max_acronym_length {
            return 0.0;
        }

        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

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


    fn match_type(&self, query: &Token, candidate: &Token) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Acronym".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Token, Token> for KeyboardProximityMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        let s1_lower = s1.to_string().to_lowercase();
        let s2_lower = s2.to_string().to_lowercase();

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

}

#[derive(Debug)]
pub struct TokenKindMetric;

impl SimilarityMetric<Token, Token> for TokenKindMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        if s1.kind == s2.kind {
            0.2
        } else {
            0.0
        }
    }

}

impl SimilarityMetric<Token, Token> for SuffixMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        if candidate_lower.ends_with(&query_lower) {
            return 0.85 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0);
        } else if query_lower.ends_with(&candidate_lower) {
            return 0.85 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
        }

        0.0
    }


    fn match_type(&self, query: &Token, candidate: &Token) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Suffix".to_string()))
        } else {
            None
        }
    }
}

pub trait Labeled<L> {
    fn name(&self) -> Option<L>;
}

impl Labeled<Token> for Element {
    fn name(&self) -> Option<Token> {
        let Element { kind, span } = self.clone();

        match kind {
            ElementKind::Literal(literal) => Some(Token { kind: literal, span }),
            ElementKind::Identifier(identifier) => Some(Token {
                kind: TokenKind::Identifier(identifier),
                span,
            }),
            ElementKind::Constructor { name, .. } => name.name(),
            ElementKind::Labeled { label, .. } => label.name(),
            ElementKind::Index { element, .. } => element.name(),
            ElementKind::Invoke { target, .. } => target.name(),
            ElementKind::Member { object, .. } => object.name(),
            ElementKind::Item(item) => item.name(),
            ElementKind::Assignment { target, .. } => target.name(),
            _ => None,
        }
    }
}

impl Labeled<Token> for Item {
    fn name(&self) -> Option<Token> {
        let Item { kind, .. } = self.clone();
        kind.name()
    }
}

impl Labeled<Token> for ItemKind {
    fn name(&self) -> Option<Token> {
        match self {
            ItemKind::Trait { name, .. } => name.name(),
            ItemKind::Variable { target, .. } => target.name(),
            ItemKind::Field { name, .. } => name.name(),
            ItemKind::Structure { name, .. } => name.name(),
            ItemKind::Enum { name, .. } => name.name(),
            ItemKind::Macro { name, .. } => name.name(),
            ItemKind::Function { name, .. } => name.name(),
            _ => None,
        }
    }
}

impl SimilarityMetric<Element, Item> for CaseInsensitiveMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                if query_token.to_string().to_lowercase() == candidate_token.to_string().to_lowercase() {
                    0.95
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

}

impl SimilarityMetric<Element, Item> for PrefixMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                if candidate_lower.starts_with(&query_lower) {
                    0.9 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                } else if query_lower.starts_with(&candidate_lower) {
                    0.9 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Prefix".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Element, Item> for SubstringMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                if candidate_lower.contains(&query_lower) {
                    0.8 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                } else if query_lower.contains(&candidate_lower) {
                    0.8 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

}

impl SimilarityMetric<Element, Item> for AcronymMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                if query_token.to_string().len() > self.max_acronym_length {
                    return 0.0;
                }

                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

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
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Acronym".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Element, Item> for KeyboardProximityMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let s1_lower = query_token.to_string().to_lowercase();
                let s2_lower = candidate_token.to_string().to_lowercase();

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
            _ => 0.0,
        }
    }

}

impl SimilarityMetric<Element, Item> for SuffixMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                if candidate_lower.ends_with(&query_lower) {
                    0.85 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                } else if query_lower.ends_with(&candidate_lower) {
                    0.85 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Suffix".to_string()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SymbolTypeMetric;

#[derive(Debug)]
pub struct ParameterCountMetric;

impl SimilarityMetric<Element, Item> for ParameterCountMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { parameters, .. }, ItemKind::Function { parameters: func_params, .. }) => {
                let query_param_count = parameters.len();
                let candidate_param_count = func_params.len();

                if query_param_count == candidate_param_count {
                    0.9
                } else if (query_param_count as isize - candidate_param_count as isize).abs() <= 2 {
                    0.7 - 0.1 * (query_param_count as isize - candidate_param_count as isize).abs() as f64
                } else {
                    0.0
                }
            },
            (ElementKind::Invoke { parameters, .. }, ItemKind::Macro { parameters: macro_params, .. }) => {
                let query_param_count = parameters.len();
                let candidate_param_count = macro_params.len();

                if query_param_count == candidate_param_count {
                    0.9
                } else if (query_param_count as isize - candidate_param_count as isize).abs() <= 2 {
                    0.7 - 0.1 * (query_param_count as isize - candidate_param_count as isize).abs() as f64
                } else {
                    0.0
                }
            },
            (ElementKind::Constructor { body, .. }, ItemKind::Structure { fields, .. }) => {
                if let ElementKind::Bundle(elements) = &body.kind {
                    let constructor_field_count = elements.len();
                    let struct_field_count = fields.len();

                    if constructor_field_count == struct_field_count {
                        0.9
                    } else if constructor_field_count < struct_field_count {
                        0.8 * (constructor_field_count as f64 / struct_field_count as f64)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            },
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("ParameterCount".to_string()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ContextualRelevanceMetric {
    pub context_weight: f64,
}

impl Default for ContextualRelevanceMetric {
    fn default() -> Self {
        Self {
            context_weight: 0.85,
        }
    }
}

impl SimilarityMetric<Element, Item> for ContextualRelevanceMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match &query.kind {
            ElementKind::Identifier(_) => {
                match &candidate.kind {
                    ItemKind::Variable { .. } => self.context_weight,
                    ItemKind::Function { .. } => self.context_weight - 0.1,
                    ItemKind::Structure { .. } => self.context_weight - 0.2,
                    ItemKind::Enum { .. } => self.context_weight - 0.2,
                    _ => 0.0,
                }
            },
            ElementKind::Invoke { .. } => {
                match &candidate.kind {
                    ItemKind::Function { .. } => self.context_weight,
                    ItemKind::Macro { .. } => self.context_weight - 0.1,
                    _ => 0.0,
                }
            },
            ElementKind::Constructor { .. } => {
                match &candidate.kind {
                    ItemKind::Structure { .. } => self.context_weight,
                    ItemKind::Enum { .. } => self.context_weight - 0.1,
                    _ => 0.0,
                }
            },
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("ContextualRelevance".to_string()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ScopeProximityMetric;

impl SimilarityMetric<Element, Item> for ScopeProximityMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(_), Some(_)) => 0.65,
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("ScopeProximity".to_string()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PartialIdentifierMetric {
    min_length: usize,
}

impl Default for PartialIdentifierMetric {
    fn default() -> Self {
        PartialIdentifierMetric {
            min_length: 3,
        }
    }
}

impl SimilarityMetric<Element, Item> for PartialIdentifierMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Identifier(query_ident), _) => {
                if query_ident.len() < self.min_length {
                    return 0.0;
                }
                match candidate.name() {
                    Some(candidate_token) => {
                        let query_lower = query_ident.to_lowercase();
                        let candidate_lower = candidate_token.to_string().to_lowercase();
                        if candidate_lower.contains(&query_lower) || query_lower.contains(&candidate_lower) {
                            0.75 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                        } else {
                            0.0
                        }
                    }
                    None => 0.0,
                }
            }
            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("PartialIdentifier".to_string()))
        } else {
            None
        }
    }
}

impl PartialEq<Item> for Element {
    fn eq(&self, other: &Item) -> bool {
        match (&self.kind, &other.kind) {
            (ElementKind::Invoke { target, parameters }, ItemKind::Function { name, parameters: func_params, .. }) => {
                target.name() == name.name() && parameters.len() == func_params.len()
            },
            (ElementKind::Invoke { target, parameters }, ItemKind::Macro { name, parameters: macro_params, .. }) => {
                target.name() == name.name() && parameters.len() == macro_params.len()
            },

            (ElementKind::Identifier(ident), ItemKind::Variable { target, .. }) => {
                if let Element { kind: ElementKind::Identifier(target_ident), .. } = *target.clone() {
                    ident == &target_ident
                } else {
                    false
                }
            },

            (ElementKind::Constructor { name: expr_name, .. }, ItemKind::Structure { name: struct_name, .. }) => {
                expr_name.name() == struct_name.name()
            },
            (ElementKind::Constructor { name: expr_name, .. }, ItemKind::Enum { name: enum_name, .. }) => {
                expr_name.name() == enum_name.name()
            },

            _ => false,
        }
    }
}

impl SimilarityMetric<Element, Item> for ExactMatchMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        if query == candidate {
            0.70
        } else {
            0.0
        }
    }

    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        if self.calculate(query, candidate) > 0.0 {
            Some(MatchType::Exact)
        } else {
            None
        }
    }
}

impl SimilarityMetric<Element, Item> for SymbolTypeMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { .. }, ItemKind::Function { .. }) => 0.98,
            (ElementKind::Invoke { .. }, ItemKind::Macro { .. }) => 0.95,
            (ElementKind::Invoke { .. }, ItemKind::Variable { .. }) => 0.0,

            (ElementKind::Identifier(_), ItemKind::Variable { .. }) => 0.95,
            (ElementKind::Identifier(_), ItemKind::Function { .. }) => 0.9,
            (ElementKind::Identifier(_), ItemKind::Macro { .. }) => 0.85,
            (ElementKind::Identifier(_), ItemKind::Structure { .. }) => 0.8,
            (ElementKind::Identifier(_), ItemKind::Enum { .. }) => 0.75,

            (ElementKind::Constructor { .. }, ItemKind::Structure { .. }) => 0.95,
            (ElementKind::Constructor { .. }, ItemKind::Enum { .. }) => 0.9,

            _ => 0.0,
        }
    }


    fn match_type(&self, query: &Element, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("SymbolType".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Element, Item> for TokenSimilarityMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { .. }, ItemKind::Variable { .. }) => return 0.0,
            (ElementKind::Invoke { .. }, ItemKind::Function { .. } | ItemKind::Macro { .. }) => {},
            (ElementKind::Identifier(_), _) => {},
            (ElementKind::Constructor { .. }, ItemKind::Structure { .. } | ItemKind::Enum { .. }) => {},
            _ => return 0.0,
        }

        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let s1_lower = query_token.to_string().to_lowercase();
                let s2_lower = candidate_token.to_string().to_lowercase();

                let s1_tokens = self.split_on_separators(&s1_lower);
                let s2_tokens = self.split_on_separators(&s2_lower);

                self.token_similarity(&s1_tokens, &s2_tokens)
            }
            _ => 0.0,
        }
    }

}

impl SimilarityMetric<Element, Item> for EditDistanceMetric {
    fn calculate(&self, query: &Element, candidate: &Item) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { .. }, ItemKind::Variable { .. }) => return 0.0,
            _ => {},
        }

        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let distance = damerau_levenshtein_distance(&query_token.to_string(), &candidate_token.to_string());
                let max_len = max(query_token.to_string().len(), candidate_token.to_string().len());

                if max_len == 0 {
                    return 1.0;
                }

                1.0 - (distance as f64 / max_len as f64)
            }
            _ => 0.0,
        }
    }

}

pub fn symbol_matcher() -> Matcher<Element, Item> {
    MatcherBuilder::<Element, Item>::new()
        .metric(ExactMatchMetric, 0.30)
        .metric(SymbolTypeMetric, 0.25)
        .metric(ParameterCountMetric, 0.15)
        .metric(ContextualRelevanceMetric::default(), 0.15)
        .metric(CaseInsensitiveMetric, 0.05)
        .metric(PrefixMetric, 0.03)
        .metric(SubstringMetric, 0.03)
        .metric(SuffixMetric, 0.03)
        .metric(EditDistanceMetric, 0.03)
        .metric(TokenSimilarityMetric::default(), 0.02)
        .metric(AcronymMetric::default(), 0.02)
        .metric(KeyboardProximityMetric::default(), 0.02)
        .threshold(0.1)
        .build()
}