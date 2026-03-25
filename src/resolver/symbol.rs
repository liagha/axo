use crate::{
    data::{memory::replace, Aggregate},
    parser::{Symbol, SymbolKind},
    resolver::{scope::Scope, Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn declare(&mut self, resolver: &mut Resolver<'symbol>) {
        self.typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                binding.target.declare(resolver);

                if let Some(annotation) = &mut binding.annotation {
                    annotation.resolve(resolver);

                    match resolver.annotation(annotation) {
                        Ok(typing) => typing,
                        Err(_) => resolver.fresh(),
                    }
                } else {
                    resolver.fresh()
                }
            }
            SymbolKind::Function(function) => {
                let head = function.target.target().unwrap();

                resolver.enter();
                for member in &mut function.members {
                    member.declare(resolver);
                }

                let members = function.members.iter().map(|m| m.typing.clone()).collect();

                let output = if let Some(annotation) = &mut function.output {
                    annotation.resolve(resolver);
                    resolver.annotation(annotation).unwrap_or_else(|_| resolver.fresh())
                } else {
                    resolver.fresh()
                };

                let active = resolver.active;
                resolver.exit();
                self.scope = resolver.scopes.remove(&active).unwrap();
                self.scope.parent = None;

                Type::new(self.identity, TypeKind::Function(head.into(), members, Some(Box::new(output))))
            }
            SymbolKind::Structure(structure) => {
                let head = structure.target.target().unwrap();

                resolver.enter();
                for member in &mut structure.members {
                    member.declare(resolver);
                }

                let active = resolver.active;
                resolver.exit();
                self.scope = resolver.scopes.remove(&active).unwrap();
                self.scope.parent = None;

                Type::new(self.identity, TypeKind::Structure(Aggregate::new(head.into(), Vec::new())))
            }
            SymbolKind::Union(union) => {
                let head = union.target.target().unwrap();

                resolver.enter();
                for member in &mut union.members {
                    member.declare(resolver);
                }

                let active = resolver.active;
                resolver.exit();
                self.scope = resolver.scopes.remove(&active).unwrap();
                self.scope.parent = None;

                Type::new(self.identity, TypeKind::Union(Aggregate::new(head.into(), Vec::new())))
            }
            SymbolKind::Module(_) => unimplemented!("module declaration not implemented!"),
        };

        resolver.insert(self.clone());
    }

    fn resolve(&mut self, resolver: &mut Resolver<'symbol>) {
        let expected = self.typing.clone();

        let typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let annotation = binding.annotation.as_mut().map(|annotation| {
                    annotation.resolve(resolver);

                    match resolver.annotation(annotation) {
                        Ok(typing) => typing,
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh()
                        }
                    }
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.resolve(resolver);
                    value.typing.clone()
                });

                let typing = match (annotation, inferred) {
                    (Some(source), Some(target)) => resolver.unify(self.span, &source, &target),
                    (Some(source), None) => source,
                    (None, Some(target)) => target,
                    (None, None) => resolver.fresh(),
                };

                resolver.unify(self.span, &binding.target.typing, &typing);
                typing
            }

            SymbolKind::Structure(structure) => {
                let head = structure.target.target().unwrap();
                let scope = replace(&mut self.scope, Scope::new(None));
                resolver.enter_scope(scope);

                let members = structure.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let active = resolver.active;
                resolver.exit();
                self.scope = resolver.scopes.remove(&active).unwrap();
                self.scope.parent = None;

                Type::new(self.identity, TypeKind::Structure(Aggregate::new(head.into(), members)))
            }

            SymbolKind::Union(union) => {
                let head = union.target.target().unwrap();
                let scope = replace(&mut self.scope, Scope::new(None));
                resolver.enter_scope(scope);

                let members = union.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let active = resolver.active;
                resolver.exit();
                self.scope = resolver.scopes.remove(&active).unwrap();
                self.scope.parent = None;

                Type::new(self.identity, TypeKind::Union(Aggregate::new(head.into(), members)))
            }

            SymbolKind::Function(function) => {
                let head = function.target.target().unwrap();
                let scope = replace(&mut self.scope, Scope::new(None));
                resolver.enter_scope(scope);

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
                            resolver.fresh()
                        }
                    }
                });

                let expectation = output.clone().unwrap_or_else(|| resolver.fresh());
                resolver.returns.push(expectation.clone());

                if let Some(body) = &mut function.body {
                    body.resolve(resolver);
                    resolver.unify(self.span, &expectation, &body.typing);
                }

                resolver.returns.pop();

                let active = resolver.active;
                resolver.exit();
                self.scope = resolver.scopes.remove(&active).unwrap();
                self.scope.parent = None;

                let inferred = Some(Box::new(resolver.reify(&expectation)));
                Type::new(self.identity, TypeKind::Function(head.into(), members, inferred))
            }

            SymbolKind::Module(_) => Type::from(TypeKind::Void),
        };

        let unified = resolver.unify(self.span, &expected, &typing);
        self.typing = unified;

        resolver.insert(self.clone());
    }

    fn reify(&mut self, resolver: &mut Resolver<'symbol>) {
        self.typing = resolver.reify(&self.typing);

        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                binding.target.reify(resolver);
                if let Some(annotation) = &mut binding.annotation {
                    annotation.reify(resolver);
                }
                if let Some(value) = &mut binding.value {
                    value.reify(resolver);
                }
            }
            SymbolKind::Structure(structure) => {
                for member in &mut structure.members {
                    member.reify(resolver);
                }
                let layout = structure.members.iter().map(|member| member.typing.clone()).collect();
                let head = structure.target.target().unwrap().into();
                self.typing = Type::new(self.identity, TypeKind::Structure(Aggregate::new(head, layout)));
            }
            SymbolKind::Union(union) => {
                for member in &mut union.members {
                    member.reify(resolver);
                }
                let layout = union.members.iter().map(|member| member.typing.clone()).collect();
                let head = union.target.target().unwrap().into();
                self.typing = Type::new(self.identity, TypeKind::Union(Aggregate::new(head, layout)));
            }
            SymbolKind::Function(function) => {
                for member in &mut function.members {
                    member.reify(resolver);
                }
                if let Some(output) = &mut function.output {
                    output.reify(resolver);
                }
                if let Some(body) = &mut function.body {
                    body.reify(resolver);
                }
            }
            SymbolKind::Module(_) => {}
        }

        resolver.insert(self.clone());
    }

    fn is_instance(&self) -> bool {
        matches!(self.kind, SymbolKind::Binding(_))
    }
}
