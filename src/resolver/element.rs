use crate::{
    data::{Aggregate, Delimited, Function, Interface, Scale, Str},
    parser::{Element, ElementKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Spanned,
};

fn assignable(element: &Element) -> bool {
    match &element.kind {
        ElementKind::Literal(token) => matches!(token.kind, TokenKind::Identifier(_)),
        ElementKind::Index(index) => assignable(&index.target),
        ElementKind::Binary(binary) => {
            matches!(
                &binary.operator.kind,
                TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Dot]
            ) && assignable(&binary.left)
        }
        ElementKind::Unary(unary) => {
            matches!(
                &unary.operator.kind,
                TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Star]
            )
        }
        _ => false,
    }
}

impl<'element> Resolvable<'element> for Element<'element> {
    fn declare(&mut self, resolver: &mut Resolver<'element>) {
        match &mut self.kind {
            ElementKind::Symbolize(symbol) => {
                symbol.declare(resolver);
                self.typing = symbol.typing.clone();
            }
            ElementKind::Delimited(delimited) => {
                if let Delimited {
                    start:
                    Token {
                        kind: TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        ..
                    },
                    members,
                    separator:
                    None
                    | Some(Token {
                               kind: TokenKind::Punctuation(PunctuationKind::Semicolon),
                               ..
                           }),
                    end:
                    Token {
                        kind: TokenKind::Punctuation(PunctuationKind::RightBrace),
                        ..
                    },
                } = delimited.as_mut()
                {
                    for member in members {
                        member.declare(resolver);
                    }
                }
            }
            _ => {}
        }
    }

    fn resolve(&mut self, resolver: &mut Resolver<'element>) {
        if self.kind.is_literal() {
            if let Some(token) = self.kind.try_unwrap_literal() {
                if let TokenKind::Identifier(_) = token.kind {
                    match resolver.lookup(self) {
                        Ok(symbol) => {
                            self.reference = Some(symbol.identity);
                        }
                        Err(errors) => resolver.errors.extend(errors),
                    }
                }
            }
        }

        self.typing = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type::from(TypeKind::Integer {
                    size: 64,
                    signed: true,
                }),
                TokenKind::Float(_) => Type::from(TypeKind::Float { size: 64 }),
                TokenKind::Boolean(_) => Type::from(TypeKind::Boolean),
                TokenKind::String(_) => Type::from(TypeKind::String),
                TokenKind::Character(_) => Type::from(TypeKind::Character),
                TokenKind::Identifier(_) => {
                    if let Ok(symbol) = resolver.lookup(self) {
                        symbol.typing
                    } else {
                        self.typing.clone()
                    }
                }
                _ => Type::from(TypeKind::Void),
            },

            ElementKind::Delimited(delimited) => match (
                &delimited.start.kind,
                delimited.separator.as_ref().map(|token| &token.kind),
                &delimited.end.kind,
            ) {
                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.separator.is_none() && delimited.members.len() == 1 {
                        delimited.members[0].resolve(resolver);
                        delimited.members[0].typing.clone()
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());

                        for member in &mut delimited.members {
                            member.resolve(resolver);
                            members.push(member.typing.clone());
                        }

                        Type::from(TypeKind::Tuple {
                            members: Box::new(members),
                        })
                    }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    resolver.enter();

                    let mut block = Type::from(TypeKind::Void);
                    let last = delimited.members.len().saturating_sub(1);

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.resolve(resolver);

                        if index == last {
                            block = member.typing.clone();
                        }
                    }

                    resolver.exit();

                    block
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    let mut inner = resolver.fresh();

                    for member in &mut delimited.members {
                        member.resolve(resolver);
                        inner = resolver.unify(member.span, &inner, &member.typing);
                    }

                    Type::from(TypeKind::Array {
                        member: Box::new(inner),
                        size: delimited.members.len() as Scale,
                    })
                }

                _ => Type::from(TypeKind::Void),
            },

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);

                match &unary.operator.kind {
                    TokenKind::Operator(operator) => match operator.as_slice() {
                        [OperatorKind::Exclamation] => resolver.unify(
                            unary.operand.span,
                            &unary.operand.typing,
                            &Type::from(TypeKind::Boolean),
                        ),
                        [OperatorKind::Tilde] => {
                            let expect = resolver.reify(&unary.operand.typing);

                            if !expect.kind.is_integer() && !expect.kind.is_variable() {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidUnary(
                                        unary.operator.clone(),
                                        expect.clone(),
                                    ),
                                    unary.operator.span,
                                ));
                            }

                            unary.operand.typing.clone()
                        }
                        [OperatorKind::Plus] | [OperatorKind::Minus] => {
                            unary.operand.typing.clone()
                        }
                        [OperatorKind::Ampersand] => {
                            if assignable(&unary.operand) {
                                Type::new(
                                    unary.operand.typing.identity,
                                    TypeKind::Pointer {
                                        target: Box::new(unary.operand.typing.clone()),
                                    },
                                )
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidUnary(
                                        unary.operator.clone(),
                                        unary.operand.typing.clone(),
                                    ),
                                    unary.operator.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Star] => {
                            let target = resolver.fresh();
                            let pointer = Type::new(
                                target.identity,
                                TypeKind::Pointer {
                                    target: Box::new(target.clone()),
                                },
                            );

                            resolver.unify(unary.operand.span, &unary.operand.typing, &pointer);

                            target
                        }
                        _ => {
                            resolver.errors.push(Error::new(
                                ErrorKind::InvalidUnary(
                                    unary.operator.clone(),
                                    unary.operand.typing.clone(),
                                ),
                                unary.operator.span,
                            ));
                            resolver.fresh()
                        }
                    },
                    _ => {
                        resolver.errors.push(Error::new(
                            ErrorKind::InvalidUnary(
                                unary.operator.clone(),
                                unary.operand.typing.clone(),
                            ),
                            unary.operator.span,
                        ));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                match &binary.operator.kind {
                    TokenKind::Operator(operator) => match operator.as_slice() {
                        [OperatorKind::Dot] => {
                            let mut left = resolver.reify(&binary.left.typing);

                            while left.kind.is_pointer() {
                                left = resolver.reify(&left.kind.unwrap_pointer());
                            }

                            let mut scope = None;

                            if let Some(reference) = binary.left.reference {
                                if let Some(symbol) = resolver.get_symbol(reference).cloned() {
                                    if !symbol.is_instance() {
                                        scope = Some(symbol.scope);
                                    }
                                }
                            }

                            if scope.is_none() {
                                if let Some(symbol) = resolver.get_symbol(left.identity).cloned() {
                                    scope = Some(symbol.scope);
                                }
                            }

                            if let Some(scope) = scope {
                                resolver.enter_scope(*scope);
                                binary.right.resolve(resolver);
                                resolver.exit();

                                self.reference = binary.right.reference;
                                binary.right.typing.clone()
                            } else {
                                binary.right.resolve(resolver);

                                if !left.kind.is_unknown() {
                                    resolver.errors.push(Error::new(
                                        ErrorKind::InvalidBinary(
                                            binary.operator.clone(),
                                            left.clone(),
                                            binary.right.typing.clone(),
                                        ),
                                        binary.operator.span,
                                    ));
                                }

                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Equal] => {
                            binary.right.resolve(resolver);

                            resolver.unify(
                                binary.right.span,
                                &binary.left.typing,
                                &binary.right.typing,
                            )
                        }
                        [OperatorKind::Plus]
                        | [OperatorKind::Minus]
                        | [OperatorKind::Star]
                        | [OperatorKind::Slash]
                        | [OperatorKind::Percent] => {
                            binary.right.resolve(resolver);

                            let left = resolver.reify(&binary.left.typing);
                            let right = resolver.reify(&binary.right.typing);

                            let valid = |kind: &TypeKind| {
                                kind.is_integer()
                                    || kind.is_float()
                                    || kind.is_pointer()
                                    || kind.is_variable()
                                    || kind.is_unknown()
                            };

                            if valid(&left.kind) && valid(&right.kind) {
                                resolver.unify(binary.right.span, &left, &right)
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidBinary(
                                        binary.operator.clone(),
                                        left.clone(),
                                        right.clone(),
                                    ),
                                    binary.right.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                        | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                            binary.right.resolve(resolver);

                            let left = resolver.reify(&binary.left.typing);
                            let right = resolver.reify(&binary.right.typing);

                            let valid = |kind: &TypeKind| {
                                kind.is_integer() || kind.is_variable() || kind.is_unknown()
                            };

                            if valid(&left.kind) && valid(&right.kind) {
                                resolver.unify(binary.right.span, &left, &right)
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidBinary(
                                        binary.operator.clone(),
                                        left.clone(),
                                        right.clone(),
                                    ),
                                    binary.right.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Ampersand]
                        | [OperatorKind::Pipe]
                        | [OperatorKind::Caret] => {
                            binary.right.resolve(resolver);

                            let left = resolver.reify(&binary.left.typing);
                            let right = resolver.reify(&binary.right.typing);

                            let valid = |kind: &TypeKind| {
                                kind.is_integer()
                                    || kind.is_boolean()
                                    || kind.is_variable()
                                    || kind.is_unknown()
                            };

                            if valid(&left.kind) && valid(&right.kind) {
                                resolver.unify(
                                    binary.right.span,
                                    &binary.left.typing,
                                    &binary.right.typing,
                                )
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidBinary(
                                        binary.operator.clone(),
                                        left.clone(),
                                        right.clone(),
                                    ),
                                    binary.right.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Ampersand, OperatorKind::Ampersand]
                        | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                            binary.right.resolve(resolver);

                            let boolean = Type::from(TypeKind::Boolean);

                            resolver.unify(binary.right.span, &binary.left.typing, &boolean);
                            resolver.unify(binary.right.span, &binary.right.typing, &boolean);

                            boolean
                        }
                        [OperatorKind::Equal, OperatorKind::Equal]
                        | [OperatorKind::Exclamation, OperatorKind::Equal]
                        | [OperatorKind::LeftAngle]
                        | [OperatorKind::LeftAngle, OperatorKind::Equal]
                        | [OperatorKind::RightAngle]
                        | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                            binary.right.resolve(resolver);

                            let merged = resolver.unify(
                                binary.right.span,
                                &binary.left.typing,
                                &binary.right.typing,
                            );
                            let real = resolver.reify(&merged);

                            let valid = real.kind.is_integer()
                                || real.kind.is_float()
                                || real.kind.is_boolean()
                                || real.kind.is_character()
                                || real.kind.is_string()
                                || real.kind.is_pointer()
                                || real.kind.is_variable()
                                || real.kind.is_unknown();

                            if !valid {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidBinary(
                                        binary.operator.clone(),
                                        binary.left.typing.clone(),
                                        binary.right.typing.clone(),
                                    ),
                                    binary.right.span,
                                ));
                            }

                            Type::from(TypeKind::Boolean)
                        }
                        _ => {
                            binary.right.resolve(resolver);

                            resolver.errors.push(Error::new(
                                ErrorKind::InvalidBinary(
                                    binary.operator.clone(),
                                    binary.left.typing.clone(),
                                    binary.right.typing.clone(),
                                ),
                                binary.operator.span,
                            ));
                            resolver.fresh()
                        }
                    },
                    _ => {
                        binary.right.resolve(resolver);

                        resolver.errors.push(Error::new(
                            ErrorKind::InvalidBinary(
                                binary.operator.clone(),
                                binary.left.typing.clone(),
                                binary.right.typing.clone(),
                            ),
                            binary.operator.span,
                        ));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    resolver.errors.push(Error::new(ErrorKind::EmptyIndex, self.span));

                    resolver.fresh()
                } else {
                    index.target.resolve(resolver);
                    index.members[0].resolve(resolver);

                    let target = resolver.reify(&index.target.typing);
                    let member = resolver.reify(&index.members[0].typing);

                    let expect = Type::from(TypeKind::Integer {
                        size: 64,
                        signed: true,
                    });

                    resolver.unify(index.members.span(), &member, &expect);

                    if target.kind.is_pointer() {
                        let base = target.kind.unwrap_pointer();
                        if base.kind.is_array() {
                            *base.kind.unwrap_array().0
                        } else {
                            *base
                        }
                    } else if target.kind.is_array() {
                        *target.kind.unwrap_array().0
                    } else if target.kind.is_tuple() {
                        let members = target.kind.unwrap_tuple();
                        let mut value = 0;

                        if let Some(token) = index.members[0].kind.try_unwrap_literal() {
                            if let TokenKind::Integer(literal) = &token.kind {
                                value = usize::try_from(*literal).unwrap_or(0);
                            }
                        }

                        if value < members.len() {
                            members[value].clone()
                        } else {
                            resolver.errors.push(Error::new(
                                ErrorKind::IndexBounds(value, members.len()),
                                index.members.span(),
                            ));
                            resolver.fresh()
                        }
                    } else if target.kind.is_variable() {
                        let element = resolver.fresh();
                        let pointer = Type::new(
                            element.identity,
                            TypeKind::Pointer {
                                target: Box::new(element.clone()),
                            },
                        );

                        resolver.unify(index.members.span(), &target, &pointer);

                        element
                    } else {
                        resolver.errors.push(Error::new(ErrorKind::Unindexable, index.target.span));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);
                self.reference = invoke.target.reference;

                let target = invoke.target.target().and_then(|name| name.as_str());

                match target {
                    Some("if") => {
                        if invoke.members.len() < 2 {
                            Type::from(TypeKind::Void)
                        } else {
                            resolver.enter();

                            invoke.members[0].resolve(resolver);

                            let boolean = Type::from(TypeKind::Boolean);

                            resolver.unify(
                                invoke.members[0].span,
                                &invoke.members[0].typing,
                                &boolean,
                            );

                            invoke.members[1].resolve(resolver);

                            let then = invoke.members[1].typing.clone();

                            let typing = if invoke.members.len() == 3 {
                                invoke.members[2].resolve(resolver);

                                resolver.unify(
                                    invoke.members[2].span,
                                    &then,
                                    &invoke.members[2].typing,
                                )
                            } else {
                                let void = Type::from(TypeKind::Void);
                                resolver.unify(invoke.members[1].span, &then, &void);
                                void
                            };

                            resolver.exit();

                            typing
                        }
                    }
                    Some("while") => {
                        if !invoke.members.is_empty() {
                            invoke.members[0].resolve(resolver);

                            let boolean = Type::from(TypeKind::Boolean);

                            resolver.unify(
                                invoke.members[0].span,
                                &invoke.members[0].typing,
                                &boolean,
                            );
                        }

                        if invoke.members.len() > 1 {
                            invoke.members[1].resolve(resolver);
                        }

                        Type::from(TypeKind::Void)
                    }
                    Some("return") => {
                        if !invoke.members.is_empty() {
                            invoke.members[0].resolve(resolver);
                        }

                        let value = invoke.members.first().map_or_else(
                            || Type::from(TypeKind::Void),
                            |member| member.typing.clone(),
                        );

                        if let Some(expect) = resolver.returns.last().cloned() {
                            resolver.unify(self.span, &expect, &value);
                        }

                        Type::from(TypeKind::Unknown)
                    }
                    Some("continue") | Some("break") => Type::from(TypeKind::Unknown),
                    _ => {
                        for member in &mut invoke.members {
                            member.resolve(resolver);
                        }

                        let output = resolver.fresh();
                        let body = resolver.fresh();
                        let mut members = Vec::new();

                        members.extend(invoke.members.iter().map(|member| member.typing.clone()));

                        let mut function = Type::from(TypeKind::Function(Box::new(Function::new(
                            Str::default(),
                            members,
                            body,
                            Some(Box::new(output.clone())),
                            Interface::Axo,
                            false,
                            false,
                        ))));

                        function = resolver.unify(self.span, &invoke.target.typing, &function);

                        if function.kind.is_function() {
                            if let Some(kind) = function.kind.unwrap_function().output {
                                *kind
                            } else {
                                Type::from(TypeKind::Void)
                            }
                        } else {
                            output
                        }
                    }
                }
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);
                self.reference = construct.target.reference;

                let mut layout = Vec::new();
                let mut typing = None;

                if let Some(reference) = construct.target.reference {
                    if let Some(symbol) = resolver.get_symbol(reference).cloned() {
                        match &symbol.typing.kind {
                            TypeKind::Structure(aggregate) => {
                                layout = aggregate.members.clone();
                                typing = Some(TypeKind::Structure(aggregate.clone()));
                            }
                            TypeKind::Union(aggregate) => {
                                layout = aggregate.members.clone();
                                typing = Some(TypeKind::Union(aggregate.clone()));
                            }
                            _ => {}
                        }

                        if typing.is_some() {
                            resolver.enter_scope(*symbol.scope.clone());
                            for (index, member) in construct.members.iter_mut().enumerate() {
                                member.resolve(resolver);
                                if let Some(expect) = layout.get(index) {
                                    resolver.unify(member.span, &member.typing, expect);
                                }
                            }
                            resolver.exit();
                        }
                    }
                }

                let head = construct.target.target().unwrap_or_default();
                let aggregate = Aggregate::new(head, layout);
                let reference = construct.target.reference.unwrap_or(0);

                Type::new(
                    reference,
                    typing.unwrap_or(TypeKind::Structure(Box::from(aggregate))),
                )
            }

            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);
                symbol.resolve(resolver);
                symbol.typing.clone()
            }
        };
    }

    fn is_instance(&self) -> bool {
        self.typing.kind.is_structure() || self.typing.kind.is_union()
    }
}
