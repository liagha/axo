use crate::{
    data::Structure,
    format::Show,
    parser::{Symbol, SymbolKind},
    resolver::{Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn declare(&mut self, resolver: &mut Resolver<'symbol>) {
        let span = self.span;

        self.typing = match &self.kind {
            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);
                let parameters = function.members.iter().map(|_| resolver.fresh(span)).collect();
                let output = resolver.fresh(span);
                Type::new(TypeKind::Function(head.into(), parameters, Some(Box::new(output))), span)
            }
            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(0);
                Type::new(TypeKind::Structure(self.identity, Structure::new(head.into(), Vec::new())), span)
            }
            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);
                Type::new(TypeKind::Union(self.identity, Structure::new(head.into(), Vec::new())), span)
            }
            _ => resolver.fresh(span),
        };

        resolver.insert(self.clone());
    }

    fn resolve(&mut self, resolver: &mut Resolver<'symbol>) {
        let span = self.span;
        let identity = self.identity;

        let expected = self.typing.clone();

        let typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding.annotation.as_mut().map(|annotation| {
                    annotation.resolve(resolver);
                    match resolver.annotation(annotation) {
                        Ok(typing) => typing,
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh(span)
                        }
                    }
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.resolve(resolver);
                    value.typing.clone()
                });

                match (declared, inferred) {
                    (Some(source), Some(target)) => resolver.unify(span, &source, &target),
                    (Some(source), None) => source,
                    (None, Some(target)) => target,
                    (None, None) => resolver.fresh(span),
                }
            }

            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(0);

                resolver.enter();

                let members = structure.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Structure(identity, Structure::new(head.into(), members)), span)
            }

            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);

                resolver.enter();

                let members = union.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Union(identity, Structure::new(head.into(), members)), span)
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);

                resolver.enter();

                for member in &mut function.members {
                    member.declare(resolver);
                }

                let members: Vec<_> = function.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let output = function.output.as_mut().map(|output| {
                    output.resolve(resolver);
                    match resolver.annotation(output) {
                        Ok(typing) => {
                            output.typing = typing.clone();
                            typing
                        }
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh(span)
                        }
                    }
                });

                if let Some(body) = &mut function.body {
                    body.resolve(resolver);

                    if let Some(expected) = &output {
                        resolver.unify(span, expected, &body.typing);
                    }
                }

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                let inferred = match (&output, &function.body) {
                    (Some(expected), _) => Some(Box::new(resolver.reify(expected))),
                    (None, Some(body)) => Some(Box::new(resolver.reify(&body.typing))),
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(head.into(), members, inferred), span)
            }

            SymbolKind::Module(_) => Type::new(TypeKind::Void, span),
        };

        let unified = resolver.unify(span, &expected, &typing);
        self.typing = unified;

        resolver.insert(self.clone());
    }

    fn reify(&mut self, resolver: &mut Resolver<'symbol>) {
        self.typing = resolver.reify(&self.typing);

        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                if let Some(value) = &mut binding.value {
                    value.reify(resolver);
                }
            }
            SymbolKind::Structure(structure) | SymbolKind::Union(structure) => {
                for member in &mut structure.members {
                    member.reify(resolver);
                }
            }
            SymbolKind::Function(function) => {
                for member in &mut function.members {
                    member.reify(resolver);
                }
                if let Some(body) = &mut function.body {
                    body.reify(resolver);
                }
            }
            SymbolKind::Module(_) => {}
        }
    }
}
