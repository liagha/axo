use {
    super::{
        ErrorKind, Resolution, Resolvable, ResolveError, Resolver,
    },
    crate::{
        data::*,
        parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
        scanner::{OperatorKind, Token, TokenKind},
        analyzer::{Analyzable, AnalyzeError, ErrorKind as AnalyzeErrorKind},
        checker::{unify, CheckError, Checkable, Type, TypeKind},
        tracker::Span,
    },
};

impl<'element> Element<'element> {
    pub(crate) fn resolve_path(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Symbol<'element>, Vec<ResolveError<'element>>> {
        match &self.kind {
            ElementKind::Literal(Token {
                                     kind: TokenKind::Identifier(_),
                                     ..
                                 })
            | ElementKind::Construct(_) => resolver.scope.try_get(self),
            ElementKind::Binary(binary) => {
                let is_namespace = match &binary.operator.kind {
                    TokenKind::Operator(operator) => {
                        matches!(operator.as_slice(), [OperatorKind::Dot])
                    }
                    _ => false,
                };

                if !is_namespace {
                    return Err(vec![ResolveError::new(
                        ErrorKind::Analyze {
                            error: AnalyzeError::new(
                                AnalyzeErrorKind::InvalidOperation(binary.operator.clone()),
                                binary.operator.span,
                            ),
                        },
                        binary.operator.span,
                    )]);
                }

                let left = binary.left.resolve_path(resolver)?;
                resolver.enter_scope(left.scope.clone());
                let right = binary.right.resolve_path(resolver);
                resolver.exit();
                if matches!(left.kind, SymbolKind::Module(_)) {
                    if let Ok(ref member) = right {
                        if matches!(member.specifier.visibility, Visibility::Private) {
                            let token = member.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("<private>")),
                                member.span,
                            ));
                            return Err(vec![ResolveError::new(
                                ErrorKind::PrivateSymbol { symbol: token },
                                member.span,
                            )]);
                        }
                    }
                }
                right
            }
            _ => Err(vec![ResolveError::new(
                ErrorKind::Analyze {
                    error: AnalyzeError::new(
                        AnalyzeErrorKind::InvalidOperation(Token::new(
                            TokenKind::Operator(OperatorKind::Dot),
                            self.span,
                        )),
                        self.span,
                    ),
                },
                self.span,
            )]),
        }
    }

    fn filter_context(
        errors: Vec<ResolveError<'element>>,
    ) -> Result<(), Vec<ResolveError<'element>>> {
        let filtered: Vec<ResolveError<'element>> = errors
            .into_iter()
            .filter(|error| {
                !matches!(
                    &error.kind,
                    ErrorKind::Analyze { error }
                        if matches!(&error.kind, AnalyzeErrorKind::InvalidPrimitiveContext { .. })
                )
            })
            .collect();

        if filtered.is_empty() {
            Ok(())
        } else {
            Err(filtered)
        }
    }

    fn is_addressable(&self) -> bool {
        match &self.kind {
            ElementKind::Literal(Token {
                                     kind: TokenKind::Identifier(_),
                                     ..
                                 }) => true,
            ElementKind::Index(_) => true,
            ElementKind::Binary(binary) => {
                matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot))
            }
            ElementKind::Unary(unary) => {
                matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star))
            }
            _ => false,
        }
    }
}

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Resolution<'element>, Vec<ResolveError<'element>>> {
        let analysis = self.analyze(resolver).map_err(|error| {
            vec![ResolveError::new(
                ErrorKind::Analyze {
                    error: error.clone(),
                },
                error.span,
            )]
        })?;

        match &self.kind {
            ElementKind::Delimited(delimited) => {
                resolver.enter();

                delimited.members.iter().for_each(|item| {
                    item.resolve(resolver);
                });

                resolver.exit();

                let typ = delimited.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(None, typ, analysis))
            }

            ElementKind::Literal(
                Token {
                    kind: TokenKind::Identifier(_),
                    ..
                }) 
            => {
                let symbol = resolver.scope.try_get(&self)?;

                let typ = symbol.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(Some(symbol.id), typ, analysis))
            }

            ElementKind::Construct(construct) => {
                let symbol = resolver.scope.try_get(&self)?;

                let typ = symbol.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(Some(symbol.id), typ, analysis))
            }

            ElementKind::Invoke(invoke) => {
                let symbol = resolver.scope.try_get(self)?;

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
                                    crate::checker::ErrorKind::InvalidOperation(token),
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
                                    crate::checker::ErrorKind::Mismatch(expected, actual),
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
                            Type::unit(self.span)
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
                            Type::unit(self.span)
                        }
                        _ => Type::unit(self.span),
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

                if let TypeKind::Method(method) = &typ.kind {
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
                                    crate::checker::ErrorKind::InvalidOperation(token),
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
                                                    crate::checker::ErrorKind::Mismatch(
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
                                    crate::checker::ErrorKind::InvalidOperation(token),
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
                                        crate::checker::ErrorKind::Mismatch(
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
                                    crate::checker::ErrorKind::InvalidOperation(token),
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
                                        crate::checker::ErrorKind::Mismatch(
                                            Type::pointer(
                                                Type::new(TypeKind::Infer, invoke.members[0].span),
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
                                        crate::checker::ErrorKind::Mismatch(
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

                let output_type = if let TypeKind::Method(method) = &typ.kind {
                    if matches!(primitive.as_deref(), Some("alloc")) {
                        Type::pointer(Type::new(TypeKind::Infer, self.span), self.span)
                    } else if matches!(primitive.as_deref(), Some("or_else")) && invoke.members.len() == 2 {
                        invoke.members[1].resolve(resolver)?.typed
                    } else {
                        *method.output.clone()
                    }
                } else {
                    typ
                };

                Ok(Resolution::new(Some(symbol.id), output_type, analysis))
            },

            ElementKind::Index(index) => {
                let symbol = resolver.scope.try_get(&self)?;

                let typ = symbol.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(Some(symbol.id), typ, analysis))
            }

            ElementKind::Binary(binary) => {
                if matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot)) {
                    let left = binary.left.resolve(resolver)?;

                    if let Some(id) = left.reference {
                        let symbol = match resolver.scope.get_id(id).cloned() {
                            Some(symbol) => symbol,
                            None => binary.left.resolve_path(resolver)?,
                        };

                        resolver.enter_scope(symbol.scope.clone());

                        let resolved = binary.right.resolve(resolver);

                        resolver.exit();

                        resolved
                    } else {
                        let field = match &binary.right.kind {
                            ElementKind::Literal(Token {
                                                     kind: TokenKind::Identifier(name),
                                                     ..
                                                 }) => name.clone(),
                            _ => {
                                return Err(vec![ResolveError::new(
                                    ErrorKind::Analyze {
                                        error: AnalyzeError::new(
                                            AnalyzeErrorKind::InvalidOperation(binary.operator.clone()),
                                            binary.operator.span,
                                        ),
                                    },
                                    binary.operator.span,
                                )]);
                            }
                        };

                        let left_type = left.typed.clone();
                        let field_type = match left_type.kind {
                            TypeKind::Structure(structure) => {
                                let struct_name = structure.target.clone();
                                let identifier = Element::new(
                                    ElementKind::Literal(Token::new(
                                        TokenKind::Identifier(struct_name),
                                        Span::void(),
                                    )),
                                    Span::void(),
                                );
                                let struct_symbol = resolver
                                    .scope
                                    .try_get(&identifier)
                                    .map_err(|errors| errors)?;

                                let field_symbol = match struct_symbol.kind {
                                    SymbolKind::Structure(structure) => structure
                                        .members
                                        .iter()
                                        .find(|symbol| match &symbol.kind {
                                            SymbolKind::Binding(binding) => binding
                                                .target
                                                .brand()
                                                .and_then(|token| match token.kind {
                                                    TokenKind::Identifier(name) => Some(name == field),
                                                    _ => None,
                                                })
                                                .unwrap_or(false),
                                            _ => false,
                                        })
                                        .cloned(),
                                    _ => None,
                                };

                                if let Some(field_symbol) = field_symbol {
                                    let inferred: Result<
                                        Type<'element>,
                                        CheckError<'element>,
                                    > = field_symbol.infer();
                                    inferred.map_err(|error| {
                                        vec![ResolveError::new(
                                            ErrorKind::Check {
                                                error: error.clone(),
                                            },
                                            error.span,
                                        )]
                                    })?
                                } else {
                                    return Err(vec![ResolveError::new(
                                        ErrorKind::UndefinedSymbol {
                                            query: Token::new(TokenKind::Identifier(field), binary.right.span),
                                        },
                                        binary.right.span,
                                    )]);
                                }
                            }
                            _ => left_type.clone(),
                        };

                        Ok(Resolution::new(None, field_type, analysis))
                    }
                } else {
                    binary.left.resolve(resolver)?;
                    binary.right.resolve(resolver)?;

                    let typ = self.infer().map_err(|error| {
                        vec![ResolveError::new(
                            ErrorKind::Check {
                                error: error.clone(),
                            },
                            error.span,
                        )]
                    })?;

                    Ok(Resolution::new(None, typ, analysis))
                }
            },

            ElementKind::Unary(unary) => {
                let operand = unary.operand.resolve(resolver)?;
                let operator = match &unary.operator.kind {
                    TokenKind::Operator(operator) => operator,
                    _ => {
                        return Err(vec![ResolveError::new(
                            ErrorKind::Check {
                                error: CheckError::new(
                                    crate::checker::ErrorKind::InvalidOperation(
                                        unary.operator.clone(),
                                    ),
                                    unary.operator.span,
                                ),
                            },
                            unary.operator.span,
                        )]);
                    }
                };

                let typ = match operator.as_slice() {
                    [OperatorKind::Ampersand] => {
                        if !unary.operand.is_addressable() {
                            return Err(vec![ResolveError::new(
                                ErrorKind::Check {
                                    error: CheckError::new(
                                        crate::checker::ErrorKind::InvalidOperation(
                                            unary.operator.clone(),
                                        ),
                                        unary.operator.span,
                                    ),
                                },
                                unary.operator.span,
                            )]);
                        }

                        Type::pointer(operand.typed, self.span)
                    }
                    [OperatorKind::Star] => match operand.typed.kind {
                        TypeKind::Pointer { to } => *to,
                        TypeKind::Infer => Type::new(TypeKind::Infer, self.span),
                        _ => {
                            return Err(vec![ResolveError::new(
                                ErrorKind::Check {
                                    error: CheckError::new(
                                        crate::checker::ErrorKind::Mismatch(
                                            Type::pointer(
                                                Type::new(TypeKind::Infer, self.span),
                                                self.span,
                                            ),
                                            operand.typed,
                                        ),
                                        self.span,
                                    ),
                                },
                                self.span,
                            )]);
                        }
                    },
                    _ => self.infer().map_err(|error| {
                        vec![ResolveError::new(
                            ErrorKind::Check {
                                error: error.clone(),
                            },
                            error.span,
                        )]
                    })?,
                };

                Ok(Resolution::new(None, typ, analysis))
            },

            ElementKind::Symbolize(symbol) => symbol.resolve(resolver),

            ElementKind::Literal(_) => {
                let typ = self.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(None, typ, analysis))
            }
        }
    }

    fn is_instance(&self, resolver: &mut Resolver<'element>) -> Boolean {
        match &self.kind {
            ElementKind::Literal(Token {
                                     kind: TokenKind::Identifier(_),
                                     ..
                                 }) => false,
            ElementKind::Unary(Unary { operand, .. }) => operand.is_instance(resolver),
            ElementKind::Binary(
                Binary {
                    left,
                    operator:
                    Token {
                        kind: TokenKind::Operator(OperatorKind::Dot),
                        ..
                    },
                    ..
                }) => left.is_instance(resolver),
            ElementKind::Symbolize(symbol) => symbol.is_instance(resolver),
            _ => true,
        }
    }
}
