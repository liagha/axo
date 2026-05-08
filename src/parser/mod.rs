mod element;
pub mod error;
mod formation;
mod parser;
mod symbol;
mod traits;

pub use {
    element::{Element, ElementKind},
    error::*,
    parser::Parser,
    symbol::{Symbol, SymbolKind},
};

use crate::{
    combinator::{Combinator, Operation, Operator},
    data::memory::Arc,
    internal::{platform::Lock, Session},
    reporter::Error,
};

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

impl<'source>
    Combinator<
        'static,
        Operator<Arc<Lock<Session<'source>>>>,
        Operation<'source, Arc<Lock<Session<'source>>>>,
    > for Parser<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let mut session = operator.store.write().unwrap();
        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Parser::execute(&mut session, &keys);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'source> Default for Parser<'source> {
    fn default() -> Self {
        Parser::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorKind, Parser};
    use crate::{
        parser::ElementKind,
        data::Str,
        scanner::{OperatorKind, PunctuationKind, Scanner, TokenKind},
        tracker::{Peekable, Position},
    };

    fn parse(source: &'static str) -> Parser<'static> {
        let mut scanner = Scanner::new(Position::new(1), Str::from(source));
        scanner.scan();
        assert!(scanner.errors.is_empty());

        let mut parser = Parser::new();
        parser.set_input(scanner.output);
        parser.parse();
        parser
    }

    fn kind<'a>(parser: &'a Parser<'static>) -> &'a ErrorKind<'static> {
        assert_eq!(parser.errors.len(), 1);
        &parser.errors[0].kind
    }

    fn parse_ok(source: &'static str) -> Parser<'static> {
        let parser = parse(source);
        assert!(
            parser.errors.is_empty(),
            "expected no parse error, got {}",
            parser.errors.len()
        );
        assert!(!parser.output.is_empty());
        parser
    }

    #[test]
    fn group_unclosed() {
        let parser = parse("(2");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis))
        ));
    }

    #[test]
    fn collection_unclosed() {
        let parser = parse("[1,2");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket))
        ));
    }

    #[test]
    fn bundle_unclosed() {
        let parser = parse("{1,2");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBrace))
        ));
    }

    #[test]
    fn nested_group_unclosed() {
        let parser = parse("((2");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis))
        ));
    }

    #[test]
    fn nested_collection_unclosed() {
        let parser = parse("[[1,2]");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket))
        ));
    }

    #[test]
    fn nested_mixed_unclosed() {
        let parser = parse("{[1,2}");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftBracket))
        ));
    }

    #[test]
    fn binding_body_missing() {
        let parser = parse("let");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedBody));
    }

    #[test]
    fn structure_head_missing() {
        let parser = parse("struct");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedHead));
    }

    #[test]
    fn structure_body_missing() {
        let parser = parse("struct A");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedBody));
    }

    #[test]
    fn union_head_missing() {
        let parser = parse("union");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedHead));
    }

    #[test]
    fn union_body_missing() {
        let parser = parse("union U");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedBody));
    }

    #[test]
    fn function_name_missing() {
        let parser = parse("func");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedName));
    }

    #[test]
    fn function_head_missing() {
        let parser = parse("func f(");
        assert!(matches!(kind(&parser), ErrorKind::ExpectedHead));
    }

    #[test]
    fn recover_continues() {
        let parser = parse("(2 1+2");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(PunctuationKind::LeftParenthesis))
        ));
    }

    #[test]
    fn recover_many() {
        let parser = parse("(2 [1,2 {3,4");
        assert!(matches!(
            kind(&parser),
            ErrorKind::UnclosedDelimiter(_)
                | ErrorKind::ExpectedBody
                | ErrorKind::UnexpectedToken(_)
        ));
    }

    #[test]
    fn literal_ok() {
        let parser = parse_ok("42");
        assert_eq!(parser.output.len(), 1);
        assert!(matches!(parser.output[0].kind, ElementKind::Literal(_)));
    }

    #[test]
    fn binary_precedence() {
        let parser = parse_ok("1+2*3");
        let root = &parser.output[0];
        let ElementKind::Binary(add) = &root.kind else {
            panic!("expected binary root");
        };
        assert!(matches!(
            add.operator.kind.try_unwrap_operator(),
            Some(OperatorKind::Plus)
        ));
        let ElementKind::Binary(mul) = &add.right.kind else {
            panic!("expected multiply on right branch");
        };
        assert!(matches!(
            mul.operator.kind.try_unwrap_operator(),
            Some(OperatorKind::Star)
        ));
    }

    #[test]
    fn suffix_chain() {
        let parser = parse_ok("a(1)[0]{2}");
        let root = &parser.output[0];
        let ElementKind::Construct(construct) = &root.kind else {
            panic!("expected construct");
        };
        assert!(matches!(construct.target.kind, ElementKind::Index(_)));
        let ElementKind::Index(index) = &construct.target.kind else {
            panic!("expected index");
        };
        assert!(matches!(index.target.kind, ElementKind::Invoke(_)));
    }

    #[test]
    fn binding_ok() {
        let parser = parse_ok("let value = 1");
        assert!(matches!(parser.output[0].kind, ElementKind::Symbolize(_)));
    }

    #[test]
    fn structure_ok() {
        let parser = parse_ok("struct A { let x: i32 }");
        assert!(matches!(parser.output[0].kind, ElementKind::Symbolize(_)));
    }

    #[test]
    fn function_ok() {
        let parser = parse_ok("func f(x): i32 { x }");
        assert!(matches!(parser.output[0].kind, ElementKind::Symbolize(_)));
    }

    #[test]
    fn ignores_comment_and_whitespace() {
        let parser = parse_ok("1 // comment\n + 2");
        assert_eq!(parser.output.len(), 1);
        assert!(matches!(parser.output[0].kind, ElementKind::Binary(_)));
    }

    #[test]
    fn valid_corpus_has_no_parse_errors() {
        let corpus = [
            "1 + 2 * 3",
            "let a = 1",
            "let b: i32 = 2",
            "struct A { let x: i32; let y: i32 }",
            "union U { let i: i32; let f: f64 }",
            "func add(x, y): i32 { x + y }",
            "a(1, 2)[0]{3,4}",
            "{ let x = 1; let y = x + 2; y }",
        ];

        for source in corpus {
            let parser = parse_ok(source);
            assert!(!parser.output.is_empty());
        }
    }

    #[test]
    fn invalid_corpus_has_parse_errors() {
        let corpus = [
            "(",
            "[1,2",
            "{1,2",
            "func",
            "func f(",
            "struct",
            "union",
            "let",
        ];

        for source in corpus {
            let parser = parse(source);
            assert!(
                !parser.errors.is_empty(),
                "expected parse error for input: {}",
                source
            );
        }
    }

    #[test]
    fn long_expression_chain_parses() {
        let mut source = String::from("1");
        for index in 0..1500 {
            source.push_str(" + ");
            source.push_str(&(index % 9 + 1).to_string());
        }

        let parser = parse_ok(Box::leak(source.into_boxed_str()));
        assert_eq!(parser.output.len(), 1);
        assert!(matches!(parser.output[0].kind, ElementKind::Binary(_)));
    }

    #[test]
    fn generated_expressions_parse() {
        for seed in 0..200usize {
            let mut value = seed as u64 + 5;
            let mut source = String::new();
            source.push('(');
            source.push_str(&(seed % 9 + 1).to_string());

            for _ in 0..40 {
                value = value
                    .wrapping_mul(2862933555777941757)
                    .wrapping_add(3037000493);
                let operator = match value % 4 {
                    0 => " + ",
                    1 => " - ",
                    2 => " * ",
                    _ => " / ",
                };
                source.push_str(operator);
                source.push_str(&((value % 9 + 1) as usize).to_string());
            }

            source.push(')');
            let parser = parse_ok(Box::leak(source.into_boxed_str()));
            assert_eq!(parser.output.len(), 1);
        }
    }
}

#[cfg(test)]
mod property {
    use super::Parser;
    use crate::{
        data::Str,
        scanner::Scanner,
        tracker::{Peekable, Position},
    };
    use proptest::prelude::*;

    fn parse(source: &str) -> Parser<'_> {
        let mut scanner = Scanner::new(Position::new(1), Str(source.as_bytes()));
        scanner.scan();

        let mut parser = Parser::new();
        parser.set_input(scanner.output);
        parser.parse();
        parser
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
            Just(':'),
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
        fn parse_never_panics(source in source_strategy()) {
            let parser = parse(&source);
            prop_assert!(parser.errors.len() <= parser.input.len().max(1));
        }

        #[test]
        fn element_spans_are_valid(source in source_strategy()) {
            let parser = parse(&source);
            for element in &parser.output {
                prop_assert!(element.span.start <= element.span.end);
            }
        }
    }
}
