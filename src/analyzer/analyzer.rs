use crate::{
    analyzer::{Analysis, AnalysisKind, AnalyzeError},
    data::Str,
    format::{Show, Stencil},
    internal::{Artifact, RecordKind, Session, SessionError},
    parser::{Element, Symbol, SymbolKind},
    resolver::Resolver,
    data::{Identity, Binding, Aggregate, Function},
};
use broccli::Color;

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
        let (kind, hash, dirty, elements) = {
            let record = session.records.get(&key).unwrap();
            let elements = if let Some(Artifact::Elements(elements)) = record.fetch(2) {
                Some(elements.clone())
            } else {
                None
            };
            (record.kind.clone(), record.hash, record.dirty, elements)
        };

        if kind != RecordKind::Source {
            return;
        }

        if !dirty {
            if let Some(mut analyses) = session.cache::<Vec<Analysis>>("analyses", hash, None) {
                analyses.shrink_to_fit();
                let record = session.records.get_mut(&key).unwrap();
                record.store(3, Artifact::Analyses(analyses));
                return;
            }
        }

        let mut analyzer = Analyzer::new(elements.unwrap_or_default());
        analyzer.analyze(&mut session.resolver);

        if let Some(stencil) = session.get_stencil() {
            session.report_section(
                "Analysis",
                Color::Blue,
                analyzer.output.format(stencil).to_string(),
            );
        }

        analyzer.output.shrink_to_fit();

        session.errors.extend(
            analyzer
                .errors
                .iter()
                .map(|error| SessionError::Analyze(error.clone())),
        );

        if let Some(analyses) = session.cache("analyses", hash, Some(analyzer.output)) {
            let record = session.records.get_mut(&key).unwrap();
            record.store(3, Artifact::Analyses(analyses));
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
                    .map(|v| v.analyze(resolver))
                    .transpose()?;

                let head = binding.target.analyze(resolver)?;

                let analyzed = Binding::new(
                    Box::new(head),
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
                    .map(|m| m.analyze(resolver))
                    .collect();

                let name = Str::from(structure.target.target().unwrap().format(Stencil::default()));
                let analyzed = Aggregate::new(name, members?);

                AnalysisKind::Structure(analyzed)
            }
            SymbolKind::Union(union) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = union
                    .members
                    .iter()
                    .map(|m| m.analyze(resolver))
                    .collect();

                let name = Str::from(union.target.target().unwrap().format(Stencil::default()));
                let analyzed = Aggregate::new(name, members?);

                AnalysisKind::Union(analyzed)
            }
            SymbolKind::Function(function) => {
                let members: Result<Vec<Analysis<'symbol>>, AnalyzeError<'symbol>> = function
                    .members
                    .iter()
                    .map(|m| m.analyze(resolver))
                    .collect();

                let body = function
                    .body
                    .clone()
                    .and_then(|b| b.analyze(resolver).ok().map(Box::new));

                let output = function.output.clone().map(|o| o.typing);
                let name = Str::from(function.target.target().unwrap().format(Stencil::default()));

                let analyzed = Function::new(
                    name,
                    members?,
                    body,
                    output,
                    function.interface,
                    function.entry,
                    function.variadic,
                );

                AnalysisKind::Function(analyzed)
            }
            SymbolKind::Module(_) => {
                unimplemented!()
            }
        };

        Ok(Analysis::new(kind, self.span, self.typing.clone()))
    }
}
