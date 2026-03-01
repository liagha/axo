# Matchete

Matchete is a versatile Rust library for similarity matching between query and candidate items. Designed for flexibility, it enables custom similarity metrics, weighted metric combinations, and configurable matching behavior via a builder pattern. Whether you're building search engines, recommendation systems, or fuzzy matching tools, Matchete provides a robust, generic framework that works with any data types implementing `Clone` and `Debug`.

The name *Matchete* draws inspiration from "machete," evoking precision and strength in cutting through complex matching tasks, while playfully nodding to its core purpose: finding the best *match*.

## Features

- **Generic Matching**: Supports any query and candidate types with `Clone` and `Debug` traits.
- **Custom Metrics**: Define tailored similarity metrics via the `SimilarityMetric` trait.
- **Weighted Scoring**: Combine multiple metrics with adjustable weights.
- **Composite Matching**: Use `MultiMatcher` to aggregate results from multiple matchers.
- **Flexible Configuration**: Set thresholds, conditional rules, and options with `MatcherBuilder`.
- **Detailed Insights**: Access per-metric scores and match types for in-depth analysis.
- **Extensible**: Easily add new metrics or strategies to suit your needs.

## Installation

Add Matchete to your project by including it in your `Cargo.toml`:

```toml
[dependencies]
matchete = "0.0.3"  # Replace with the actual version
```

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed via rustup.

## Usage

Matchete offers a simple yet powerful API for matching tasks. Below is a basic example of string matching with a custom metric. For more examples, see the [Examples](#examples) section.

```rust
use matchete::{Matcher, SimilarityMetric};

// Define a simple string similarity metric
struct SimpleMetric;

impl SimilarityMetric<String, String> for SimpleMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        if query == candidate { 1.0 } else { 0.5 }
    }

    fn id(&self) -> &str { "simple" }
}

fn main() {
    // Create a matcher
    let matcher = Matcher::<String, String>::new()
        .with_metric(SimpleMetric, 1.0)
        .with_threshold(0.6);

    // Match a query against candidates
    let query = String::from("hello");
    let candidates = vec![String::from("hello"), String::from("world")];
    if let Some(result) = matcher.find_best_match(&query, &candidates) {
        println!("Best match: {} (score: {:.2})", result.candidate, result.score);
    }
}
```

## Examples

The `examples` folder contains practical demonstrations of Matchete's capabilities:

- `string.rs`: Simple string matching with a single metric.
- `multimetrics.rs`: Combining metrics using `MatcherBuilder` with conditional thresholds.
- `numeric.rs`: Matching `f64` numbers with a custom metric.
- `analysis.rs`: Analyzing matches with per-metric score details.
- `multimatcher.rs`: Combining multiple matchers with `MultiMatcher`.

Run an example using:
```bash
cargo run --example basic_string_matching
```

Each example includes detailed comments and instructions. The `examples/common.rs` file provides shared metrics used across these demos.

## Configuration

Customize Matchete's behavior with:
- **Thresholds**: Set minimum scores for matches (default: 0.4).
- **Weights**: Assign importance to each metric.
- **Options**: Configure settings like case sensitivity via key-value pairs.
- **Conditional Thresholds**: Apply dynamic thresholds based on query properties.
- **Metric Groups**: Organize metrics for advanced strategies like fallback chains.

See `multimetrics.rs` for an example of complex configurations.

## Contributing

Contributions are welcome! To contribute:
1. Fork the repository.
2. Create a branch for your feature or fix (`git checkout -b feature/my-feature`).
3. Commit your changes (`git commit -m "Add my feature"`).
4. Push to your branch (`git push origin feature/my-feature`).
5. Open a pull request with a clear description.

Please include tests and update documentation. Adhere to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

Matchete is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions, suggestions, or issues, visit the [GitHub repository](https://github.com/liagha/matchete).