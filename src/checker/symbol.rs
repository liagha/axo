use crate::{
    parser::{Symbol, SymbolKind},
    checker::{CheckError, Checkable, ErrorKind, Type},
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self) -> Result<(), Vec<CheckError<'symbol>>> {
        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let mut errors = Vec::new();

                let annotation = binding.annotation.as_ref().and_then(|value| {
                    match Type::annotation(&*value) {
                        Ok(ty) => Some(ty),
                        Err(error) => { errors.push(error); None }
                    }
                });

                let inferred = match &mut binding.value {
                    Some(value) => {
                        value.check()?;
                        Some(value.ty.clone())
                    }
                    None => None,
                };

                if let (Some(declared), Some(inferred)) = (annotation, inferred) {
                    if Type::unify(&declared, &inferred).is_none() {
                        return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(declared, inferred.clone()),
                            inferred.span,
                        )]);
                    }
                }

                if errors.is_empty() { Ok(()) } else { Err(errors) }
            }

            SymbolKind::Structure(structure) => {
                let mut errors = Vec::new();
                for member in structure.members.iter_mut() {
                    if let Err(errs) = member.check() { errors.extend(errs); }
                }
                if errors.is_empty() { Ok(()) } else { Err(errors) }
            }

            SymbolKind::Function(function) => {
                let mut errors = Vec::new();

                for member in function.members.iter_mut() {
                    if let Err(errs) = member.check() { errors.extend(errs); }
                }

                if let Err(errs) = function.body.check() { errors.extend(errs); }

                if !errors.is_empty() {
                    return Err(errors);
                }

                let output = function.output.as_ref().and_then(|value| {
                    match Type::annotation(&*value) {
                        Ok(ty) => Some(ty),
                        Err(error) => { errors.push(error); None }
                    }
                });

                if let Some(expected) = output {
                    if Type::unify(&expected, &function.body.ty).is_none() {
                        return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(expected, function.body.ty.clone()),
                            self.span,
                        )]);
                    }
                }

                Ok(())
            }

            SymbolKind::Module(_) | SymbolKind::Preference(_) => Ok(()),
        }
    }
}
