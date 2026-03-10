use crate::{
    data::*,
    analyzer::{
        Analysis, AnalyzeError, ErrorKind,
    },
    format::Show,
    parser::{Element, Symbol, SymbolKind},
    resolver::{
        Resolver,
    },
    scanner::{Token, TokenKind},
};
use crate::analyzer::AnalysisKind;

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
        let kind = match &self.kind {
            TokenKind::Float(float) => {
                AnalysisKind::Float {
                    value: *float,
                    size: 64,
                }
            },

            TokenKind::Integer(integer) => {
                AnalysisKind::Integer {
                    value: *integer,
                    size: 64,
                    signed: true,
                }
            },

            TokenKind::Boolean(boolean) => {
                AnalysisKind::Boolean {
                    value: *boolean,
                }
            },

            TokenKind::String(string) => {
                AnalysisKind::String {
                    value: *string,
                }
            },

            TokenKind::Character(character) => {
                AnalysisKind::Character {
                    value: *character,
                }
            },

            TokenKind::Identifier(identifier) => {
                AnalysisKind::Usage(*identifier)
            }

            TokenKind::Operator(_) | TokenKind::Punctuation(_) | TokenKind::Comment(_) => {
                AnalysisKind::Tuple(Vec::new())
            }
        };
        Ok(Analysis::new(kind, self.span))
    }
}

impl<'symbol> Analyzable<'symbol> for Symbol<'symbol> {
    fn analyze(
        &self,
        resolver: &mut Resolver<'symbol>,
    ) -> Result<Analysis<'symbol>, AnalyzeError<'symbol>> {
        let kind = match &self.kind {
            SymbolKind::Binding(binding) => {
                let value = binding
                    .value
                    .clone()
                    .map(|value| value.analyze(resolver))
                    .transpose()?;

                let target_token = binding
                    .target
                    .brand()
                    .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, binding.target.span))?;

                let analyzed = Binding::new(
                    Str::from(target_token.format(0)),
                    value.map(Box::new),
                    None,
                    binding.kind,
                );

                AnalysisKind::Binding(analyzed)
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

                AnalysisKind::Structure(analyzed)
            }
            SymbolKind::Function(function) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = function
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let body = if let Some(body) = function.body.as_ref() {
                    body.analyze(resolver)?
                } else {
                    Analysis::unit(self.span)
                };

                let output = Some(self.ty.clone());

                let analyzed = Function::new(
                    Str::from(function.target.brand().unwrap().format(0)),
                    members?,
                    Box::new(body),
                    output,
                    function.interface,
                    function.entry,
                );

                AnalysisKind::Function(analyzed)
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

                AnalysisKind::Module(
                    Str::from(target.format(0)),
                    members?,
                )
            }
        };
        Ok(Analysis::new(kind, self.span))
    }
}