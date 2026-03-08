use crate::{
    data::*,
    analyzer::{
        Analysis, AnalyzeError, ErrorKind,
    },
    checker::{Checkable, Type, TypeKind},
    format::Show,
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::{
        Resolver,
    },
    scanner::{Token, TokenKind},
};

pub struct Analyzer<'analyzer> {
    pub input: Vec<Element<'analyzer>>,
    pub output: Vec<Analysis<'analyzer>>,
    pub errors: Vec<AnalyzeError<'analyzer>>,
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn new(input: Vec<Element<'analyzer>>) -> Self {
        Self {
            input,
            output: Vec::new(),
            errors: Vec::new(),
        }    
    }
    
    pub fn analyze(&mut self, resolver: &mut Resolver<'analyzer>) {
        for element in self.input.iter_mut() {
            match element.analyze(resolver) {
                Ok(analysis) => {
                    self.output.push(analysis);
                }
                
                Err(error) => {
                    self.errors.push(error);
                }
            }
        }
    }
}

pub trait Analyzable<'analyzable> {
    fn analyze(
        &self,
        resolver: &mut Resolver<'analyzable>,
    ) -> Result<Analysis<'analyzable>, AnalyzeError<'analyzable>>;
}

impl<'token> Analyzable<'token> for Token<'token> {
    fn analyze(
        &self,
        _resolver: &mut Resolver<'token>,
    ) -> Result<Analysis<'token>, AnalyzeError<'token>> {
        match &self.kind {
            TokenKind::Float(float) => {
                Ok(Analysis::Float {
                    value: float.clone(),
                    size: 64,
                })
            },
            TokenKind::Integer(integer) => {
                Ok(Analysis::Integer {
                    value: integer.clone(),
                    size: 64,
                    signed: true,
                })
            },
            TokenKind::Boolean(boolean) => {
                Ok(Analysis::Boolean {
                    value: boolean.clone(),
                })
            },
            TokenKind::String(string) => {
                Ok(Analysis::String {
                    value: string.clone(),
                })
            },
            TokenKind::Character(character) => {
                Ok(Analysis::Character {
                    value: character.clone(),
                })
            },
            TokenKind::Identifier(identifier) => {
                Ok(Analysis::Usage(identifier.clone()))
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
        resolver: &mut Resolver<'symbol>,
    ) -> Result<Analysis<'symbol>, AnalyzeError<'symbol>> {
        match &self.kind {
            SymbolKind::Binding(binding) => {
                let value = binding
                    .value
                    .clone()
                    .map(|value| value.analyze(resolver))
                    .transpose()?;

                let annotation = binding
                    .annotation
                    .as_deref()
                    .and_then(|value|
                        match &value.kind {
                            ElementKind::Literal(Token {
                                                     kind: TokenKind::Identifier(identifier),
                                                     span,
                                                 }) => identifier
                                .as_str()
                                .and_then(|name| {
                                    TypeKind::from_name(name).or_else(|| {
                                        if name == "Type" {
                                            Some(TypeKind::Type(Box::new(Type::new(TypeKind::Unknown, *span))))
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .or_else(|| {
                                    resolver.scope.lookup(value)
                                        .ok()
                                        .and_then(|symbol| symbol.infer().ok())
                                        .map(|item| item.kind)
                                }),
                            _ => None,
                        }
                    );

                let target_token = binding
                    .target
                    .brand()
                    .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, binding.target.span))?;

                let analyzed = Binding::new(
                    Str::from(target_token.format(0)),
                    value.map(Box::new),
                    annotation,
                    binding.kind,
                );

                Ok(Analysis::Binding(analyzed))
            }
            SymbolKind::Structure(structure) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = structure
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let analyzed = Structure::new(
                    Str::from(structure.target.brand().unwrap().format(0)),
                    members?,
                );

                Ok(Analysis::Structure(analyzed))
            }
            SymbolKind::Enumeration(enumeration) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = enumeration
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let analyzed = Structure::new(
                    Str::from(enumeration.target.brand().unwrap().format(0)),
                    members?,
                );

                Ok(Analysis::Enumeration(analyzed))
            }
            SymbolKind::Method(method) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = method
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let body = method.body.analyze(resolver)?;

                let output = method
                    .output
                    .clone()
                    .map(|output| output.analyze(resolver).map(Box::new))
                    .transpose()?;

                let analyzed = Method::new(
                    Str::from(method.target.brand().unwrap().format(0)),
                    members?,
                    Box::new(body),
                    output,
                    method.interface,
                    method.variadic,
                    method.entry,
                );

                Ok(Analysis::Method(analyzed))
            }
            SymbolKind::Module(module) => {
                let target = module
                    .target
                    .brand()
                    .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, module.target.span))?;

                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = self
                    .scope
                    .all()
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                Ok(Analysis::Module(
                    Str::from(target.format(0)),
                    members?,
                ))
            }
            SymbolKind::Preference(_) => Ok(Analysis::unit()),
        }

    }
}
