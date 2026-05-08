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
}
