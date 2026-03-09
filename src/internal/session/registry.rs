use {
    super::{Resolver},
    crate::{
        data::Str,
        parser::{Element, ElementKind, SymbolKind},
        scanner::{Token, TokenKind},
        tracker::Span,
    },
};
use crate::data::Identity;

impl<'registry> Resolver<'registry> {
    fn lookup_value(
        &mut self,
        key: Str<'registry>,
    ) -> Option<Token<'registry>> {
        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(key), Span::void())),
            Span::void(),
        );

        let result = self.scope.lookup(&identifier).ok()?;

        if let SymbolKind::Preference(preference) = result.kind {
            Some(preference.value.clone())
        } else {
            None
        }
    }

    fn path(
        &mut self,
        key: &'registry str,
        index: usize,
    ) -> Option<Str<'registry>> {
        let numbered = Str::from(format!("{}({})", key, index));
        let plain = Str::from(key);

        for candidate in [numbered, plain] {
            if let Some(Token {
                kind: TokenKind::Identifier(value),
                ..
            }) = self.lookup_value(candidate)
            {
                return Some(value);
            }
        }

        None
    }

    pub fn preference(
        &mut self,
        identifier: Str<'registry>,
    ) -> Option<Token<'registry>> {
        self.lookup_value(identifier)
    }

    pub fn verbosity(&mut self) -> u8 {
        match self.lookup_value(Str::from("Verbosity")) {
            Some(Token {
                kind: TokenKind::Integer(value),
                ..
            }) => value as u8,
            _ => 1,
        }
    }

    pub fn input(&mut self) -> Str<'registry> {
        for candidate in [Str::from("Input"), Str::from("Input(0)")] {
            if let Some(Token {
                kind: TokenKind::Identifier(path),
                ..
            }) = self.lookup_value(candidate)
            {
                return path;
            }
        }
        Str::default()
    }

    pub fn schema(&mut self, identity: Identity) -> Option<Str<'registry>> {
        Self::path(self, "Output", identity)
    }
}
