use {
    super::{Resolver},
    crate::{
        data::{Str, Identity},
        parser::{Element, ElementKind, SymbolKind},
        scanner::{Token, TokenKind},
        tracker::Span,
    },
};

impl<'registry> Resolver<'registry> {
    fn get_configuration(
        &mut self,
        key: Str<'registry>,
    ) -> Option<Token<'registry>> {
        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(key), Span::void())),
            Span::void(),
        );

        let result = self.scope.lookup(&identifier).ok()?;

        if let SymbolKind::Binding(binding) = result.kind {
            binding.value.as_ref().map(|v| v.brand().clone().unwrap_or_else(|| unreachable!())).cloned()
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
            }) = self.get_configuration(candidate)
            {
                return Some(value);
            }
        }

        None
    }

    pub fn configuration(
        &mut self,
        identifier: Str<'registry>,
    ) -> Option<Token<'registry>> {
        self.get_configuration(identifier)
    }

    pub fn verbosity(&mut self) -> u8 {
        match self.get_configuration(Str::from("Verbosity")) {
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
            }) = self.get_configuration(candidate)
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
