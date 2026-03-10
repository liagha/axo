use crate::{
    parser::{Symbol, SymbolKind},
    checker::{CheckError, Checkable, ErrorKind, Type, TypeKind},
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self, errors: &mut Vec<CheckError<'symbol>>) {
        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let mut failed = false;

                let annotation = binding.annotation.as_ref().and_then(|value| {
                    match Type::annotation(&*value) {
                        Ok(ty) => Some(ty),
                        Err(error) => {
                            errors.push(error);
                            failed = true;
                            None
                        }
                    }
                });

                let inferred = match &mut binding.value {
                    Some(value) => {
                        value.check(errors);
                        if value.ty.kind == TypeKind::Unknown {
                            failed = true;
                            None
                        } else {
                            Some(value.ty.clone())
                        }
                    }
                    None => None,
                };

                if let (Some(declared), Some(inferred)) = (annotation, inferred) {
                    if Type::unify(&declared, &inferred).is_none() {
                        errors.push(CheckError::new(
                            ErrorKind::Mismatch(declared, inferred.clone()),
                            inferred.span,
                        ));
                        failed = true;
                    }
                }

                if failed { return; }
            }

            SymbolKind::Structure(structure) => {
                for member in structure.members.iter_mut() {
                    member.check(errors);
                }
            }

            SymbolKind::Function(function) => {
                let mut failed = false;

                for member in function.members.iter_mut() {
                    member.check(errors);
                }

                function.body.check(errors);
                if function.body.ty.kind == TypeKind::Unknown {
                    failed = true;
                }

                if failed { return; }

                let output = function.output.as_ref().and_then(|value| {
                    match Type::annotation(&*value) {
                        Ok(ty) => Some(ty),
                        Err(error) => {
                            errors.push(error);
                            None
                        }
                    }
                });

                if let Some(expected) = output {
                    if Type::unify(&expected, &function.body.ty).is_none() {
                        errors.push(CheckError::new(
                            ErrorKind::Mismatch(expected, function.body.ty.clone()),
                            self.span,
                        ));
                    }
                }
            }

            SymbolKind::Module(_) => {}
        }
    }
}