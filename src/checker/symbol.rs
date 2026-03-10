use crate::{
    parser::{Symbol, SymbolKind},
    checker::{CheckError, Checkable, ErrorKind, Type, TypeKind},
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self, errors: &mut Vec<CheckError<'symbol>>) {
        let ty = match &mut self.kind {
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

                if let (Some(declared), Some(inferred)) = (annotation.clone(), inferred) {
                    if Type::unify(&declared, &inferred).is_none() {
                        errors.push(CheckError::new(
                            ErrorKind::Mismatch(declared, inferred.clone()),
                            inferred.span,
                        ));
                        failed = true;
                    }
                }

                if failed { return; }
                
                annotation.unwrap_or(Type::new(TypeKind::Void, self.span))
            }

            SymbolKind::Structure(structure) => {
                let _members = structure
                    .members
                    .iter_mut()
                    .map(|member| {
                        member.check(errors);
                        
                        member.ty.clone()
                    });
                
                Type::new(TypeKind::Void, self.span)
            }

            SymbolKind::Function(function) => {
                let mut failed = false;

                for member in function.members.iter_mut() {
                    member.check(errors);
                }

                let output = function.output.as_ref().and_then(|output| {
                    match Type::annotation(&*output) {
                        Ok(ty) => Some(ty),
                        Err(error) => {
                            errors.push(error);
                            None
                        }
                    }
                });

                if let Some(body) = &mut function.body {
                    body.check(errors);
                    
                    if body.ty.kind == TypeKind::Unknown {
                        failed = true;
                    }

                    if failed { return; }

                    if let Some(expected) = output.clone() {
                        if Type::unify(&expected, &body.ty).is_none() {
                            errors.push(
                                CheckError::new(
                                    ErrorKind::Mismatch(expected, body.ty.clone()),
                                    self.span,
                                )
                            );
                        }
                    }
                }

                output.unwrap_or(Type::new(TypeKind::Void, self.span))
            }

            SymbolKind::Module(_) => {
                Type::new(TypeKind::Void, self.span)
            }
        };
        
        self.ty = ty
    }
}