use crate::axo_matcher::{
    Matcher, MatcherBuilder, SimilarityMetric,
    LevenshteinSimilarity, JaroWinklerSimilarity,
    CosineSimilarity, SoundexSimilarity, WordOverlapSimilarity,
    CompositeSimilarity, CompositeStrategy, ExactMatchMetric
};

pub struct MatcherPresets;

impl MatcherPresets {
    /// Creates a matcher optimized for fuzzy string matching
    pub fn fuzzy_string_matcher() -> Matcher<String, String> {
        MatcherBuilder::<String, String>::new()
            .metric(ExactMatchMetric, 1.0)
            .metric(JaroWinklerSimilarity::default(), 0.8)
            .metric(LevenshteinSimilarity, 0.6)
            .threshold(0.7)
            .option("case_sensitive", "false")
            .build()
    }

    /// Creates a matcher optimized for name matching
    pub fn name_matcher() -> Matcher<String, String> {
        let mut composite = CompositeSimilarity::<String, String>::new("name_matcher", CompositeStrategy::Maximum);
        composite
            .add_metric(SoundexSimilarity::default())
            .add_metric(JaroWinklerSimilarity::default());

        MatcherBuilder::<String, String>::new()
            .metric(ExactMatchMetric, 1.0)
            .metric(composite, 0.8)
            .threshold(0.6)
            .option("case_sensitive", "false")
            .build()
    }

    /// Creates a matcher optimized for address matching
    pub fn address_matcher() -> Matcher<String, String> {
        MatcherBuilder::new()
            .begin_group("token_matching")
            .metric(WordOverlapSimilarity::default(), 0.7)
            .metric(CosineSimilarity::default(), 0.5)
            .end_group()
            .metric(JaroWinklerSimilarity::default(), 0.3)
            .threshold(0.6)
            .option("normalize_addresses", "true")
            .build()
    }

    /// Creates a matcher optimized for numeric proximity
    pub fn numeric_matcher() -> Matcher<f64, f64> {
        // We'd need to implement NumericSimilarity separately
        struct NumericSimilarity {
            max_difference: f64,
        }

        impl SimilarityMetric<f64, f64> for NumericSimilarity {
            fn calculate(&self, query: &f64, candidate: &f64) -> f64 {
                let diff = (query - candidate).abs();
                if diff > self.max_difference {
                    0.0
                } else {
                    1.0 - (diff / self.max_difference)
                }
            }

            fn id(&self) -> &str {
                "numeric_proximity"
            }
        }

        MatcherBuilder::new()
            .metric(NumericSimilarity { max_difference: 1000.0 }, 1.0)
            .threshold(0.8)
            .build()
    }

    /// Creates a matcher optimized for document/text similarity
    pub fn document_matcher() -> Matcher<String, String> {
        MatcherBuilder::new()
            .metric(WordOverlapSimilarity::default(), 0.6)
            .metric(CosineSimilarity::new(3), 0.4) // trigrams
            .threshold(0.5)
            .option("normalize_whitespace", "true")
            .option("ignore_case", "true")
            .build()
    }

    /// Creates a customizable fuzzy matcher with sensible defaults
    pub fn customize_fuzzy_matcher() -> MatcherBuilder<String, String> {
        MatcherBuilder::<String, String>::new()
            .metric(ExactMatchMetric, 1.0)
            .metric(JaroWinklerSimilarity::default(), 0.7)
            .metric(LevenshteinSimilarity, 0.5)
            .threshold(0.6)
    }
}