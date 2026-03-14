use crate::{
    data::Structure,
    format::Show,
    parser::{Symbol, SymbolKind},
    resolver::{Resolvable, Resolver, Type, TypeKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(&mut self, resolver: &mut Resolver<'symbol>) {
        resolver.add(self.clone());

        let typ = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding.annotation.as_mut().map(|annotation| {
                    annotation.resolve(resolver);
                    match Type::annotation(resolver, annotation) {
                        Ok(typ) => typ,
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh(self.span)
                        }
                    }
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.resolve(resolver);
                    value.typ.clone()
                });

                match (declared, inferred) {
                    (Some(source), Some(target)) => resolver.unify(self.span, &source, &target),
                    (Some(source), None) => source,
                    (None, Some(target)) => target,
                    (None, None) => resolver.fresh(self.span),
                }
            }

            SymbolKind::Structure(structure) => {
                resolver.enter();

                let head = structure.target.brand().unwrap().format(0);
                let members = structure.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typ.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Structure(Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Union(union) => {
                resolver.enter();

                let head = union.target.brand().unwrap().format(0);
                let members = union.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    member.typ.clone()
                }).collect();

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                Type::new(TypeKind::Union(Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);

                resolver.enter();

                let members = function.members.iter_mut().map(|member| {
                    member.resolve(resolver);
                    resolver.bind(member.identity, member.typ.clone());
                    member.typ.clone()
                }).collect();

                let output = function.output.as_mut().map(|output| {
                    output.resolve(resolver);
                    match Type::annotation(resolver, output) {
                        Ok(typ) => {
                            output.typ = typ.clone();
                            typ
                        },
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh(self.span)
                        }
                    }
                });

                if let Some(body) = &mut function.body {
                    body.resolve(resolver);
                    if let Some(expected) = &output {
                        resolver.unify(self.span, expected, &body.typ);
                    }
                }

                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();

                let inferred = match (&output, &function.body) {
                    (Some(expected), _) => Some(Box::new(resolver.reify(expected))),
                    (None, Some(body)) => Some(Box::new(resolver.reify(&body.typ))),
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(head.into(), members, inferred), self.span)
            }

            SymbolKind::Module(_) => Type::new(TypeKind::Void, self.span),
        };

        self.typ = typ;
        resolver.add(self.clone());
    }

    fn reify(&mut self, resolver: &mut Resolver<'symbol>) {
        self.typ = resolver.reify(&self.typ);

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
