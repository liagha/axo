use {
    super::{Resolver},
    crate::{
        data::Str,
        parser::{Element, ElementKind, SymbolKind},
        scanner::{Token, TokenKind},
        tracker::Span,
    },
};

impl<'registry> Resolver<'registry> {
    fn lookup_value(
        resolver: &mut Resolver<'registry>,
        key: Str<'registry>,
    ) -> Option<Token<'registry>> {
        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(key), Span::void())),
            Span::void(),
        );

        let result = resolver.scope.try_get(&identifier).ok()?;
        if let SymbolKind::Preference(preference) = result.kind {
            Some(preference.value.clone())
        } else {
            None
        }
    }

    fn path(
        resolver: &mut Resolver<'registry>,
        key: &'registry str,
        index: usize,
    ) -> Option<Str<'registry>> {
        let numbered = Str::from(format!("{}({})", key, index));
        let plain = Str::from(key);

        for candidate in [numbered, plain] {
            if let Some(Token {
                kind: TokenKind::Identifier(value),
                ..
            }) = Self::lookup_value(resolver, candidate)
            {
                return Some(value);
            }
        }

        None
    }

    pub fn preference(
        resolver: &mut Resolver<'registry>,
        identifier: Str<'registry>,
    ) -> Option<Token<'registry>> {
        Self::lookup_value(resolver, identifier)
    }

    pub fn verbosity(resolver: &mut Resolver<'registry>) -> u8 {
        match Self::lookup_value(resolver, Str::from("Verbosity")) {
            Some(Token {
                kind: TokenKind::Integer(value),
                ..
            }) => value as u8,
            _ => 0,
        }
    }

    pub fn input(resolver: &mut Resolver<'registry>) -> Str<'registry> {
        for candidate in [Str::from("Input"), Str::from("Input(0)")] {
            if let Some(Token {
                kind: TokenKind::Identifier(path),
                ..
            }) = Self::lookup_value(resolver, candidate)
            {
                return path;
            }
        }
        Str::default()
    }

    pub fn output(resolver: &mut Resolver<'registry>, index: usize) -> Option<Str<'registry>> {
        Self::path(resolver, "Output", index)
    }

    pub fn code(resolver: &mut Resolver<'registry>, index: usize) -> Option<Str<'registry>> {
        Self::path(resolver, "OutputCode", index)
            .or_else(|| Self::path(resolver, "OutputIR", index))
    }

    pub fn binary(resolver: &mut Resolver<'registry>, index: usize) -> Option<Str<'registry>> {
        Self::path(resolver, "OutputBinary", index)
            .or_else(|| Self::path(resolver, "OutputExec", index))
            .or_else(|| Self::path(resolver, "Output", index))
    }

    pub fn run(resolver: &mut Resolver<'registry>) -> bool {
        match Self::lookup_value(resolver, Str::from("Run")) {
            Some(Token {
                kind: TokenKind::Boolean(value),
                ..
            }) => value,
            _ => false,
        }
    }
}
