use {
    super::{
        scope::Scope,
        Inference, Resolution, Resolvable, ResolveError, Resolver,
    },
    crate::{
        analyzer::Analyzable,
        checker::{
            annotation_type, unify,
            Type,
            CheckError, Checkable,
        },
        data::{Binary, Boolean, Str},
        parser::{ElementKind, Symbol, SymbolKind, Visibility},
        resolver::ErrorKind,
        scanner::{OperatorKind, Token, TokenKind},
        tracker::Span,
    },
};

fn symbol_name<'symbol>(symbol: &Symbol<'symbol>) -> Option<Str<'symbol>> {
    symbol.brand().and_then(|token| match token.kind {
        TokenKind::Identifier(name) => Some(name),
        _ => None,
    })
}

fn import_into_scope<'symbol>(
    resolver: &mut Resolver<'symbol>,
    symbol: Symbol<'symbol>,
) -> Result<(), ResolveError<'symbol>> {
    let Some(name) = symbol_name(&symbol) else {
        resolver.scope.add(symbol);
        return Ok(());
    };

    let conflict = resolver.scope.symbols.iter().find(|candidate| {
        if matches!(candidate.kind, SymbolKind::Inclusion(_)) {
            return false;
        }
        candidate
            .brand()
            .and_then(|token| match token.kind {
                TokenKind::Identifier(candidate_name) => Some(candidate_name == name),
                _ => None,
            })
            .unwrap_or(false)
    });

    if let Some(conflict) = conflict {
        if conflict.kind == symbol.kind {
            return Ok(());
        }
        let token = conflict.brand().unwrap_or(Token::new(
            TokenKind::Identifier(name),
            conflict.span,
        ));
        return Err(ResolveError::new(
            ErrorKind::ImportConflict { symbol: token },
            conflict.span,
        ));
    }

    resolver.scope.add(symbol);
    Ok(())
}

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(
        &self,
        resolver: &mut Resolver<'symbol>,
    ) -> Result<Resolution<'symbol>, Vec<ResolveError<'symbol>>> {
        let mut symbol = self.clone();
        let generic = symbol.generic.clone();

        let id = resolver.next_id();
        symbol.id = id.clone();

        match &mut symbol.kind {
            SymbolKind::Inclusion(inclusion) => {
                let valid_import_path = matches!(
                    inclusion.target.kind,
                    ElementKind::Binary(Binary {
                        operator:
                            Token {
                                kind: TokenKind::Operator(OperatorKind::Dot),
                                ..
                            },
                        ..
                    })
                );

                if !valid_import_path {
                    let token = inclusion.target.brand().unwrap_or(Token::new(
                        TokenKind::Identifier(Str::from("use")),
                        inclusion.target.span,
                    ));
                    return Err(vec![ResolveError::new(
                        ErrorKind::InvalidImportPath { query: token },
                        inclusion.target.span,
                    )]);
                }

                inclusion.target.resolve(resolver)?;
                let mut import_errors = Vec::new();

                if let Ok(found) = inclusion.target.resolve_path(resolver) {
                    if let SymbolKind::Module(_) = found.kind {
                        for member in found.scope.symbols.iter() {
                            if matches!(member.specifier.visibility, Visibility::Private) {
                                continue;
                            }

                            if let Err(error) = import_into_scope(resolver, member.clone()) {
                                import_errors.push(error);
                            }
                        }
                    } else if matches!(found.specifier.visibility, Visibility::Private) {
                        let token = found.brand().unwrap_or(Token::new(
                            TokenKind::Identifier(Str::from("<private>")),
                            found.span,
                        ));
                        import_errors.push(ResolveError::new(
                            ErrorKind::PrivateSymbol { symbol: token },
                            found.span,
                        ));
                    } else if let Err(error) = import_into_scope(resolver, found) {
                        import_errors.push(error);
                    }
                }

                resolver.scope.add(symbol);

                if !import_errors.is_empty() {
                    return Err(import_errors);
                }
            }
            SymbolKind::Preference(_) => {}
            SymbolKind::Extension(extension) => {
                if extension
                    .members
                    .iter()
                    .any(|member| !matches!(member.kind, SymbolKind::Method(_)))
                {
                    return Err(vec![ResolveError::new(
                        ErrorKind::Analyze {
                            error: crate::analyzer::AnalyzeError::new(
                                crate::analyzer::ErrorKind::InvalidOperation(
                                    Token::new(
                                        TokenKind::Identifier(Str::from("extend")),
                                        extension.target.span,
                                    ),
                                ),
                                extension.target.span,
                            ),
                        },
                        extension.target.span,
                    )]);
                }

                let scope = resolver.scope.clone();

                if let Ok(mut target) = Scope::try_lookup(&*extension.target, &scope) {
                    if let Some(extension) = &extension.extension {
                        if let Ok(found) = Scope::try_lookup(&*extension, &scope) {
                            if let SymbolKind::Structure(structure) = found.kind {
                                resolver.scope.remove(&target);
                                target
                                    .scope
                                    .symbols
                                    .extend(structure.members.iter().cloned());
                                target.generic.merge(&generic);
                                resolver.scope.add(target);
                            }
                        }
                    } else {
                        resolver.scope.remove(&target);
                        target
                            .scope
                            .symbols
                            .extend(extension.members.iter().cloned());
                        target.generic.merge(&generic);
                        resolver.scope.add(target);
                    }
                }
            }
            _ => {
                resolver.scope.add(symbol);
            }
        }

        let analysis = self.analyze(resolver).map_err(|error| {
            vec![ResolveError::new(
                ErrorKind::Analyze {
                    error: error.clone(),
                },
                error.span,
            )]
        })?;

        let typ = match &self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding
                    .annotation
                    .as_ref()
                    .and_then(|value| annotation_type(value));
                let inferred = binding
                    .value
                    .as_ref()
                    .map(|value| value.resolve(resolver).map(|resolution| resolution.typed))
                    .transpose()?;

                let target = binding.target.brand().unwrap_or_else(|| {
                    Token::new(
                        TokenKind::Identifier(Str::from("<anonymous>")),
                        Span::void(),
                    )
                });
                let inference = Inference::new(target, declared.clone(), inferred.clone());
                if let Some(index) = resolver
                    .symbols
                    .iter()
                    .position(|(symbol, _)| symbol.brand() == self.brand())
                {
                    resolver.symbols[index] = (self.clone(), Some(inference));
                } else {
                    resolver.symbols.push((self.clone(), Some(inference)));
                }

                match (declared, inferred) {
                    (Some(declared), Some(inferred)) => {
                        if let Some(unified) = unify(&declared, &inferred) {
                            unified
                        } else {
                            return Err(vec![ResolveError::new(
                                ErrorKind::Check {
                                    error: CheckError::new(
                                        crate::checker::ErrorKind::Mismatch(
                                            declared,
                                            inferred.clone(),
                                        ),
                                        inferred.span,
                                    ),
                                },
                                inferred.span,
                            )]);
                        }
                    }
                    (Some(declared), None) => declared,
                    (None, Some(inferred)) => inferred,
                    (None, None) => Type::unit(self.span),
                }
            }
            _ => self.infer().map_err(|error| {
                vec![ResolveError::new(
                    ErrorKind::Check {
                        error: error.clone(),
                    },
                    error.span,
                )]
            })?,
        };

        let resolution = Resolution::new(Some(id), typ, analysis);

        Ok(resolution)
    }

    fn is_instance(&self, resolver: &mut Resolver<'symbol>) -> Boolean {
        match &self.kind {
            SymbolKind::Inclusion(_) => false,
            SymbolKind::Extension(_) => false,
            SymbolKind::Binding(_) => true,
            SymbolKind::Structure(_) => false,
            SymbolKind::Enumeration(_) => true,
            SymbolKind::Method(_) => true,
            SymbolKind::Module(_) => true,
            SymbolKind::Preference(_) => true,
        }
    }
}
