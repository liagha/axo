use {
    super::scope::Scope,
    crate::{
        data::Str,
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::Span,
    },
};
use crate::data::{Binding, Delimited, Interface, Method, Module};
use crate::parser::Visibility;

pub fn builtin<'resolver>(
    target: &Element<'resolver>,
    scope: &Scope<'resolver>,
) -> Option<Symbol<'resolver>> {
    let name = target.brand().and_then(|token| match token.kind {
        TokenKind::Identifier(identifier) => identifier.as_str().map(|name| name.to_string()),
        _ => None,
    })?;

    match name.as_str() {
        "compiler" => Some(compiler(scope)),
        "Int32" => Some(typed_method("Int32", "Integer")),
        "Int64" => Some(typed_method("Int64", "Integer")),
        "Float" => Some(typed_method("Float", "Float")),
        "Boolean" => Some(typed_method("Boolean", "Boolean")),
        "String" => Some(typed_method("String", "String")),
        "Character" => Some(typed_method("Character", "Character")),
        "Char" => Some(typed_method("Char", "Character")),
        "Unit" => Some(typed_method("Unit", "Unit")),
        "Integer" => Some(typed_method("Integer", "Integer")),
        "if" => Some(method("if")),
        "while" => Some(method("while")),
        "for" => Some(method("for")),
        "break" => Some(method("break")),
        "continue" => Some(method("continue")),
        "return" => Some(method("return")),
        _ => None,
    }
}

fn compiler<'resolver>(scope: &Scope<'resolver>) -> Symbol<'resolver> {
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

fn parameter<'resolver>(name: &'static str, annotation: &'static str) -> Symbol<'resolver> {
    let parameter_target = Element::new(
        ElementKind::Literal(Token::new(
            TokenKind::Identifier(Str::from(name)),
            Span::void(),
        )),
        Span::void(),
    );

    let parameter_annotation = Element::new(
        ElementKind::Literal(Token::new(
            TokenKind::Identifier(Str::from(annotation)),
            Span::void(),
        )),
        Span::void(),
    );

    Symbol::new(
        0,
        SymbolKind::Binding(Binding::new(
            Box::new(parameter_target),
            None::<Box<Element<'resolver>>>,
            Some(Box::new(parameter_annotation)),
            false,
        )),
        Span::void(),
        Visibility::Public,
    )
}

fn typed_method_with_params<'resolver>(
    name: &'static str,
    params: &[(&'static str, &'static str)],
    output: &'static str,
) -> Symbol<'resolver> {
    let target = Element::new(
        ElementKind::Literal(Token::new(
            TokenKind::Identifier(Str::from(name)),
            Span::void(),
        )),
        Span::void(),
    );

    let members = params
        .iter()
        .map(|(param, annotation)| parameter(param, annotation))
        .collect::<Vec<_>>();

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
            members,
            Box::new(body),
            Some(Box::new(output_annotation)),
            Interface::Compiler,
            false,
            false,
        )),
        Span::void(),
        Visibility::Public,
    )
}

fn method<'resolver>(name: &'static str) -> Symbol<'resolver> {
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

fn typed_method<'resolver>(name: &'static str, output: &'static str) -> Symbol<'resolver> {
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
