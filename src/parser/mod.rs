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
    combinator::{Action, Operation, Operator},
    data::memory::Arc,
    internal::{platform::Lock, Session},
    reporter::Error,
    tracker::Location,
};

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Parser<'source>
{
    fn action(
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
        Parser::new(Location::from(file!()))
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorKind, Parser};
    use crate::{
        data::Str,
        scanner::{PunctuationKind, Scanner, TokenKind},
        tracker::{Location, Peekable, Position},
    };

    fn parse(source: &'static str) -> Parser<'static> {
        let mut scanner = Scanner::new(Position::new(1), Str::from(source));
        scanner.scan();
        assert!(scanner.errors.is_empty());

        let mut parser = Parser::new(Location::from("test"));
        parser.set_input(scanner.output);
        parser.parse();
        parser
    }

    fn kind<'a>(parser: &'a Parser<'static>) -> &'a ErrorKind<'static> {
        assert_eq!(parser.errors.len(), 1);
        &parser.errors[0].kind
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
        let parser = parse("var");
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
    fn stress_nested_unclosed() {
        for n in [8usize, 16, 32, 64, 128] {
            let mut text = String::new();
            for _ in 0..n {
                text.push('(');
            }
            text.push('1');
            let parser = parse(Box::leak(text.into_boxed_str()));
            assert!(!parser.errors.is_empty());
            assert!(matches!(
                kind(&parser),
                ErrorKind::UnclosedDelimiter(_)
                    | ErrorKind::ExpectedBody
                    | ErrorKind::UnexpectedToken(_)
            ));
        }
    }

    #[test]
    fn stress_malformed_mix() {
        for n in [8usize, 16, 32, 64] {
            let mut text = String::new();
            for _ in 0..n {
                text.push_str("( [ { var ");
            }
            let parser = parse(Box::leak(text.into_boxed_str()));
            assert!(!parser.errors.is_empty());
        }
    }
}
