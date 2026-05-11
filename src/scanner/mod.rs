mod character;
mod error;
mod formation;
mod operator;
mod punctuation;
mod scanner;
mod token;
mod traits;

pub use {character::Character, error::*, operator::*, punctuation::*, scanner::Scanner, token::*};

pub type ScanError<'error> = Error<'error, ErrorKind<'error>>;

use crate::{
    combinator::{Combinator, Operation},
    internal::session::Store,
    reporter::Error,
};

impl<'op, 'source>
Combinator<
    'static,
    (&'op mut crate::combinator::Operator<Store<'source>>, &'op mut Operation<'source, Store<'source>>),
> for Scanner<'source>
{
    fn combinator(
        &self,
        joint: &mut (&'op mut crate::combinator::Operator<Store<'source>>, &'op mut Operation<'source, Store<'source>>),
    ) {
        let (operator, operation) = (&mut joint.0, &mut joint.1);

        let mut session = operator.store.write().unwrap();
        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Scanner::execute(&mut session, &keys);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}
impl<'source> Default for Scanner<'source> {
    fn default() -> Self {
        let position = crate::tracker::Position::new(0);
        Scanner::new(position, crate::data::Str::from(""))
    }
}

#[cfg(test)]
mod tests {
    use super::{OperatorKind, PunctuationKind, Scanner, TokenKind};
    use crate::{
        data::Str,
        tracker::Position,
    };

    fn scan(source: &'static str) -> Scanner<'static> {
        let mut scanner = Scanner::new(Position::new(1), Str::from(source));
        scanner.scan();
        scanner
    }

    fn compact(scanner: &Scanner<'static>) -> Vec<TokenKind<'static>> {
        scanner
            .output
            .iter()
            .filter_map(|token| match token.kind {
                TokenKind::Punctuation(PunctuationKind::Space)
                | TokenKind::Punctuation(PunctuationKind::Newline)
                | TokenKind::Punctuation(PunctuationKind::Return)
                | TokenKind::Punctuation(PunctuationKind::Tab) => None,
                _ => Some(token.kind.clone()),
            })
            .collect()
    }

    #[test]
    fn scans_number_variants() {
        let scanner = scan("42 3.5 1e3");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert_eq!(kinds.len(), 3);
        assert!(matches!(kinds[0], TokenKind::Integer(42)));
        assert!(matches!(kinds[1], TokenKind::Float(_)));
        assert!(matches!(kinds[2], TokenKind::Float(_)));
    }

    #[test]
    fn scans_identifier_and_boolean() {
        let scanner = scan("true false name _name2");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert_eq!(kinds.len(), 4);
        assert!(matches!(kinds[0], TokenKind::Boolean(true)));
        assert!(matches!(kinds[1], TokenKind::Boolean(false)));
        assert!(matches!(kinds[2], TokenKind::Identifier(_)));
        assert!(matches!(kinds[3], TokenKind::Identifier(_)));
    }

    #[test]
    fn scans_operator_and_punctuation() {
        let scanner = scan("+= == && || .. ... ( ) [ ] { } , ;");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert_eq!(kinds.len(), 14);
        assert!(matches!(
            kinds[0].try_unwrap_operator(),
            Some(OperatorKind::Composite(op)) if op.as_slice() == [OperatorKind::Plus, OperatorKind::Equal]
        ));
        assert!(matches!(
            kinds[1].try_unwrap_operator(),
            Some(OperatorKind::Composite(op)) if op.as_slice() == [OperatorKind::Equal, OperatorKind::Equal]
        ));
        assert!(matches!(
            kinds[2].try_unwrap_operator(),
            Some(OperatorKind::Composite(op)) if op.as_slice() == [OperatorKind::Ampersand, OperatorKind::Ampersand]
        ));
        assert!(matches!(
            kinds[3].try_unwrap_operator(),
            Some(OperatorKind::Composite(op)) if op.as_slice() == [OperatorKind::Pipe, OperatorKind::Pipe]
        ));
        assert!(matches!(
            kinds[4].try_unwrap_operator(),
            Some(OperatorKind::Composite(op)) if op.as_slice() == [OperatorKind::Dot, OperatorKind::Dot]
        ));
        assert!(matches!(
            kinds[5].try_unwrap_operator(),
            Some(OperatorKind::Composite(op)) if op.as_slice() == [OperatorKind::Dot, OperatorKind::Dot, OperatorKind::Dot]
        ));
        assert!(matches!(kinds[6], TokenKind::Punctuation(PunctuationKind::LeftParenthesis)));
        assert!(matches!(kinds[7], TokenKind::Punctuation(PunctuationKind::RightParenthesis)));
        assert!(matches!(kinds[8], TokenKind::Punctuation(PunctuationKind::LeftBracket)));
        assert!(matches!(kinds[9], TokenKind::Punctuation(PunctuationKind::RightBracket)));
        assert!(matches!(kinds[10], TokenKind::Punctuation(PunctuationKind::LeftBrace)));
        assert!(matches!(kinds[11], TokenKind::Punctuation(PunctuationKind::RightBrace)));
        assert!(matches!(kinds[12], TokenKind::Punctuation(PunctuationKind::Comma)));
        assert!(matches!(kinds[13], TokenKind::Punctuation(PunctuationKind::Semicolon)));
    }

    #[test]
    fn scans_string_character_and_backtick() {
        let scanner = scan("\"a\\n\\x41\" '\\u0041' `b\\t`");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert_eq!(kinds.len(), 3);
        assert!(matches!(kinds[0].try_unwrap_string(), Some(value) if value.as_str() == Some("a\nA")));
        assert!(matches!(kinds[1], TokenKind::Character('A')));
        assert!(matches!(kinds[2].try_unwrap_string(), Some(value) if value.as_str() == Some("b\t")));
    }

    #[test]
    fn scans_comments() {
        let scanner = scan("a//line\nb/*ok*/c");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert_eq!(kinds.len(), 5);
        assert!(matches!(kinds[0], TokenKind::Identifier(_)));
        assert!(matches!(kinds[1], TokenKind::Comment(_)));
        assert!(matches!(kinds[2], TokenKind::Identifier(_)));
        assert!(matches!(kinds[3], TokenKind::Comment(_)));
        assert!(matches!(kinds[4], TokenKind::Identifier(_)));
    }

    #[test]
    fn tokenizes_invalid_escape_sequence() {
        let scanner = scan("\"\\q\"");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert!(!kinds.is_empty());
        assert!(
            kinds
                .iter()
                .any(|kind| matches!(kind.try_unwrap_identifier(), Some(name) if name.as_str() == Some("q")))
        );
    }

    #[test]
    fn tokenizes_at_operator() {
        let scanner = scan("@");
        assert!(scanner.errors.is_empty());
        let kinds = compact(&scanner);
        assert_eq!(kinds.len(), 1);
        assert!(matches!(
            kinds[0].try_unwrap_operator(),
            Some(OperatorKind::At)
        ));
    }

    #[test]
    fn valid_corpus_has_no_scan_errors() {
        let corpus = [
            "let a = 1 + 2 * 3",
            "let s = \"hello\\nworld\"",
            "let c = '\\u0041'",
            "func f(x): i32 { x + 1 }",
            "struct A { let x: i32; let y: i32 }",
            "union U { let i: i32; let f: f64 }",
            "a(1,2)[0]{3,4}",
            "// line\n/* block */ let b = 0",
        ];

        for source in corpus {
            let scanner = scan(source);
            assert!(
                scanner.errors.is_empty(),
                "scanner errors for corpus input: {}",
                source
            );
            assert!(!scanner.output.is_empty());
        }
    }

    #[test]
    fn stress_large_input() {
        let mut source = String::new();
        for index in 0..3000 {
            source.push_str("let v");
            source.push_str(&index.to_string());
            source.push_str(" = ");
            source.push_str(&(index % 97).to_string());
            source.push_str(" + ");
            source.push_str(&((index + 3) % 89).to_string());
            source.push('\n');
        }

        let scanner = scan(Box::leak(source.into_boxed_str()));
        assert!(scanner.errors.is_empty());
        assert!(!scanner.output.is_empty());
    }

    #[test]
    fn generated_inputs_stay_stable() {
        let alphabet = [
            "a", "b", "1", "2", "+", "-", "*", "/", "(", ")", "{", "}", "[", "]", ",", ";",
            "\"x\"", "'y'", "true", "false", " ",
        ];

        for seed in 0..120usize {
            let mut value = seed as u64 + 17;
            let mut source = String::new();

            for _ in 0..80 {
                value = value
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let index = (value as usize) % alphabet.len();
                source.push_str(alphabet[index]);
            }

            let scanner = scan(Box::leak(source.into_boxed_str()));
            assert!(scanner.errors.len() <= scanner.input.len());
            assert!(scanner.output.len() + scanner.errors.len() > 0);
        }
    }

    #[test]
    fn debug_failing_span() {
        let source = "a\"\"";
        let mut scanner = Scanner::new(Position::new(1), Str::from(source));
        scanner.scan();

        let mut previous = 0;
        for (i, token) in scanner.output.iter().enumerate() {
            assert!(
                token.span.start <= token.span.end,
                "Token [{}] has invalid span: {:?}", i, token.span
            );
            assert!(
                token.span.start >= previous,
                "Token [{}] starts before previous token ends: {:?} (previous ended at {})",
                i, token.span, previous
            );
            previous = token.span.end;
        }

        assert_eq!(scanner.output.len(), 2);
        assert!(matches!(scanner.output[0].kind, TokenKind::Identifier(_)));
        assert!(matches!(scanner.output[1].kind, TokenKind::String(_)));
        assert!(scanner.output[0].span.start <= scanner.output[0].span.end);
        assert!(scanner.output[1].span.start <= scanner.output[1].span.end);
        assert!(scanner.output[0].span.end <= scanner.output[1].span.start);
    }
}

#[cfg(test)]
mod property {
    use super::Scanner;
    use crate::{
        data::Str,
        tracker::Position,
    };
    use proptest::prelude::*;

    fn scan(source: &str) -> Scanner<'_> {
        let mut scanner = Scanner::new(Position::new(1), Str(source.as_bytes()));
        scanner.scan();
        scanner
    }

    fn source_strategy() -> impl Strategy<Value = String> {
        let alphabet = prop_oneof![
            Just('a'),
            Just('b'),
            Just('x'),
            Just('y'),
            Just('0'),
            Just('1'),
            Just('2'),
            Just('+'),
            Just('-'),
            Just('*'),
            Just('/'),
            Just('='),
            Just('('),
            Just(')'),
            Just('['),
            Just(']'),
            Just('{'),
            Just('}'),
            Just(','),
            Just(';'),
            Just('"'),
            Just('\''),
            Just('\\'),
            Just(' '),
            Just('\n'),
            Just('\t'),
        ];

        prop::collection::vec(alphabet, 0..400).prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn scan_never_panics(source in source_strategy()) {
            let scanner = scan(&source);
            prop_assert!(scanner.errors.len() <= scanner.input.len());
            prop_assert!(scanner.output.len() + scanner.errors.len() > 0 || source.is_empty());
        }

        #[test]
        fn token_spans_are_ordered(source in source_strategy()) {
            let scanner = scan(&source);
            let mut previous = 0;

            for token in &scanner.output {
                prop_assert!(token.span.start <= token.span.end);
                prop_assert!(token.span.start >= previous);
                previous = token.span.end;
            }
        }
    }
}
