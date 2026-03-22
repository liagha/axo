use {
    crate::{
        internal::session::Resolver,
        data::{Identity, Str},
        parser::{Element, ElementKind, SymbolKind},
        scanner::{Token, TokenKind},
        tracker::Span,
    },
};

impl<'registry> Resolver<'registry> {
    fn get_configuration(&mut self, key: Str<'registry>) -> Option<Token<'registry>> {
        let configuration = self.registry.values().find(|symbol| {
            symbol.target() == Some(Str::from("configuration"))
        })?.clone();

        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(key), Span::void())),
            Span::void(),
        );

        let scope = configuration.scope;
        let result = scope.lookup(&identifier, self).ok()?;

        if let SymbolKind::Binding(binding) = result.kind {
            if let Some(value) = binding.value {
                if let ElementKind::Literal(literal) = value.kind {
                    return Some(literal);
                }
            }
        }

        None
    }

    fn path(&mut self, key: &'registry str, index: usize) -> Option<Str<'registry>> {
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

    pub fn configuration(&mut self, identifier: Str<'registry>) -> Option<Token<'registry>> {
        self.get_configuration(identifier)
    }

    pub fn verbosity(&mut self) -> u8 {
        match self.get_configuration(Str::from("Verbosity")) {
            Some(Token {
                     kind: TokenKind::Integer(value),
                     ..
                 }) => value as u8,
            _ => 0,
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
