use {
    super::{
        Resolver,
    },
    crate::{
        data::{Str, Delimited, Interface, Function},
        parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::Span,
    },
};

impl<'resolver> Resolver<'resolver> {
    pub fn builtin(
        target: &Element<'resolver>,
    ) -> Option<Symbol<'resolver>> {
        let name = target.brand().and_then(|token| match token.kind {
            TokenKind::Identifier(identifier) => identifier.as_str().map(|name| name.to_string()),
            _ => None,
        })?;

        match name.as_str() {
            "Int8" => Some(Resolver::function("Int8", "Integer")),
            "Int16" => Some(Resolver::function("Int16", "Integer")),
            "Int32" => Some(Resolver::function("Int32", "Integer")),
            "Int64" => Some(Resolver::function("Int64", "Integer")),
            "Integer" => Some(Resolver::function("Integer", "Integer")),

            "UInt8" => Some(Resolver::function("UInt8", "Integer")),
            "UInt16" => Some(Resolver::function("UInt16", "Integer")),
            "UInt32" => Some(Resolver::function("UInt32", "Integer")),
            "UInt64" => Some(Resolver::function("UInt64", "Integer")),

            "Float32" => Some(Resolver::function("Float32", "Float")),
            "Float64" => Some(Resolver::function("Float64", "Float")),
            "Float" => Some(Resolver::function("Float", "Float")),

            "Boolean" => Some(Resolver::function("Boolean", "Boolean")),
            "Character" => Some(Resolver::function("Character", "Character")),
            "String" => Some(Resolver::function("String", "String")),

            "Void" => Some(Resolver::function("Void", "Void")),

            "if" => Some(Resolver::statement("if")),
            "while" => Some(Resolver::statement("while")),
            "break" => Some(Resolver::statement("break")),
            "continue" => Some(Resolver::statement("continue")),
            "return" => Some(Resolver::statement("return")),

            _ => None,
        }
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
            SymbolKind::Function(Function::new(
                Box::new(target),
                Vec::new(),
                Some(Box::new(body)),
                None,
                Interface::Compiler,
                false,
            )),
            Span::void(),
            Visibility::Public,
        )
    }

    fn function(name: &'static str, output: &'static str) -> Symbol<'resolver> {
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
            SymbolKind::Function(Function::new(
                Box::new(target),
                Vec::new(),
                Some(Box::new(body)),
                Some(Box::new(output_annotation)),
                Interface::Compiler,
                false,
            )),
            Span::void(),
            Visibility::Public,
        )
    }
}
