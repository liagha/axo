use crate::{
    data::Str,
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::{
        scope::Scope,
        Resolver,
    },
    scanner::{Token, TokenKind},
};
use crate::analyzer::{
    element::{analyze, Analyzer},
    Analysis, AnalyzeError, ErrorKind, Instruction,
};
use crate::checker::{Checkable, Type, TypeKind};
use crate::data::schema::*;
use crate::format::Show;

pub trait Analyzable<'analyzable> {
    fn analyze(
        &self,
        resolver: &Resolver<'analyzable>,
    ) -> Result<Analysis<'analyzable>, AnalyzeError<'analyzable>>;
}

fn annotation_type_kind<'symbol>(
    element: &Element<'symbol>,
    resolver: &Resolver<'symbol>,
) -> Option<TypeKind<'symbol>> {
    match &element.kind {
        ElementKind::Literal(Token {
            kind: TokenKind::Identifier(identifier),
            span,
        }) => identifier
            .as_str()
            .and_then(|name| {
                TypeKind::from_name(name).or_else(|| {
                    if name == "Type" {
                        Some(TypeKind::Type(Box::new(Type::new(TypeKind::Infer, *span))))
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                Scope::try_lookup(element, &resolver.scope)
                    .ok()
                    .and_then(|symbol| symbol.infer().ok())
                    .map(|item| item.kind)
            }),
        _ => None,
    }
}

pub(crate) fn symbol<'symbol>(
    item: &Symbol<'symbol>,
    resolver: &Resolver<'symbol>,
    context: Analyzer,
) -> Result<Analysis<'symbol>, AnalyzeError<'symbol>> {
    match &item.kind {
        SymbolKind::Inclusion(_) => Ok(Analysis::unit()),
        SymbolKind::Extension(_) => Ok(Analysis::unit()),
        SymbolKind::Binding(binding) => {
            let value = binding
                .value
                .clone()
                .map(|value| analyze(&value, resolver, context))
                .transpose()?;

            let annotation = binding
                .annotation
                .as_deref()
                .and_then(|value| annotation_type_kind(value, resolver));

            let target_token = binding
                .target
                .brand()
                .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, binding.target.span))?;

            let analyzed = Binding::new(
                Str::from(target_token.format(0)),
                value.map(Box::new),
                annotation,
                binding.constant,
            );

            Ok(Analysis::new(Instruction::Binding(analyzed)))
        }
        SymbolKind::Structure(structure) => {
            let members: Result<Vec<Box<Analysis<'symbol>>>, AnalyzeError<'symbol>> = structure
                .members
                .iter()
                .map(|member| symbol(member, resolver, context).map(Box::new))
                .collect();

            let analyzed = Structure::new(
                Str::from(structure.target.brand().unwrap().format(0)),
                members?,
            );

            Ok(Analysis::new(Instruction::Structure(analyzed)))
        }
        SymbolKind::Enumeration(enumeration) => {
            let members: Result<Vec<Box<Analysis<'symbol>>>, AnalyzeError<'symbol>> = enumeration
                .members
                .iter()
                .map(|member| symbol(member, resolver, context).map(Box::new))
                .collect();

            let analyzed = Structure::new(
                Str::from(enumeration.target.brand().unwrap().format(0)),
                members?,
            );

            Ok(Analysis::new(Instruction::Enumeration(analyzed)))
        }
        SymbolKind::Method(method) => {
            let members: Result<Vec<Box<Analysis<'symbol>>>, AnalyzeError<'symbol>> = method
                .members
                .iter()
                .map(|member| symbol(member, resolver, context).map(Box::new))
                .collect();

            let body = analyze(&method.body, resolver, context.method())?;

            let output = method
                .output
                .clone()
                .map(|output| analyze(&output, resolver, context).map(Box::new))
                .transpose()?;

            let analyzed = Method::new(
                Str::from(method.target.brand().unwrap().format(0)),
                members?,
                Box::new(body),
                output,
                method.variadic,
            );

            Ok(Analysis::new(Instruction::Method(analyzed)))
        }
        SymbolKind::Module(module) => {
            let target = module
                .target
                .brand()
                .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, module.target.span))?;

            let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = item
                .scope
                .all()
                .iter()
                .map(|member| symbol(member, resolver, context))
                .collect();

            Ok(Analysis::new(Instruction::Module(
                Str::from(target.format(0)),
                members?,
            )))
        }
        SymbolKind::Preference(_) => Ok(Analysis::unit()),
    }
}

impl<'token> Analyzable<'token> for Token<'token> {
    fn analyze(
        &self,
        resolver: &Resolver<'token>,
    ) -> Result<Analysis<'token>, AnalyzeError<'token>> {
        match &self.kind {
            TokenKind::Float(float) => Ok(Analysis::new(Instruction::Float {
                value: float.clone(),
                size: 64,
            })),
            TokenKind::Integer(integer) => Ok(Analysis::new(Instruction::Integer {
                value: integer.clone(),
                size: 64,
                signed: true,
            })),
            TokenKind::Boolean(boolean) => Ok(Analysis::new(Instruction::Boolean {
                value: boolean.clone(),
            })),
            TokenKind::String(string) => Ok(Analysis::new(Instruction::String {
                value: string.clone(),
            })),
            TokenKind::Character(character) => Ok(Analysis::new(Instruction::Character {
                value: character.clone(),
            })),
            TokenKind::Identifier(identifier) => {
                Ok(Analysis::new(Instruction::Usage(identifier.clone())))
            }
            TokenKind::Operator(_) => Ok(Analysis::unit()),
            TokenKind::Punctuation(_) => Ok(Analysis::unit()),
            TokenKind::Comment(_) => Ok(Analysis::unit()),
        }
    }
}

impl<'symbol> Analyzable<'symbol> for Symbol<'symbol> {
    fn analyze(
        &self,
        resolver: &Resolver<'symbol>,
    ) -> Result<Analysis<'symbol>, AnalyzeError<'symbol>> {
        symbol(self, resolver, Analyzer::root())
    }
}
