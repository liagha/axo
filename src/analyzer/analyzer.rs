use crate::{
    analyzer::{Analysis, AnalysisKind, AnalyzeError},
    data::{Aggregate, Binding, Function, Identity, Str},
    internal::{Artifact, RecordKind, Session, SessionError},
    parser::{Element, Symbol, SymbolKind},
    resolver::Resolver,
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
                Ok(analysis) => self.output.push(analysis),
                Err(error) => self.errors.push(error),
            }
        }
    }

    pub fn execute(session: &mut Session<'analyzer>, keys: &[Identity]) {
        for &key in keys {
            Self::process(session, key);
        }
    }

    fn process(session: &mut Session<'analyzer>, key: Identity) {
        let (kind, elements) = {
            let record = session.records.get(&key).unwrap();
            let elements = if let Some(Artifact::Elements(elements)) = record.fetch(2) {
                Some(elements.clone())
            } else {
                None
            };
            (record.kind.clone(), elements)
        };

        if kind != RecordKind::Source {
            return;
        }

        let mut analyzer = Analyzer::new(elements.unwrap_or_default());
        analyzer.analyze(&mut session.resolver);

        analyzer.output.shrink_to_fit();

        session.errors.extend(
            analyzer
                .errors
                .iter()
                .map(|error| SessionError::Analyze(error.clone())),
        );

        let record = session.records.get_mut(&key).unwrap();
        record.artifacts.insert(3, Artifact::Analyses(analyzer.output));
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

                let head = binding.target.analyze(resolver)?;

                let binding = Binding::new(
                    Box::new(head),
                    value.map(Box::new),
                    self.typing.clone(),
                    binding.kind,
                );

                AnalysisKind::Binding(binding)
            }
            SymbolKind::Structure(structure) => {
                let members = structure
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect::<Result<Vec<_>, _>>()?;

                AnalysisKind::Structure(Aggregate::new(
                    Str::from(structure.target.target().unwrap_or_default().to_string()),
                    members,
                ))
            }
            SymbolKind::Union(union) => {
                let members = union
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect::<Result<Vec<_>, _>>()?;

                AnalysisKind::Union(Aggregate::new(
                    Str::from(union.target.target().unwrap_or_default().to_string()),
                    members,
                ))
            }
            SymbolKind::Function(function) => {
                let members = function
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect::<Result<Vec<_>, _>>()?;

                let body = function
                    .body
                    .clone()
                    .map(|body| body.analyze(resolver))
                    .transpose()?
                    .map(Box::new);

                let output = function.output.clone().map(|output| output.typing);
                let function = Function::new(
                    Str::from(function.target.target().unwrap_or_default().to_string()),
                    members,
                    body,
                    output,
                    function.interface,
                    function.entry,
                    function.variadic,
                );

                AnalysisKind::Function(function)
            }
            SymbolKind::Module(module) => {
                AnalysisKind::Module(module.target.target().unwrap_or_default(), Vec::new())
            }
        };

        Ok(Analysis::new(kind, self.span, self.typing.clone()))
    }
}