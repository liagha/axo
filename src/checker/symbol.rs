use crate::{
    parser::{Symbol, SymbolKind},
    checker::{
        annotation, unify, CheckError, Checkable, ErrorKind,
    },
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn check(&mut self) -> Vec<CheckError<'symbol>> {
        match &self.kind {
            SymbolKind::Binding(binding) => {
                let declared = binding
                    .annotation
                    .as_ref()
                    .and_then(|value| annotation(value));

                let inferred = match binding.value.as_ref() {
                    Some(value) => {
                        let mut v = value.clone();
                        let errors = v.check();
                        if !errors.is_empty() {
                            return errors;
                        }
                        Some(v.ty.clone())
                    }
                    None => None,
                };

                match (declared, inferred) {
                    (Some(declared), Some(inferred)) => {
                        if unify(&declared, &inferred).is_none() {
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

                for field in structure.members.iter() {
                    let mut f = field.clone();
                    errors.extend(f.check());
                }

                errors
            }

            SymbolKind::Enumeration(enumeration) => {
                let mut errors = vec![];

                for field in enumeration.members.iter() {
                    let mut f = field.clone();
                    errors.extend(f.check());
                }

                errors
            }

            SymbolKind::Method(method) => {
                let mut errors = vec![];

                for field in method.members.iter() {
                    let mut f = field.clone();
                    errors.extend(f.check());
                }

                if !errors.is_empty() {
                    return errors;
                }

                let mut body_element = method.body.clone();
                let body_errors = body_element.check();

                if !body_errors.is_empty() {
                    return body_errors;
                }

                let body = body_element.ty.clone();

                let declared_output = method.output.as_ref().and_then(|value| annotation(value));

                if let Some(expected) = declared_output {
                    if unify(&expected, &body).is_none() {
                        return vec![CheckError::new(
                            ErrorKind::Mismatch(expected, body),
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
