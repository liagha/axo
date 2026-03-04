use {
    crate::{
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{Token, TokenKind},
        checker::{
            annotation, unify, CheckError, Checkable, ErrorKind, Type, TypeKind,
        },
        data::*,
        format::Show,
    },
};

fn returns<'symbol>(
    element: &Element<'symbol>,
    expected: &Type<'symbol>,
) -> Result<bool, CheckError<'symbol>> {
    match &element.kind {
        ElementKind::Invoke(invoke) => {
            let is_return = invoke
                .target
                .brand()
                .and_then(|token| match token.kind {
                    TokenKind::Identifier(name) => name.as_str().map(|value| value == "return"),
                    _ => None,
                })
                .unwrap_or(false);

            if is_return {
                let actual = match invoke.members.len() {
                    0 => Type::unit(element.span),
                    1 => invoke.members[0].infer()?,
                    _ => {
                        let token = invoke.target.brand().unwrap_or(Token::new(
                            TokenKind::Identifier(Str::from("return")),
                            element.span,
                        ));
                        return Err(CheckError::new(ErrorKind::InvalidOperation(token.clone()), token.span));
                    }
                };

                if unify(expected, &actual).is_some() {
                    return Ok(true);
                }

                return Err(CheckError::new(
                    ErrorKind::Mismatch(expected.clone(), actual.clone()),
                    actual.span,
                ));
            }

            let mut found = returns(&invoke.target, expected)?;
            for member in &invoke.members {
                found |= returns(member, expected)?;
            }
            Ok(found)
        }
        ElementKind::Delimited(delimited) => {
            let mut found = false;
            for member in &delimited.members {
                found |= returns(member, expected)?;
            }
            Ok(found)
        }
        ElementKind::Unary(unary) => returns(&unary.operand, expected),
        ElementKind::Binary(binary) => {
            let left = returns(&binary.left, expected)?;
            let right = returns(&binary.right, expected)?;
            Ok(left || right)
        }
        ElementKind::Index(index) => {
            let mut found = returns(&index.target, expected)?;
            for member in &index.members {
                found |= returns(member, expected)?;
            }
            Ok(found)
        }
        ElementKind::Construct(construct) => {
            let mut found = returns(&construct.target, expected)?;
            for member in &construct.members {
                found |= returns(member, expected)?;
            }
            Ok(found)
        }
        ElementKind::Symbolize(_) | ElementKind::Literal(_) => Ok(false),
    }
}

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn infer(&self) -> Result<Type<'symbol>, CheckError<'symbol>> {
        match &self.kind {
            SymbolKind::Inclusion(_) => Ok(Type::unit(self.span)),
            SymbolKind::Extension(_) => Ok(Type::unit(self.span)),
            SymbolKind::Binding(binding) => {
                let declared = binding
                    .annotation
                    .as_ref()
                    .and_then(|value| annotation(value));

                let inferred = binding
                    .value
                    .as_ref()
                    .map(|value| value.infer())
                    .transpose()?;

                match (declared, inferred) {
                    (Some(declared), Some(inferred)) => {
                        if let Some(unified) = unify(&declared, &inferred) {
                            Ok(unified)
                        } else {
                            Err(CheckError::new(
                                ErrorKind::Mismatch(declared, inferred.clone()),
                                inferred.span,
                            ))
                        }
                    }
                    (Some(declared), None) => Ok(declared),
                    (None, Some(inferred)) => Ok(inferred),
                    (None, None) => Ok(Type::unit(self.span)),
                }
            }
            SymbolKind::Structure(structure) => {
                let members: Result<Vec<Box<Type<'symbol>>>, CheckError<'symbol>> = structure
                    .members
                    .iter()
                    .map(|field| field.clone().infer().map(Box::new))
                    .collect();

                let structure = Structure::new(
                    Str::from(structure.target.brand().unwrap().format(0)),
                    members?,
                );

                Ok(Type::new(TypeKind::Structure(structure), self.span))
            }
            SymbolKind::Enumeration(enumeration) => {
                let members: Result<Vec<Box<Type<'symbol>>>, CheckError<'symbol>> = enumeration
                    .members
                    .iter()
                    .map(|field| field.clone().infer().map(Box::new))
                    .collect();

                let enumeration = Structure::new(
                    Str::from(enumeration.target.brand().unwrap().format(0)),
                    members?,
                );

                Ok(Type::new(TypeKind::Enumeration(enumeration), self.span))
            }
            SymbolKind::Method(method) => {
                let members: Result<Vec<Box<Type<'symbol>>>, CheckError<'symbol>> = method
                    .members
                    .iter()
                    .map(|field| field.clone().infer().map(Box::new))
                    .collect();

                let body = method.body.infer()?;

                let declared_output = method.output.as_ref().and_then(|value| annotation(value));

                let output = if let Some(expected) = declared_output.clone() {
                    let explicit_return = returns(&method.body, &expected)?;

                    if !explicit_return && !unify(&expected, &body).is_some() {
                        return Err(CheckError::new(
                            ErrorKind::Mismatch(expected.clone(), body.clone()),
                            body.span,
                        ));
                    }

                    Box::new(expected)
                } else {
                    Box::new(body.clone())
                };

                let method = Method::new(
                    Str::from(method.target.brand().unwrap().format(0)),
                    members?,
                    Box::new(body),
                    output,
                    method.variadic,
                );

                Ok(Type::new(TypeKind::Method(method), self.span))
            }
            SymbolKind::Module(_) => Ok(Type::unit(self.span)),
            SymbolKind::Preference(_) => Ok(Type::unit(self.span)),
        }
    }
}
