use {
    super::{
        brand::Branded
    },
    crate::{
        axo_scanner::Token,
        axo_parser::{Element, ElementKind, Symbol, SymbolKind},
    },
    matchete::{
        Resemblance, Assessor,
        prelude::*,
    },
};

/// Token-Token Similarity Metrics

impl Resemblance<Token, Token> for JaroWinklerScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for CosineScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for ExactMatchScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for CaseInsensitiveScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for PrefixScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for SuffixScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for SubstringScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for EditDistanceScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for TokenSimilarityScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}
impl Resemblance<Token, Token> for AcronymScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for KeyboardProximityScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for FuzzySearchScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for PhoneticScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for NGramScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

impl Resemblance<Token, Token> for WordOverlapScorer {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        self.perfect(&query.to_string(), &candidate.to_string())
    }
}

/// Metric that provides a small bonus for matching token kinds
#[derive(Debug)]
pub struct TokenKindMetric;

impl Resemblance<Token, Token> for TokenKindMetric {
    fn resemblance(&self, query: &Token, candidate: &Token) -> f64 {
        if query.kind == candidate.kind {
            0.2
        } else {
            0.0
        }
    }

    fn perfect(&self, query: &Token, candidate: &Token) -> bool {
        query == candidate
    }
}

/// Element-Symbol Similarity Metrics

impl Resemblance<Element, Symbol> for JaroWinklerScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for CosineScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for ExactMatchScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for CaseInsensitiveScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for PrefixScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for SuffixScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for SubstringScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for EditDistanceScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        if matches!(query.kind, ElementKind::Invoke { .. }) &&
            matches!(candidate.kind, SymbolKind::Binding { .. }) {
            return 0.0;
        }
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for TokenSimilarityScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { .. }, SymbolKind::Binding { .. }) => return 0.0,
            (ElementKind::Invoke { .. }, SymbolKind::Function { .. }) => {},
            (ElementKind::Identifier(_), _) => {},
            (ElementKind::Constructor { .. }, SymbolKind::Structure { .. } | SymbolKind::Enumeration { .. }) => {},
            _ => return 0.0,
        }
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for AcronymScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for KeyboardProximityScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for FuzzySearchScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for PhoneticScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for NGramScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

impl Resemblance<Element, Symbol> for WordOverlapScorer {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.resemblance(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => 0.0,
        }
    }

    fn perfect(&self, query: &Element, candidate: &Symbol) -> bool {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                self.perfect(&query_token.to_string(), &candidate_token.to_string())
            }
            _ => false,
        }
    }
}

/// Specialized Metrics for Element-Symbol Matching

#[derive(Debug)]
pub struct SymbolTypeMetric;

impl Resemblance<Element, Symbol> for SymbolTypeMetric {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { .. }, SymbolKind::Function { .. }) => 0.98,
            (ElementKind::Invoke { .. }, SymbolKind::Binding { .. }) => 0.0,
            (ElementKind::Identifier(_), SymbolKind::Binding { .. }) => 0.95,
            (ElementKind::Identifier(_), SymbolKind::Function { .. }) => 0.9,
            (ElementKind::Identifier(_), SymbolKind::Structure { .. }) => 0.8,
            (ElementKind::Identifier(_), SymbolKind::Enumeration { .. }) => 0.75,
            (ElementKind::Constructor { .. }, SymbolKind::Structure { .. }) => 0.95,
            (ElementKind::Constructor { .. }, SymbolKind::Enumeration { .. }) => 0.9,
            _ => 0.0,
        }
    }

    fn perfect(&self, _query: &Element, _candidate: &Symbol) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct ParameterCountMetric;

impl Resemblance<Element, Symbol> for ParameterCountMetric {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (&query.kind, &candidate.kind) {
            (ElementKind::Constructor { fields: f1, .. }, SymbolKind::Structure { fields: f2, .. }) => {
                let constructor = f1.len();
                let structure = f2.len();
                if constructor == structure {
                    0.9
                } else if constructor < structure {
                    0.8 * (constructor as f64 / structure as f64)
                } else {
                    0.0
                }
            }
            _ => 0.0,
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

impl Resemblance<Element, Symbol> for ContextualRelevanceMetric {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match &query.kind {
            ElementKind::Identifier(_) => {
                match &candidate.kind {
                    SymbolKind::Binding { .. } => self.context_weight,
                    SymbolKind::Function { .. } => self.context_weight - 0.1,
                    SymbolKind::Structure { .. } => self.context_weight - 0.2,
                    SymbolKind::Enumeration { .. } => self.context_weight - 0.2,
                    _ => 0.0,
                }
            }
            ElementKind::Invoke { .. } => {
                match &candidate.kind {
                    SymbolKind::Function { .. } => self.context_weight,
                    _ => 0.0,
                }
            }
            ElementKind::Constructor { .. } => {
                match &candidate.kind {
                    SymbolKind::Structure { .. } => self.context_weight,
                    SymbolKind::Enumeration { .. } => self.context_weight - 0.1,
                    _ => 0.0,
                }
            }
            _ => 0.0,
        }
    }
}

#[derive(Debug)]
pub struct ScopeProximityMetric;

impl Resemblance<Element, Symbol> for ScopeProximityMetric {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(_), Some(_)) => 0.65,
            _ => 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PartialIdentifierMetric {
    min_length: usize,
}

impl Default for PartialIdentifierMetric {
    fn default() -> Self {
        Self {
            min_length: 3,
        }
    }
}

impl Resemblance<Element, Symbol> for PartialIdentifierMetric {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> f64 {
        match &query.kind {
            ElementKind::Identifier(query_ident) => {
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
}

/// Exact Matching

impl PartialEq<Symbol> for Element {
    fn eq(&self, other: &Symbol) -> bool {
        match (&self.kind, &other.kind) {
            (ElementKind::Identifier(ident), SymbolKind::Binding { target, .. }) => {
                if let ElementKind::Identifier(target_ident) = &target.kind {
                    ident == target_ident
                } else {
                    false
                }
            }
            (ElementKind::Constructor { name: expr_name, .. }, SymbolKind::Structure { name: struct_name, .. }) => {
                expr_name.name() == struct_name.name()
            }
            (ElementKind::Constructor { name: expr_name, .. }, SymbolKind::Enumeration { name: enum_name, .. }) => {
                expr_name.name() == enum_name.name()
            }
            _ => false,
        }
    }
}

/// Creates a configured assessor for matching AST elements to symbols
pub fn symbol_matcher() -> Assessor<Element, Symbol> {
    Assessor::<Element, Symbol>::new()
        .floor(0.1)
        .with(ExactMatchScorer, 0.30)
        .with(SymbolTypeMetric, 0.25)
        .with(ParameterCountMetric, 0.15)
        .with(ContextualRelevanceMetric::default(), 0.15)
        .with(JaroWinklerScorer::default(), 0.05)
        .with(CosineScorer::default(), 0.05)
        .with(CaseInsensitiveScorer, 0.05)
        .with(PrefixScorer, 0.03)
        .with(SubstringScorer, 0.03)
        .with(SuffixScorer, 0.03)
        .with(EditDistanceScorer, 0.03)
        .with(TokenSimilarityScorer::default(), 0.02)
        .with(AcronymScorer::default(), 0.02)
        .with(KeyboardProximityScorer::default(), 0.02)
        .with(FuzzySearchScorer::default(), 0.02)
        .with(PhoneticScorer::default(), 0.02)
        .with(NGramScorer::default(), 0.02)
        .with(WordOverlapScorer::default(), 0.02)
        .with(ScopeProximityMetric, 0.01)
        .with(PartialIdentifierMetric::default(), 0.01)
}