use {
    super::{
        Resolver,
        scope::Scope,
    },
    crate::{
        data::{Str, Delimited, Interface, Method, Module},
        parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::Span,
    },
};

impl<'resolver> Resolver<'resolver> {
    pub fn builtin(
        target: &Element<'resolver>,
        scope: &Scope<'resolver>,
    ) -> Option<Symbol<'resolver>> {
        let name = target.brand().and_then(|token| match token.kind {
            TokenKind::Identifier(identifier) => identifier.as_str().map(|name| name.to_string()),
            _ => None,
        })?;

        match name.as_str() {
            "compiler" => Some(Resolver::compiler(scope)),
            "Int32" => Some(Resolver::method("Int32", "Integer")),
            "Int64" => Some(Resolver::method("Int64", "Integer")),
            "Float" => Some(Resolver::method("Float", "Float")),
            "Boolean" => Some(Resolver::method("Boolean", "Boolean")),
            "String" => Some(Resolver::method("String", "String")),
            "Character" => Some(Resolver::method("Character", "Character")),
            "Char" => Some(Resolver::method("Char", "Character")),
            "Unit" => Some(Resolver::method("Unit", "Unit")),
            "Integer" => Some(Resolver::method("Integer", "Integer")),
            "if" => Some(Resolver::statement("if")),
            "while" => Some(Resolver::statement("while")),
            "for" => Some(Resolver::statement("for")),
            "break" => Some(Resolver::statement("break")),
            "continue" => Some(Resolver::statement("continue")),
            "return" => Some(Resolver::statement("return")),
            _ => None,
        }
    }

    fn compiler(scope: &Scope<'resolver>) -> Symbol<'resolver> {
        let identifier = Element::new(
            ElementKind::Literal(Token::new(
                TokenKind::Identifier(Str::from("compiler")),
                Span::void(),
            )),
            Span::void(),
        );

        Symbol::new(
            0,
            SymbolKind::Module(Module::new(Box::new(identifier))),
            Span::void(),
            Visibility::Public,
        )
            .with_scope(scope.root().clone())
    }

    fn statement(name: &'static str) -> Symbol<'resolver> {
        let target = Element::new(
            ElementKind::Literal(Token::new(
                TokenKind::Identifier(Str::from(name)),
                Span::void(),
            )),
            Span::void(),
        );

        let body = Element::new(
            ElementKind::Delimited(Delimited::new(
                Token::new(
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    Span::void(),
                ),
                Vec::new(),
                None,
                Token::new(
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                    Span::void(),
                ),
            )),
            Span::void(),
        );

        Symbol::new(
            0,
            SymbolKind::Method(Method::new(
                Box::new(target),
                Vec::new(),
                Box::new(body),
                None,
                Interface::Compiler,
                true,
                false,
            )),
            Span::void(),
            Visibility::Public,
        )
    }

    fn method(name: &'static str, output: &'static str) -> Symbol<'resolver> {
        let target = Element::new(
            ElementKind::Literal(Token::new(
                TokenKind::Identifier(Str::from(name)),
                Span::void(),
            )),
            Span::void(),
        );

        let output_annotation = Element::new(
            ElementKind::Literal(Token::new(
                TokenKind::Identifier(Str::from(output)),
                Span::void(),
            )),
            Span::void(),
        );

        let body = match output {
            "Integer" => Element::new(
                ElementKind::Literal(Token::new(TokenKind::Integer(0), Span::void())),
                Span::void(),
            ),
            "Float" => Element::new(
                ElementKind::Literal(Token::new(TokenKind::Integer(0), Span::void())),
                Span::void(),
            ),
            "Boolean" => Element::new(
                ElementKind::Literal(Token::new(TokenKind::Boolean(false), Span::void())),
                Span::void(),
            ),
            "Character" => Element::new(
                ElementKind::Literal(Token::new(TokenKind::Character('a'), Span::void())),
                Span::void(),
            ),
            "String" => Element::new(
                ElementKind::Literal(Token::new(TokenKind::String(Str::from("")), Span::void())),
                Span::void(),
            ),
            "Unit" => Element::new(
                ElementKind::Delimited(Delimited::new(
                    Token::new(
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        Span::void(),
                    ),
                    Vec::new(),
                    None,
                    Token::new(
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                        Span::void(),
                    ),
                )),
                Span::void(),
            ),
            _ => Element::new(
                ElementKind::Delimited(Delimited::new(
                    Token::new(
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        Span::void(),
                    ),
                    Vec::new(),
                    None,
                    Token::new(
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                        Span::void(),
                    ),
                )),
                Span::void(),
            ),
        };

        Symbol::new(
            0,
            SymbolKind::Method(Method::new(
                Box::new(target),
                Vec::new(),
                Box::new(body),
                Some(Box::new(output_annotation)),
                Interface::Compiler,
                true,
                false,
            )),
            Span::void(),
            Visibility::Public,
        )
    }
}
