use crate::{
    data::*,
    analyzer::{Analysis, AnalyzeError, ErrorKind},
    format::Show,
    parser::{Element, Symbol, SymbolKind},
    resolver::Resolver,
};
use crate::analyzer::AnalysisKind;
use crate::checker::{Type};

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
                    .brand()
                    .ok_or_else(|| AnalyzeError::new(ErrorKind::Unimplemented, binding.target.span))?;

                let annotation = binding
                    .annotation
                    .as_ref()
                    .map(|annotation| Type::annotation(&*annotation).unwrap());

                let analyzed = Binding::new(
                    Str::from(head.format(0)),
                    value.map(Box::new),
                    annotation,
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
            SymbolKind::Union(union) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = union
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let analyzed = Structure::new(
                    Str::from(union.target.brand().unwrap().format(0)),
                    members?,
                );

                AnalysisKind::Union(analyzed)
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
                    Analysis::new(AnalysisKind::Block(Vec::new()), self.span, Type::unit(self.span))
                };

                let output = if let Some(output) = &function.output {
                    Type::annotation(&*output).ok()
                } else {
                    None
                };

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

        Ok(Analysis::new(kind, self.span, self.ty.clone()))
    }
}
