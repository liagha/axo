mod classifier;
mod element;
pub mod error;
mod parser;
mod symbol;
mod traits;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::{Symbol, SymbolKind, Visibility},
    error::*,
};

use {
    broccli::Color,
    crate::{
        reporter::Error,
        combinator::{Action, Operation, Operator},
        data::{
            Identity,
            memory::Arc,
        },
        format::Show,
        internal::{
            platform::Lock,
            time::Duration,
            SessionError, RecordKind, Session,
        },
        tracker::{Peekable, Location},
    },
};

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

pub fn parse<'source>(session: &mut Session<'source>, keys: &[Identity]) {
    use crate::parser::Parser;

    for &key in keys {
        let (kind, hash, dirty, location, tokens) = {
            let record = session.records.get(&key).unwrap();
            (
                record.kind.clone(),
                record.hash,
                record.dirty,
                record.location,
                record.tokens.clone(),
            )
        };

        if kind != RecordKind::Source {
            continue;
        }

        if !dirty {
            if let Some(mut elements) = session.cache::<Vec<Element>>("elements", hash, None) {
                elements.shrink_to_fit();
                let record = session.records.get_mut(&key).unwrap();
                record.elements = Some(elements);
                record.tokens = None;
                continue;
            }
        }

        let mut parser = Parser::new(location);
        parser.set_input(tokens.unwrap());
        parser.parse();

        if let Some(stencil) = session.get_stencil() {
            session.report_section(
                "Elements",
                Color::Cyan,
                parser.output.format(stencil).to_string(),
            );
        }

        parser.output.shrink_to_fit();

        session.errors.extend(
            parser
                .errors
                .iter()
                .map(|error| SessionError::Parse(error.clone())),
        );

        let elements = session.cache("elements", hash, Some(parser.output));
        let record = session.records.get_mut(&key).unwrap();
        record.elements = elements;
        record.tokens = None;
    }
}

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
    ) -> () {
        let mut session = operator.store.write().unwrap();
        let initial = session.errors.len();
        session.report_start("parsing");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();
        parse(&mut session, &keys);

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("parsing", duration, session.errors.len() - initial);

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
            ErrorKind::UnclosedDelimiter(_) | ErrorKind::ExpectedBody | ErrorKind::UnexpectedToken(_)
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
                ErrorKind::UnclosedDelimiter(_) | ErrorKind::ExpectedBody | ErrorKind::UnexpectedToken(_)
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
