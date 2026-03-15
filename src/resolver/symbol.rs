use crate::{
    data::Structure,
    format::Show,
    parser::{Symbol, SymbolKind},
    resolver::{Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn declare(&mut self, resolver: &mut Resolver<'symbol>) {
        let span = self.span;

        self.typing = match &mut self.kind {
            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);
                let parameters = function.members.iter().map(|_| resolver.fresh(span)).collect();
                let output = resolver.fresh(span);

                resolver.enter();
                for member in &mut function.members {
                    member.declare(resolver);
                }
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;
                resolver.exit();

                Type::new(TypeKind::Function(head.into(), parameters, Some(Box::new(output))), span)
            }
            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(0);

                resolver.enter();
                for member in &mut structure.members {
                    member.declare(resolver);
                }
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;
                resolver.exit();

                Type::new(TypeKind::Constructor(self.identity, Structure::new(head.into(), Vec::new())), span)
            }
            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);

                resolver.enter();
                for member in &mut union.members {
                    member.declare(resolver);
                }
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;
                resolver.exit();

                Type::new(TypeKind::Constructor(self.identity, Structure::new(head.into(), Vec::new())), span)
            }
            SymbolKind::Enumeration(enumeration) => {
                let head = enumeration.target.brand().unwrap().format(0);

                resolver.enter();
                for member in &mut enumeration.members {
                    member.declare(resolver);
                }
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;
                resolver.exit();

                Type::new(TypeKind::Constructor(self.identity, Structure::new(head.into(), Vec::new())), span)
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

                resolver.enter_scope(self.scope.clone());

                let members = structure.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Constructor(identity, Structure::new(head.into(), members)), span)
            }

            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);

                resolver.enter_scope(self.scope.clone());

                let members = union.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typing.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Constructor(identity, Structure::new(head.into(), members)), span)
            }

            SymbolKind::Enumeration(enumeration) => {
                let head = enumeration.target.brand().unwrap().format(0);

                resolver.enter_scope(self.scope.clone());

                let mut members = Vec::new();

                for member in &mut enumeration.members {
                    member.resolve(resolver);

                    let instance = Type::new(TypeKind::Enumeration(identity, Structure::new(head.clone().into(), Vec::new())), member.span);

                    if let SymbolKind::Binding(_) = member.kind {
                        member.typing = resolver.unify(member.span, &member.typing, &instance);
                        resolver.insert(member.clone());
                    } else if let SymbolKind::Structure(_) | SymbolKind::Union(_) | SymbolKind::Enumeration(_) = member.kind {
                        if let TypeKind::Constructor(_, structure) = &member.typing.kind {
                            member.typing = Type::new(TypeKind::Constructor(identity, structure.clone()), member.span);
                            resolver.insert(member.clone());
                        }
                    }

                    members.push(member.typing.clone());
                }

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Constructor(identity, Structure::new(head.into(), members)), span)
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);

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
                            resolver.fresh(span)
                        }
                    }
                });

                let expectation = output.clone().unwrap_or_else(|| resolver.fresh(span));
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
                let layout = structure.members.iter().map(|m| m.typing.clone()).collect();
                let head = structure.target.brand().unwrap().format(0).into();
                self.typing = Type::new(TypeKind::Constructor(self.identity, Structure::new(head, layout)), self.span);
            }
            SymbolKind::Union(union) => {
                for member in &mut union.members {
                    member.reify(resolver);
                }
                let layout = union.members.iter().map(|m| m.typing.clone()).collect();
                let head = union.target.brand().unwrap().format(0).into();
                self.typing = Type::new(TypeKind::Constructor(self.identity, Structure::new(head, layout)), self.span);
            }
            SymbolKind::Enumeration(enumeration) => {
                for member in &mut enumeration.members {
                    member.reify(resolver);
                }
                let layout = enumeration.members.iter().map(|m| m.typing.clone()).collect();
                let head = enumeration.target.brand().unwrap().format(0).into();
                self.typing = Type::new(TypeKind::Constructor(self.identity, Structure::new(head, layout)), self.span);
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
