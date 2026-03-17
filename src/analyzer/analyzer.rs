use crate::{
    data::*,
    analyzer::{Analysis, AnalyzeError, ErrorKind},
    format::{Show, Verbosity},
    parser::{Element, Symbol, SymbolKind},
    resolver::{Resolver},
    analyzer::AnalysisKind,
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

                let head = binding
                    .target
                    .target()
                    .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, binding.target.span))?;

                let analyzed = Binding::new(
                    Str::from(head.format(Verbosity::Minimal)),
                    value.map(Box::new),
                    self.typing.clone(),
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

                let analyzed = Aggregate::new(
                    Str::from(structure.target.target().unwrap().format(Verbosity::Minimal)),
                    members?,
                );

                AnalysisKind::Structure(analyzed)
            }
            SymbolKind::Union(union) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = union
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let analyzed = Aggregate::new(
                    Str::from(union.target.target().unwrap().format(Verbosity::Minimal)),
                    members?,
                );

                AnalysisKind::Union(analyzed)
            }
            SymbolKind::Enumeration(enumeration) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = enumeration
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let analyzed = Aggregate::new(
                    Str::from(enumeration.target.target().unwrap().format(Verbosity::Minimal)),
                    members?,
                );

                AnalysisKind::Enumeration(analyzed) 
            }
            SymbolKind::Function(function) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = function
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let body = function.body.clone().map(|body| body.analyze(resolver).ok().map(Box::new)).flatten();

                let output = function.output.clone().map(|output| output.typing);

                let analyzed = Function::new(
                    Str::from(function.target.target().unwrap().format(Verbosity::Minimal)),
                    members?,
                    body,
                    output,
                    function.interface,
                    function.entry,
                );

                AnalysisKind::Function(analyzed)
            }
            SymbolKind::Module(module) => {
                let target = module
                    .target
                    .target()
                    .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, module.target.span))?;

                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = self
                    .scope
                    .collect(&resolver.scopes, &resolver.registry)
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                AnalysisKind::Module(
                    Str::from(target.format(Verbosity::Minimal)),
                    members?,
                )
            }
        };

        Ok(Analysis::new(kind, self.span, self.typing.clone()))
    }
}
