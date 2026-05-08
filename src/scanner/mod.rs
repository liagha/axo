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
    data::memory::Arc,
    internal::{platform::Lock, Session},
    reporter::Error,
};

impl<'source>
    Combinator<
        'static,
        crate::combinator::Operator<Arc<Lock<Session<'source>>>>,
        Operation<'source, Arc<Lock<Session<'source>>>>,
    > for Scanner<'source>
{
    fn combinator(
        &self,
        operator: &mut crate::combinator::Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
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
}
