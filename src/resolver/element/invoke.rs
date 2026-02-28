use crate::{
    data::Str,
    parser::Element,
    resolver::{
        checker::{unify, CheckError, Checkable, Type},
        ErrorKind, Resolution, Resolvable, ResolveError, Resolver,
    },
    scanner::{Token, TokenKind},
    schema::Invoke,
    tracker::Span,
};

pub(super) fn resolve_invoke<'element>(
    element: &Element<'element>,
    invoke: &Invoke<Box<Element<'element>>, Element<'element>>,
    resolver: &mut Resolver<'element>,
    analysis: crate::resolver::analyzer::Analysis<'element>,
) -> Result<Resolution<'element>, Vec<ResolveError<'element>>> {
    let symbol = resolver.scope.try_get(element)?;

    let primitive = invoke.target.brand().and_then(|token| match token.kind {
        TokenKind::Identifier(name) => name.as_str().map(str::to_owned),
        _ => None,
    });

    let typ = if matches!(primitive.as_deref(), Some("if" | "while" | "for")) {
        let invalid = |name: &'static str, span: Span<'element>| {
            let token = invoke
                .target
                .brand()
                .unwrap_or(Token::new(TokenKind::Identifier(Str::from(name)), span));
            vec![ResolveError::new(
                ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::InvalidOperation(token),
                        span,
                    ),
                },
                span,
            )]
        };
        let mismatch = |expected: Type<'element>, actual: Type<'element>, span: Span<'element>| {
            vec![ResolveError::new(
                ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::Mismatch(expected, actual),
                        span,
                    ),
                },
                span,
            )]
        };

        match primitive.as_deref() {
            Some("if") => {
                if invoke.members.len() != 3 {
                    return Err(invalid("if", invoke.target.span));
                }

                let condition = invoke.members[0].resolve(resolver)?.typed;
                if !condition.is_boolean() {
                    return Err(mismatch(
                        Type::boolean(invoke.members[0].span),
                        condition,
                        invoke.members[0].span,
                    ));
                }

                let then = invoke.members[1].resolve(resolver)?.typed;
                let otherwise = invoke.members[2].resolve(resolver)?.typed;

                if let Some(unified) = unify(&then, &otherwise) {
                    unified
                } else {
                    return Err(mismatch(then, otherwise, invoke.members[2].span));
                }
            }
            Some("while") => {
                if invoke.members.len() != 2 {
                    return Err(invalid("while", invoke.target.span));
                }

                let condition = invoke.members[0].resolve(resolver)?.typed;
                if !condition.is_boolean() {
                    return Err(mismatch(
                        Type::boolean(invoke.members[0].span),
                        condition,
                        invoke.members[0].span,
                    ));
                }

                if let Err(errors) = invoke.members[1].resolve(resolver) {
                    Element::filter_context(errors)?;
                }
                Type::unit(element.span)
            }
            Some("for") => {
                if invoke.members.len() != 4 {
                    return Err(invalid("for", invoke.target.span));
                }

                invoke.members[0].resolve(resolver)?;

                let condition = invoke.members[1].resolve(resolver)?.typed;
                if !condition.is_boolean() {
                    return Err(mismatch(
                        Type::boolean(invoke.members[1].span),
                        condition,
                        invoke.members[1].span,
                    ));
                }

                if let Err(errors) = invoke.members[2].resolve(resolver) {
                    Element::filter_context(errors)?;
                }
                if let Err(errors) = invoke.members[3].resolve(resolver) {
                    Element::filter_context(errors)?;
                }
                Type::unit(element.span)
            }
            _ => Type::unit(element.span),
        }
    } else {
        symbol.infer().map_err(|error| {
            vec![ResolveError::new(
                ErrorKind::Check {
                    error: error.clone(),
                },
                error.span,
            )]
        })?
    };

    let mut invoke_errors = Vec::new();
    let is_builtin_primitive = matches!(
        primitive.as_deref(),
        Some(
            "print"
                | "print_raw"
                | "eprint"
                | "eprint_raw"
                | "read_line"
                | "len"
                | "write"
                | "alloc"
                | "free"
                | "is_some"
                | "is_none"
                | "Some"
                | "None"
                | "unwrap"
                | "or_else"
        )
    );

    if let crate::resolver::checker::TypeKind::Method(method) = &typ.kind {
        let expected = method.members.len();
        let provided = invoke.members.len();
        if (!method.variadic && provided != expected) || (method.variadic && provided < expected) {
            let token = invoke.target.brand().unwrap_or(Token::new(
                TokenKind::Identifier(Str::from("invoke")),
                invoke.target.span,
            ));
            invoke_errors.push(ResolveError::new(
                ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::InvalidOperation(token),
                        invoke.target.span,
                    ),
                },
                invoke.target.span,
            ));
        }

        if !is_builtin_primitive {
            for (argument, expected_type) in invoke.members.iter().zip(method.members.iter()) {
                match argument.resolve(resolver) {
                    Ok(actual_resolution) => {
                        let actual = actual_resolution.typed;
                        let compatible = **expected_type == actual
                            || (expected_type.is_numeric() && actual.is_numeric())
                            || unify(expected_type, &actual).is_some();
                        if !compatible {
                            invoke_errors.push(ResolveError::new(
                                ErrorKind::Check {
                                    error: CheckError::new(
                                        crate::resolver::checker::ErrorKind::Mismatch(
                                            (**expected_type).clone(),
                                            actual.clone(),
                                        ),
                                        argument.span,
                                    ),
                                },
                                argument.span,
                            ));
                        }
                    }
                    Err(errors) => invoke_errors.extend(errors),
                }
            }
        }
    }

    if matches!(primitive.as_deref(), Some("alloc")) {
        if invoke.members.len() != 1 {
            let token = invoke.target.brand().unwrap_or(Token::new(
                TokenKind::Identifier(Str::from("alloc")),
                invoke.target.span,
            ));
            invoke_errors.push(ResolveError::new(
                ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::InvalidOperation(token),
                        invoke.target.span,
                    ),
                },
                invoke.target.span,
            ));
        } else {
            let size = invoke.members[0].resolve(resolver)?.typed;
            if !size.is_integer() {
                invoke_errors.push(ResolveError::new(
                    ErrorKind::Check {
                        error: CheckError::new(
                            crate::resolver::checker::ErrorKind::Mismatch(
                                Type::integer(64, true, invoke.members[0].span),
                                size,
                            ),
                            invoke.members[0].span,
                        ),
                    },
                    invoke.members[0].span,
                ));
            }
        }
    }

    if matches!(primitive.as_deref(), Some("free")) {
        if invoke.members.len() != 2 {
            let token = invoke.target.brand().unwrap_or(Token::new(
                TokenKind::Identifier(Str::from("free")),
                invoke.target.span,
            ));
            invoke_errors.push(ResolveError::new(
                ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::InvalidOperation(token),
                        invoke.target.span,
                    ),
                },
                invoke.target.span,
            ));
        } else {
            let ptr = invoke.members[0].resolve(resolver)?.typed;
            if !ptr.is_pointer() {
                invoke_errors.push(ResolveError::new(
                    ErrorKind::Check {
                        error: CheckError::new(
                            crate::resolver::checker::ErrorKind::Mismatch(
                                Type::pointer(
                                    Type::new(crate::resolver::checker::TypeKind::Infer, invoke.members[0].span),
                                    invoke.members[0].span,
                                ),
                                ptr,
                            ),
                            invoke.members[0].span,
                        ),
                    },
                    invoke.members[0].span,
                ));
            }

            let size = invoke.members[1].resolve(resolver)?.typed;
            if !size.is_integer() {
                invoke_errors.push(ResolveError::new(
                    ErrorKind::Check {
                        error: CheckError::new(
                            crate::resolver::checker::ErrorKind::Mismatch(
                                Type::integer(64, true, invoke.members[1].span),
                                size,
                            ),
                            invoke.members[1].span,
                        ),
                    },
                    invoke.members[1].span,
                ));
            }
        }
    }

    if !invoke_errors.is_empty() {
        return Err(invoke_errors);
    }

    let output_type = if let crate::resolver::checker::TypeKind::Method(method) = &typ.kind {
        if matches!(primitive.as_deref(), Some("alloc")) {
            Type::pointer(Type::new(crate::resolver::checker::TypeKind::Infer, element.span), element.span)
        } else if matches!(primitive.as_deref(), Some("or_else")) && invoke.members.len() == 2 {
            invoke.members[1].resolve(resolver)?.typed
        } else {
            *method.output.clone()
        }
    } else {
        typ
    };

    Ok(Resolution::new(Some(symbol.id), output_type, analysis))
}
