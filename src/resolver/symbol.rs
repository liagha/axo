use crate::{
    data::Structure,
    format::Show,
    parser::{Symbol, SymbolKind},
    resolver::{Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(&mut self, resolver: &mut Resolver<'symbol>) {
        if matches!(self.kind, SymbolKind::Function(_)) {
            let (head, members, output) = if let SymbolKind::Function(function) = &mut self.kind {
                let head = function.target.brand().unwrap().format(0);

                resolver.enter();

                let members: Vec<_> = function.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    let placeholder = member.clone();
                    resolver.add(placeholder);
                    member.typing.clone()
                }).collect();

                let output = function.output.as_mut().map(|output| {
                    output.resolve(resolver);
                    match resolver.annotation(output) {
                        Ok(typing) => {
                            output.typing = typing.clone();
                            typing
                        },
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh(self.span)
                        }
                    }
                });

                (head, members, output)
            } else {
                unreachable!()
            };

            let current = Type::new(TypeKind::Function(head.clone().into(), members.clone(), output.clone().map(Box::new)), self.span);
            self.typing = current;
            resolver.add(self.clone());

            if let SymbolKind::Function(function) = &mut self.kind {
                if let Some(body) = &mut function.body {
                    body.resolve(resolver);
                    if let Some(expected) = &output {
                        resolver.unify(self.span, expected, &body.typing);
                    }
                }
            }

            let mut scope = resolver.scope.clone();
            scope.parent = None;
            self.scope = scope;

            resolver.exit();

            let inferred = if let SymbolKind::Function(function) = &self.kind {
                match (&output, &function.body) {
                    (Some(expected), _) => Some(Box::new(resolver.reify(expected))),
                    (None, Some(body)) => Some(Box::new(resolver.reify(&body.typing))),
                    (None, None) => None,
                }
            } else {
                unreachable!()
            };

            self.typing = Type::new(TypeKind::Function(head.into(), members, inferred), self.span);
            resolver.add(self.clone());
            return;
        }

        let mut placeholder = self.clone();
        match &placeholder.kind {
            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(0);
                placeholder.typing = Type::new(TypeKind::Structure(self.identity, Structure::new(head.into(), Vec::new())), self.span);
                resolver.add(placeholder);
            }
            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);
                placeholder.typing = Type::new(TypeKind::Union(self.identity, Structure::new(head.into(), Vec::new())), self.span);
                resolver.add(placeholder);
            }
            _ => {}
        }

        let typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding.annotation.as_mut().map(|annotation| {
                    annotation.resolve(resolver);
                    match resolver.annotation(annotation) {
                        Ok(typing) => typing,
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh(self.span)
                        }
                    }
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.resolve(resolver);
                    value.typing.clone()
                });

                match (declared, inferred) {
                    (Some(source), Some(target)) => resolver.unify(self.span, &source, &target),
                    (Some(source), None) => source,
                    (None, Some(target)) => target,
                    (None, None) => resolver.fresh(self.span),
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

                Type::new(TypeKind::Structure(self.identity, Structure::new(head.into(), members)), self.span)
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

                Type::new(TypeKind::Union(self.identity, Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Module(_) => Type::new(TypeKind::Void, self.span),

            SymbolKind::Function(_) => unreachable!(),
        };

        self.typing = typing;
        resolver.add(self.clone());
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