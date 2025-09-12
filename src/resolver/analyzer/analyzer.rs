use {
    crate::{
        resolver::{
            Resolver,
            analyzer::{Analysis, AnalyzeError, ErrorKind, Instruction}
        },
        parser::{
            Symbol, SymbolKind
        }, 
        scanner::{
            Token, TokenKind,
        }, 
        data::{Str},
        schema::*,
    },
};

pub trait Analyzable<'analyzable> {
    fn analyze(&self, resolver: &Resolver<'analyzable>) -> Result<Analysis<'analyzable>, AnalyzeError<'analyzable>>;
}

impl<'token> Analyzable<'token> for Token<'token> {
    fn analyze(&self, _resolver: &Resolver<'token>) -> Result<Analysis<'token>, AnalyzeError<'token>> {
        match &self.kind {
            TokenKind::Float(float) => Ok(Analysis::new(Instruction::Float { value: float.clone(), size: 64 })),
            TokenKind::Integer(integer) => Ok(Analysis::new(Instruction::Integer { value: integer.clone(), size: 64, signed: true })),
            TokenKind::Boolean(boolean) => Ok(Analysis::new(Instruction::Boolean { value: boolean.clone() })),
            TokenKind::Identifier(identifier) => {
                Ok(Analysis::new(Instruction::Usage(identifier.clone())))
            }
            TokenKind::String(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span)),
            TokenKind::Character(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
            }
            TokenKind::Operator(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
            }
            TokenKind::Punctuation(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
            }
            TokenKind::Comment(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span)),
        }
    }
}

impl<'symbol> Analyzable<'symbol> for Symbol<'symbol> {
    fn analyze(&self, resolver: &Resolver<'symbol>) -> Result<Analysis<'symbol>, AnalyzeError<'symbol>> {
        match &self.kind {
            SymbolKind::Inclusion(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span)),
            SymbolKind::Extension(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span)),
            SymbolKind::Binding(binding) => {
                let value = binding
                    .value
                    .clone()
                    .map(|value| value.analyze(resolver))
                    .transpose()?;

                let annotation = binding
                    .annotation
                    .clone()
                    .map(|value| value.analyze(resolver))
                    .transpose()?;

                let analyzed = Binding::new(
                    Str::from(binding.target.brand().unwrap().to_string()),
                    value.map(Box::new),
                    annotation.map(Box::new),
                    binding.constant,
                );
                Ok(Analysis::new(Instruction::Binding(analyzed)))
            }
            SymbolKind::Structure(structure) => {
                let members: Result<Vec<Box<Analysis<'symbol>>>, AnalyzeError<'symbol>> =
                    structure
                        .members
                        .iter()
                        .map(|member| member.analyze(resolver).map(Box::new))
                        .collect();
                
                let analyzed = Structure::new(
                    Str::from(structure.target.brand().unwrap().to_string()),
                    members?,
                );
                
                Ok(Analysis::new(Instruction::Structure(analyzed)))
            }
            SymbolKind::Enumeration(enumeration) => {
                let members: Result<Vec<Box<Analysis<'symbol>>>, AnalyzeError<'symbol>> =
                    enumeration
                        .members
                        .iter()
                        .map(|member| member.analyze(resolver).map(Box::new))
                        .collect();
                
                let analyzed = Structure::new(
                    Str::from(enumeration.target.brand().unwrap().to_string()),
                    members?,
                );
                
                Ok(Analysis::new(Instruction::Enumeration(analyzed)))
            }
            SymbolKind::Method(method) => {
                let members: Result<Vec<Box<Analysis<'symbol>>>, AnalyzeError<'symbol>> =
                    method
                        .members
                        .iter()
                        .map(|member| member.analyze(resolver).map(Box::new))
                        .collect();
                
                let body = method.body.analyze(resolver)?;
                
                let output = method
                    .output
                    .clone()
                    .map(|output| output.analyze(resolver).map(Box::new))
                    .transpose()?;
                
                let analyzed = Method::new(
                    Str::from(method.target.brand().unwrap().to_string()),
                    members?,
                    Box::new(body),
                    output,
                    method.variadic,
                );
                
                Ok(Analysis::new(Instruction::Method(analyzed)))
            }
            SymbolKind::Module(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span)),
            SymbolKind::Preference(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
            }
        }
    }
}