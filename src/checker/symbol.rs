use crate::{
    checker::{Checkable, Checker, Type, TypeKind},
    data::Structure,
    format::Show,
    parser::{Symbol, SymbolKind},
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self, checker: &mut Checker<'_, 'symbol>) {
        let typ = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding.annotation.as_mut().map(|annotation| {
                    match Type::annotation(checker, annotation) {
                        Ok(typ) => typ,
                        Err(error) => {
                            checker.errors.push(error);
                            checker.fresh(self.span)
                        }
                    }
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.check(checker);
                    value.typ.clone()
                });

                match (declared, inferred) {
                    (Some(source), Some(target)) => checker.unify(self.span, &source, &target),
                    (Some(source), None) => source,
                    (None, Some(target)) => target,
                    (None, None) => checker.fresh(self.span),
                }
            }

            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(0);
                let members = structure.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.typ.clone()
                }).collect();

                Type::new(TypeKind::Structure(Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);
                let members = union.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.typ.clone()
                }).collect();

                Type::new(TypeKind::Union(Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);
                let scope = checker.environment.clone();

                let members = function.members.iter_mut().map(|member| {
                    member.check(checker);
                    checker.environment.insert(member.identity, member.typ.clone());
                    member.typ.clone()
                }).collect();

                let output = function.output.as_mut().map(|annotation| {
                    match Type::annotation(checker, annotation) {
                        Ok(typ) => typ,
                        Err(error) => {
                            checker.errors.push(error);
                            checker.fresh(self.span)
                        }
                    }
                });

                if let Some(body) = &mut function.body {
                    body.check(checker);
                    if let Some(expected) = &output {
                        checker.unify(self.span, expected, &body.typ);
                    }
                }

                checker.environment = scope;

                let inferred = match (&output, &function.body) {
                    (Some(expected), _) => Some(Box::new(checker.reify(expected))),
                    (None, Some(body)) => Some(Box::new(checker.reify(&body.typ))),
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(head.into(), members, inferred), self.span)
            }

            SymbolKind::Module(_) => Type::new(TypeKind::Void, self.span),
        };

        self.typ = typ;
    }

    fn reify(&mut self, checker: &mut Checker<'_, 'symbol>) {
        self.typ = checker.reify(&self.typ);

        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                if let Some(value) = &mut binding.value {
                    value.reify(checker);
                }
            }
            SymbolKind::Structure(structure) | SymbolKind::Union(structure) => {
                for member in &mut structure.members {
                    member.reify(checker);
                }
            }
            SymbolKind::Function(function) => {
                for member in &mut function.members {
                    member.reify(checker);
                }
                if let Some(body) = &mut function.body {
                    body.reify(checker);
                }
            }
            SymbolKind::Module(_) => {}
        }
    }
}
