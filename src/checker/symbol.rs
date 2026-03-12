use crate::{
    parser::{Symbol, SymbolKind},
    checker::{Checkable, Checker, Type, TypeKind},
};
use crate::data::Structure;
use crate::format::Show;

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self, checker: &mut Checker<'_, 'symbol>) {
        let ty = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding.annotation.as_ref().map(|a| {
                    Type::annotation(a).unwrap_or_else(|e| {
                        checker.errors.push(e);
                        checker.fresh(self.span)
                    })
                });

                let inferred = binding.value.as_mut().map(|v| {
                    v.check(checker);
                    v.ty.clone()
                });

                match (declared, inferred) {
                    (Some(d), Some(i)) => checker.unify(self.span, &d, &i),
                    (Some(d), None) => d,
                    (None, Some(i)) => i,
                    (None, None) => checker.fresh(self.span),
                }
            }

            SymbolKind::Structure(structure) => {
                let head = structure.target.brand().format(0);

                let members = structure.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.ty.clone()
                }).collect();

                let structure = Structure::new(head, members);

                Type::new(TypeKind::Structure(structure), self.span)
            }

            SymbolKind::Union(union) => {
                let head = union.target.brand().format(0);

                let members = union.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.ty.clone()
                }).collect();

                let union = Structure::new(head, members);

                Type::new(TypeKind::Union(union), self.span)
            }

            SymbolKind::Function(function) => {
                let head = function.target.brand().format(0);

                let members: Vec<_> = function.members.iter_mut().map(|member| {
                    member.check(checker);
                    member.ty.clone()
                }).collect();

                let output = function.output.as_ref().map(|output| {
                    Type::annotation(output).unwrap_or_else(|e| {
                        checker.errors.push(e);
                        checker.fresh(self.span)
                    })
                });

                if let Some(body) = &mut function.body {
                    body.check(checker);

                    if let Some(expected) = &output {
                        checker.unify(self.span, expected, &body.ty);
                    }
                }

                Type::new(TypeKind::Function(head, members, output.map(Box::new)), self.span)
            }

            SymbolKind::Module(_) => {
                Type::new(TypeKind::Void, self.span)
            }
        };

        self.ty = ty
    }
}
