use crate::{
    data::{memory::replace, Aggregate, Function, Interface},
    parser::{Symbol, SymbolKind},
    resolver::{scope::Scope, Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn declare(&mut self, resolver: &mut Resolver<'symbol>) {
        // Phase 1: Pre-declare aggregates and modules to make their types available to members
        let pre_typing = match &self.kind {
            SymbolKind::Structure(structure) => {
                let head = structure.target.target().unwrap();
                Some(Type::new(
                    self.identity,
                    TypeKind::Structure(Box::new(Aggregate::new(head.into(), Vec::new()))),
                ))
            }
            SymbolKind::Union(union) => {
                let head = union.target.target().unwrap();
                Some(Type::new(
                    self.identity,
                    TypeKind::Union(Box::new(Aggregate::new(head.into(), Vec::new()))),
                ))
            }
            SymbolKind::Module(module) => {
                let head = module.target.target().unwrap();
                Some(Type::new(self.identity, TypeKind::Module(head.into())))
            }
            _ => None,
        };

        if let Some(typing) = pre_typing {
            self.typing = typing;
            resolver.insert(self.clone());
        }

        // Phase 2: Declare members and resolve internal scopes
        self.typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                binding.target.declare(resolver);

                if let Some(annotation) = &mut binding.annotation {
                    annotation.resolve(resolver);
                    resolver.annotation(annotation).unwrap_or_else(|_| resolver.fresh())
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

                let mut move_to_body = false;
                let output = if let Some(annotation) = &mut function.output {
                    annotation.resolve(resolver);
                    match resolver.annotation(annotation) {
                        Ok(typing) => typing,
                        Err(_) => {
                            move_to_body = true;
                            resolver.fresh()
                        }
                    }
                } else {
                    resolver.fresh()
                };

                // Adjust AST in case the parser accidentally placed the body block into the output annotation
                if move_to_body && function.body.is_none() {
                    function.body = function.output.take();
                }

                let body = resolver.fresh();

                let active = resolver.active;
                resolver.exit();
                self.scope = Box::from(resolver.scopes.remove(&active).unwrap());
                self.scope.parent = None;

                Type::new(
                    self.identity,
                    TypeKind::Function(Box::new(Function::new(head.into(), members, body, Some(Box::new(output)), Interface::Axo, false, false))),
                )
            }
            SymbolKind::Structure(structure) => {
                resolver.enter();
                for member in &mut structure.members {
                    member.declare(resolver);
                }

                let active = resolver.active;
                resolver.exit();
                self.scope = Box::from(resolver.scopes.remove(&active).unwrap());
                self.scope.parent = None;

                self.typing.clone()
            }
            SymbolKind::Union(union) => {
                resolver.enter();
                for member in &mut union.members {
                    member.declare(resolver);
                }

                let active = resolver.active;
                resolver.exit();
                self.scope = Box::from(resolver.scopes.remove(&active).unwrap());
                self.scope.parent = None;

                self.typing.clone()
            }
            SymbolKind::Module(_) => {
                self.typing.clone()
            }
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
                let scope = replace(&mut self.scope, Box::from(Scope::new(None)));
                resolver.enter_scope(*scope);

                let mut layout = Vec::new();
                for member in &mut structure.members {
                    member.resolve(resolver);
                    if member.is_instance() {
                        layout.push(member.typing.clone());
                    }
                }

                let active = resolver.active;
                resolver.exit();
                self.scope = Box::from(resolver.scopes.remove(&active).unwrap());
                self.scope.parent = None;

                Type::new(
                    self.identity,
                    TypeKind::Structure(Box::new(Aggregate::new(head.into(), layout))),
                )
            }

            SymbolKind::Union(union) => {
                let head = union.target.target().unwrap();
                let scope = replace(&mut self.scope, Box::from(Scope::new(None)));
                resolver.enter_scope(*scope);

                let mut layout = Vec::new();
                for member in &mut union.members {
                    member.resolve(resolver);
                    if member.is_instance() {
                        layout.push(member.typing.clone());
                    }
                }

                let active = resolver.active;
                resolver.exit();
                self.scope = Box::from(resolver.scopes.remove(&active).unwrap());
                self.scope.parent = None;

                Type::new(
                    self.identity,
                    TypeKind::Union(Box::new(Aggregate::new(head.into(), layout))),
                )
            }

            SymbolKind::Function(function) => {
                let head = function.target.target().unwrap();
                let scope = replace(&mut self.scope, Box::from(Scope::new(None)));
                resolver.enter_scope(*scope);

                let members: Vec<_> = function
                    .members
                    .iter_mut()
                    .map(|member| {
                        member.resolve(resolver);
                        member.typing.clone()
                    })
                    .collect();

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

                let body = if let Some(body) = &mut function.body {
                    body.resolve(resolver);
                    resolver.unify(self.span, &expectation, &body.typing);
                    body.typing.clone()
                } else {
                    Type::from(TypeKind::Void)
                };

                resolver.returns.pop();

                let active = resolver.active;
                resolver.exit();
                self.scope = Box::from(resolver.scopes.remove(&active).unwrap());
                self.scope.parent = None;

                let inferred = Some(Box::new(resolver.reify(&expectation)));
                Type::new(
                    self.identity,
                    TypeKind::Function(Box::new(Function::new(head.into(), members, body, inferred, Interface::Axo, false, false))),
                )
            }

            SymbolKind::Module(module) => {
                let head = module.target.target().unwrap();
                Type::new(self.identity, TypeKind::Module(head.into()))
            }
        };

        let unified = resolver.unify(self.span, &expected, &typing);
        self.typing = unified;

        resolver.insert(self.clone());
    }

    fn is_instance(&self) -> bool {
        matches!(self.kind, SymbolKind::Binding(_))
    }
}