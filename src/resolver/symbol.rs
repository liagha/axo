use crate::{
    data::Aggregate,
    format::{Show, Verbosity},
    parser::{ElementKind, Symbol, SymbolKind},
    resolver::{Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn declare(&mut self, resolver: &mut Resolver<'symbol>) {
        self.typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                binding.target.declare(resolver);
                resolver.fresh()
            }
            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(Verbosity::Minimal);
                let members = function.members.iter().map(|_| resolver.fresh()).collect();
                let output = resolver.fresh();

                resolver.enter();

                for member in &mut function.members {
                    member.declare(resolver);
                }

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Function(head.into(), members, Some(Box::new(output))))
            }
            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter();

                for member in &mut structure.members {
                    member.declare(resolver);
                }

                let mut scope = resolver.scope.clone();
                
                scope.parent = None;
                self.scope = scope;
                
                resolver.exit();

                Type::new(TypeKind::Constructor(self.identity, Aggregate::new(head.into(), Vec::new())))
            }
            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter();
                
                for member in &mut union.members {
                    member.declare(resolver);
                }
                
                let mut scope = resolver.scope.clone();
                
                scope.parent = None;
                self.scope = scope;
                
                resolver.exit();

                Type::new(TypeKind::Constructor(self.identity, Aggregate::new(head.into(), Vec::new())))
            }
            SymbolKind::Enumeration(enumeration) => {
                let head = enumeration.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter();
                
                for member in &mut enumeration.members {
                    member.declare(resolver);
                }
                
                let mut scope = resolver.scope.clone();
                
                scope.parent = None;
                self.scope = scope;
                
                resolver.exit();

                Type::new(TypeKind::Constructor(self.identity, Aggregate::new(head.into(), Vec::new())))
            }
            SymbolKind::Module(_) => unimplemented!("module declaration not implemented!"),
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
                            resolver.fresh()
                        }
                    }
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.resolve(resolver);
                    value.typing.clone()
                });

                let base = match (declared, inferred) {
                    (Some(source), Some(target)) => resolver.unify(span, &source, &target),
                    (Some(source), None) => source,
                    (None, Some(target)) => target,
                    (None, None) => resolver.fresh(),
                };

                match &mut binding.target.kind {
                    ElementKind::Construct(construct) => {
                        construct.target.resolve(resolver);

                        let instance = if let TypeKind::Constructor(identity, aggregate) = &construct.target.typing.kind {
                            if let Some(symbol) = resolver.scope.find(*identity) {
                                let kind = match &symbol.kind {
                                    SymbolKind::Structure(_) => Some(TypeKind::Structure(*identity, aggregate.clone())),
                                    SymbolKind::Union(_) => Some(TypeKind::Union(*identity, aggregate.clone())),
                                    SymbolKind::Enumeration(_) => Some(TypeKind::Enumeration(*identity, aggregate.clone())),
                                    _ => None,
                                };

                                if let Some(valid) = kind {
                                    Type::new(valid)
                                } else {
                                    resolver.fresh()
                                }
                            } else {
                                resolver.fresh()
                            }
                        } else {
                            resolver.fresh()
                        };

                        resolver.unify(span, &base, &instance);

                        let mut layout = Vec::new();
                        let mut scope = None;

                        if let Some(reference) = construct.target.reference {
                            if let Some(symbol) = resolver.scope.find(reference) {
                                match &symbol.kind {
                                    SymbolKind::Structure(structure) => {
                                        scope = Some(symbol.scope.clone());
                                        for member in &structure.members {
                                            if member.is_instance() {
                                                layout.push(member.typing.clone());
                                            }
                                        }
                                    }
                                    SymbolKind::Union(union) => {
                                        scope = Some(symbol.scope.clone());
                                        for member in &union.members {
                                            if member.is_instance() {
                                                layout.push(member.typing.clone());
                                            }
                                        }
                                    }
                                    SymbolKind::Enumeration(enumeration) => {
                                        scope = Some(symbol.scope.clone());
                                        for member in &enumeration.members {
                                            if member.is_instance() {
                                                layout.push(member.typing.clone());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if let Some(environment) = scope {
                            resolver.enter_scope(environment);
                        } else {
                            resolver.enter();
                        }

                        for (index, member) in construct.members.iter_mut().enumerate() {
                            member.resolve(resolver);
                            if let Some(expected) = layout.get(index) {
                                resolver.unify(member.span, &member.typing, expected);
                            }
                        }

                        resolver.exit();

                        Type::new(TypeKind::Boolean)
                    }
                    _ => {
                        binding.target.declare(resolver);
                        resolver.unify(span, &binding.target.typing, &base);
                        base
                    }
                }
            }

            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter_scope(self.scope.clone());

                let members = structure.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Constructor(identity, Aggregate::new(head.into(), members)))
            }

            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter_scope(self.scope.clone());

                let members = union.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Constructor(identity, Aggregate::new(head.into(), members)))
            }

            SymbolKind::Enumeration(enumeration) => {
                let head = enumeration.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter_scope(self.scope.clone());

                let mut members = Vec::new();

                for member in &mut enumeration.members {
                    member.resolve(resolver);

                    let instance = Type::new(TypeKind::Enumeration(identity, Aggregate::new(head.clone().into(), Vec::new())));

                    if let SymbolKind::Binding(_) = member.kind {
                        member.typing = resolver.unify(member.span, &member.typing, &instance);
                        resolver.insert(member.clone());
                    } else if matches!(member.kind, SymbolKind::Structure(_) | SymbolKind::Union(_) | SymbolKind::Enumeration(_)) {
                        if let TypeKind::Constructor(original, structure) = &member.typing.kind {
                            member.typing = Type::new(TypeKind::Constructor(*original, structure.clone()));
                            resolver.insert(member.clone());
                        }
                    }

                    members.push(member.typing.clone());
                }

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Constructor(identity, Aggregate::new(head.into(), members)))
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(Verbosity::Minimal);

                resolver.enter_scope(self.scope.clone());

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
                    resolver.unify(span, &expectation, &body.typing);
                }

                resolver.returns.pop();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                let inferred = Some(Box::new(resolver.reify(&expectation)));

                Type::new(TypeKind::Function(head.into(), members, inferred))
            }

            SymbolKind::Module(_) => Type::new(TypeKind::Void),
        };

        let unified = resolver.unify(span, &expected, &typing);
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
                let head = structure.target.brand().unwrap().format(Verbosity::Minimal).into();
                self.typing = Type::new(TypeKind::Constructor(self.identity, Aggregate::new(head, layout)));
            }
            SymbolKind::Union(union) => {
                for member in &mut union.members {
                    member.reify(resolver);
                }
                let layout = union.members.iter().map(|member| member.typing.clone()).collect();
                let head = union.target.brand().unwrap().format(Verbosity::Minimal).into();
                self.typing = Type::new(TypeKind::Constructor(self.identity, Aggregate::new(head, layout)));
            }
            SymbolKind::Enumeration(enumeration) => {
                for member in &mut enumeration.members {
                    member.reify(resolver);
                }
                let layout = enumeration.members.iter().map(|member| member.typing.clone()).collect();
                let head = enumeration.target.brand().unwrap().format(Verbosity::Minimal).into();
                self.typing = Type::new(TypeKind::Constructor(self.identity, Aggregate::new(head, layout)));
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
