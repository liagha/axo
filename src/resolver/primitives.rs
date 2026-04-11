use crate::{
    data::{Delimited, Function, Interface, Str},
    parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
    resolver::Resolver,
    scanner::{PunctuationKind, Token, TokenKind},
    tracker::Span,
};

#[allow(dead_code)]
pub enum Primitive {
    Import = 0,
    If = 1,
    While = 2,
    Continue = 3,
    Break = 4,
    Return = 5,
    Int8 = 6,
    Int16 = 7,
    Int32 = 8,
    Int64 = 9,
    UInt8 = 10,
    UInt16 = 11,
    UInt32 = 12,
    UInt64 = 13,
    Float32 = 14,
    Float64 = 15,
    Boolean = 16,
    Character = 17,
    String = 18,
    Void = 19,
}

impl<'resolver> Resolver<'resolver> {
    pub fn builtin(target: &Element<'resolver>) -> Option<Symbol<'resolver>> {
        let name = target.target()?;

        match name.as_str()? {
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
            ElementKind::literal(Token::new(
                TokenKind::string(Str::from(name)),
                Span::void(),
            )),
            Span::void(),
        );

        let body = Element::new(
            ElementKind::Delimited(Box::new(Delimited::new(
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
            ))),
            Span::void(),
        );

        Symbol::new(
            SymbolKind::function(Function::new(
                Box::new(target),
                Vec::new(),
                Some(Box::new(body)),
                None,
                Interface::Compiler,
                false,
                false,
            )),
            Span::void(),
            Visibility::Public,
        )
    }

    fn function(name: &'static str, output: &'static str) -> Symbol<'resolver> {
        let target = Element::new(
            ElementKind::literal(Token::new(
                TokenKind::identifier(Str::from(name)),
                Span::void(),
            )),
            Span::void(),
        );

        let output_annotation = Element::new(
            ElementKind::literal(Token::new(
                TokenKind::string(Str::from(output)),
                Span::void(),
            )),
            Span::void(),
        );

        let body = match output {
            "Integer" => Element::new(
                ElementKind::literal(Token::new(TokenKind::integer(0), Span::void())),
                Span::void(),
            ),
            "Float" => Element::new(
                ElementKind::literal(Token::new(TokenKind::integer(0), Span::void())),
                Span::void(),
            ),
            "Boolean" => Element::new(
                ElementKind::literal(Token::new(TokenKind::Boolean(false), Span::void())),
                Span::void(),
            ),
            "Character" => Element::new(
                ElementKind::literal(Token::new(TokenKind::character('a'), Span::void())),
                Span::void(),
            ),
            "String" => Element::new(
                ElementKind::literal(Token::new(TokenKind::string(Str::from("")), Span::void())),
                Span::void(),
            ),
            "Unit" => Element::new(
                ElementKind::Delimited(Box::new(Delimited::new(
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
                ))),
                Span::void(),
            ),
            _ => Element::new(
                ElementKind::Delimited(Box::new(Delimited::new(
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
                ))),
                Span::void(),
            ),
        };

        Symbol::new(
            SymbolKind::function(Function::new(
                Box::new(target),
                Vec::new(),
                Some(Box::new(body)),
                Some(Box::new(output_annotation)),
                Interface::Compiler,
                false,
                false,
            )),
            Span::void(),
            Visibility::Public,
        )
    }
}
