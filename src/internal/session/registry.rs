use {
    crate::{
        internal::session::Resolver,
        data::Str,
        parser::{Element, ElementKind, SymbolKind},
        scanner::{Token, TokenKind},
        tracker::Span,
    },
};

impl<'registry> Resolver<'registry> {
    fn get_directive(&mut self, key: Str<'registry>) -> Option<Token<'registry>> {
        let directive = self.registry.values().find(|symbol| {
            symbol.target() == Some(Str::from("directive"))
        })?.clone();

        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(key), Span::void())),
            Span::void(),
        );

        let scope = directive.scope;
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

    pub fn verbosity(&mut self) -> u8 {
        match self.get_directive(Str::from("Verbosity")) {
            Some(Token {
                     kind: TokenKind::Integer(value),
                     ..
                 }) => value as u8,
            _ => 0,
        }
    }
}
