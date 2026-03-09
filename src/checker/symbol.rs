use crate::{
    parser::{Symbol, SymbolKind},
    checker::{
        CheckError, Checkable, ErrorKind,
    },
};
use crate::checker::Type;

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self) -> Vec<CheckError<'symbol>> {
        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let mut errors = Vec::new();

                let annotation = binding
                    .annotation
                    .as_ref()
                    .map(
                        |value| {
                            match Type::annotation(&*value) {
                                Ok(ty) => Some(ty),
                                Err(error) => {
                                    errors.push(error);

                                    None
                                }
                            }
                        }
                    ).flatten();

                let inferred = match &mut binding.value {
                    Some(value) => {
                        let errors = value.check();

                        if !errors.is_empty() {
                            return errors;
                        }

                        Some(value.ty.clone())
                    }
                    None => None,
                };

                match (annotation, inferred) {
                    (Some(declared), Some(inferred)) => {
                        if Type::unify(&declared, &inferred).is_none() {
                            return vec![CheckError::new(
                                ErrorKind::Mismatch(declared, inferred.clone()),
                                inferred.span,
                            )];
                        }
                    }
                    _ => {}
                }

                vec![]
            }

            SymbolKind::Structure(structure) => {
                let mut errors = vec![];

                for member in structure.members.iter_mut() {
                    errors.extend(member.check());
                }

                errors
            }

            SymbolKind::Function(function) => {
                let mut errors = vec![];

                for member in function.members.iter_mut() {
                    errors.extend(member.check());
                }

                errors.extend(function.body.check());

                if !errors.is_empty() {
                    return errors;
                }

                let output = function
                    .output
                    .as_ref()
                    .map(
                        |value| {
                            match Type::annotation(&*value) {
                                Ok(ty) => Some(ty),
                                Err(error) => {
                                    errors.push(error);

                                    None
                                }
                            }
                        }
                    ).flatten();

                if let Some(expected) = output {
                    if Type::unify(&expected, &function.body.ty).is_none() {
                        return vec![CheckError::new(
                            ErrorKind::Mismatch(expected, function.body.ty.clone()),
                            self.span,
                        )];
                    }
                }

                vec![]
            }

            SymbolKind::Module(_) => vec![],
            SymbolKind::Preference(_) => vec![],
        }
    }
}
