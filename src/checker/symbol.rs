use crate::{
    checker::{Checkable, Checker, Type, TypeKind},
    data::Structure,
    format::Show,
    parser::{Symbol, SymbolKind},
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self, checker: &mut Checker<'_, 'symbol>) {
        let type_value = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding.annotation.as_ref().map(|annotation| {
                    Type::annotation(annotation).unwrap_or_else(|error| {
                        checker.errors.push(error);
                        checker.fresh(self.span)
                    })
                });

                let inferred = binding.value.as_mut().map(|value| {
                    value.check(checker);

                    value.ty.clone()
                });

                match (declared, inferred) {
                    (Some(declared_type), Some(inferred_type)) => checker.unify(self.span, &declared_type, &inferred_type),
                    (Some(declared_type), None) => declared_type,
                    (None, Some(inferred_type)) => inferred_type,
                    (None, None) => checker.fresh(self.span),
                }
            }

            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().unwrap().format(0);

                let members = structure.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.ty.clone()
                }).collect();

                Type::new(TypeKind::Structure(Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Union(union) => {
                let head = union.target.brand().unwrap().format(0);

                let members = union.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.ty.clone()
                }).collect();

                Type::new(TypeKind::Union(Structure::new(head.into(), members)), self.span)
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().unwrap().format(0);

                let members: Vec<_> = function.members.iter_mut().map(|member| {
                    member.check(checker);
                    checker.environment.insert(member.identity, member.ty.clone());
                    member.ty.clone()
                }).collect();

                let output = function.output.as_ref().map(|annotation| {
                    Type::annotation(annotation).unwrap_or_else(|error| {
                        checker.errors.push(error);
                        checker.fresh(self.span)
                    })
                });

                if let Some(body) = &mut function.body {
                    body.check(checker);
                    if let Some(expected) = &output {
                        checker.unify(self.span, expected, &body.ty);
                    }
                }

                let inferred_output = match (&output, &function.body) {
                    (Some(output), _) => Some(Box::new(checker.concretize(output))),
                    (None, Some(body)) => Some(Box::new(checker.concretize(&body.ty))),
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(head.into(), members, inferred_output), self.span)
            }

            SymbolKind::Module(_) => {
                Type::new(TypeKind::Void, self.span)
            }
        };

        self.ty = type_value;
    }
}
