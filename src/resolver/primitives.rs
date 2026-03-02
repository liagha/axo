use {
    super::scope::Scope,
    crate::{
        data::Str,
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::Span,
    },
};
use crate::data::schema::{Binding, Delimited, Method, Module};

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
        "stdin" => Some(stdin(scope)),
        "print" => Some(print()),
        "print_raw" => Some(print_raw()),
        "eprint" => Some(eprint()),
        "eprint_raw" => Some(eprint_raw()),
        "read_line" => Some(read_line()),
        "len" => Some(len()),
        "write" => Some(write()),
        "alloc" => Some(alloc()),
        "free" => Some(free()),
        "is_some" => Some(is_some()),
        "is_none" => Some(is_none()),
        "Some" => Some(some()),
        "None" => Some(none()),
        "unwrap" => Some(unwrap()),
        "or_else" => Some(or_else()),
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
        SymbolKind::Module(Module::new(Box::new(identifier))),
        Span::void(),
        0,
    )
    .with_scope(scope.root().clone())
}

fn stdin<'resolver>(scope: &Scope<'resolver>) -> Symbol<'resolver> {
    let identifier = Element::new(
        ElementKind::Literal(Token::new(
            TokenKind::Identifier(Str::from("stdin")),
            Span::void(),
        )),
        Span::void(),
    );

    Symbol::new(
        SymbolKind::Module(Module::new(Box::new(identifier))),
        Span::void(),
        0,
    )
    .with_scope(scope.root().clone())
}

fn print<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("print", &[("value", "Infer")], "Unit")
}

fn print_raw<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("print_raw", &[("value", "String")], "Unit")
}

fn eprint<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("eprint", &[("value", "Infer")], "Unit")
}

fn eprint_raw<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("eprint_raw", &[("value", "String")], "Unit")
}

fn len<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("len", &[("value", "String")], "Integer")
}

fn write<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params(
        "write",
        &[("fd", "Integer"), ("value", "String")],
        "Integer",
    )
}

fn alloc<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("alloc", &[("size", "Integer")], "Infer")
}

fn free<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("free", &[("ptr", "Infer"), ("size", "Integer")], "Unit")
}

fn read_line<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("read_line", &[], "String")
}

fn is_some<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("is_some", &[("option", "Option")], "Boolean")
}

fn is_none<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("is_none", &[("option", "Option")], "Boolean")
}

fn unwrap<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("unwrap", &[("option", "Option")], "Infer")
}

fn some<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("Some", &[("type", "Infer"), ("value", "Infer")], "Option")
}

fn none<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params("None", &[("type", "Infer"), ("fallback", "Infer")], "Option")
}

fn or_else<'resolver>() -> Symbol<'resolver> {
    typed_method_with_params(
        "or_else",
        &[("option", "Option"), ("fallback", "Infer")],
        "Infer",
    )
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
        SymbolKind::Binding(Binding::new(
            Box::new(parameter_target),
            None::<Box<Element<'resolver>>>,
            Some(Box::new(parameter_annotation)),
            false,
        )),
        Span::void(),
        0,
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
        SymbolKind::Method(Method::new(
            Box::new(target),
            members,
            Box::new(body),
            Some(Box::new(output_annotation)),
            false,
        )),
        Span::void(),
        0,
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
        SymbolKind::Method(Method::new(
            Box::new(target),
            Vec::new(),
            Box::new(body),
            None,
            true,
        )),
        Span::void(),
        0,
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
        SymbolKind::Method(Method::new(
            Box::new(target),
            Vec::new(),
            Box::new(body),
            Some(Box::new(output_annotation)),
            true,
        )),
        Span::void(),
        0,
    )
}
