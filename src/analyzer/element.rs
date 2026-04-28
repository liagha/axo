use crate::{
    analyzer::{Analysis, AnalysisKind, Analyzable, AnalyzeError, ErrorKind, Target},
    data::*,
    parser::SymbolKind,
    parser::{Element, ElementKind},
    resolver::{Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
};

fn target<'a>(id: Option<Identity>, name: Option<Str<'a>>) -> Target<'a> {
    Target::new(id.unwrap_or_default(), name.unwrap_or_default())
}

fn name<'a>(typing: &Type<'a>) -> Option<Str<'a>> {
    match &typing.kind {
        TypeKind::Binding(binding) => Some(binding.target),
        TypeKind::Function(function) if !function.target.is_empty() => Some(function.target),
        TypeKind::Has(target) => name(target),
        _ => None,
    }
}

fn slot<'a>(typing: &Type<'a>, id: Option<Identity>, label: Option<Str<'a>>) -> Option<Scale> {
    match &typing.kind {
        TypeKind::Pointer { target } => slot(target, id, label),
        TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => aggregate
            .members
            .iter()
            .position(|member| {
                id.is_some_and(|id| member.identity == id)
                    || label.is_some_and(|label| name(member) == Some(label))
            })
            .map(|slot| slot as Scale),
        _ => None,
    }
}

fn mutate<'a>(
    target: Analysis<'a>,
    value: Analysis<'a>,
    operator: &Token<'a>,
) -> Result<AnalysisKind<'a>, AnalyzeError<'a>> {
    match &target.kind {
        AnalysisKind::Symbol(name) => Ok(AnalysisKind::Write(name.clone(), Box::new(value))),
        AnalysisKind::Dereference(_) | AnalysisKind::Slot(_, _) | AnalysisKind::Index(_) => {
            Ok(AnalysisKind::Store(Box::new(target), Box::new(value)))
        }
        _ => Err(AnalyzeError::new(
            ErrorKind::InvalidMutation(
                operator.clone(),
                target.typing.clone(),
                value.typing.clone(),
            ),
            operator.span,
        )),
    }
}

impl<'element> Analyzable<'element> for Element<'element> {
    fn analyze(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Analysis<'element>, AnalyzeError<'element>> {
        let typing = self.typing.clone();

        match &self.kind {
            ElementKind::Literal(literal) => {
                let kind = match &literal.kind {
                    TokenKind::Integer(value) => {
                        let (size, signed) = match &typing.kind {
                            TypeKind::Integer { size, signed } => (*size, *signed),
                            _ => (64, true),
                        };
                        AnalysisKind::Integer {
                            value: *value,
                            size,
                            signed,
                        }
                    }
                    TokenKind::Float(value) => {
                        let size = match &typing.kind {
                            TypeKind::Float { size } => *size,
                            _ => 64,
                        };
                        AnalysisKind::Float {
                            value: *value,
                            size,
                        }
                    }
                    TokenKind::Boolean(value) => AnalysisKind::Boolean { value: *value },
                    TokenKind::String(value) => AnalysisKind::String { value: **value },
                    TokenKind::Character(value) => AnalysisKind::Character { value: *value },
                    TokenKind::Identifier(identifier) => {
                        AnalysisKind::Symbol(target(self.reference, Some(**identifier)))
                    }
                    _ => unreachable!("unreachable token kind."),
                };

                Ok(Analysis::new(kind, self.span, typing))
            }

            ElementKind::Delimited(delimited) => {
                let kind = match (
                    &delimited.start.kind,
                    delimited.separator.as_ref().map(|token| &token.kind),
                    &delimited.end.kind,
                ) {
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) => AnalysisKind::Block(
                        delimited
                            .members
                            .iter()
                            .map(|item| item.analyze(resolver))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                        _,
                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                    ) => AnalysisKind::Array(
                        delimited
                            .members
                            .iter()
                            .map(|item| item.analyze(resolver))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        _,
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        if delimited.members.is_empty() {
                            AnalysisKind::Tuple(Vec::new())
                        } else if delimited.separator.is_none() && delimited.members.len() == 1 {
                            return delimited.members[0].analyze(resolver);
                        } else {
                            AnalysisKind::Tuple(
                                delimited
                                    .members
                                    .iter()
                                    .map(|item| item.analyze(resolver))
                                    .collect::<Result<Vec<_>, _>>()?,
                            )
                        }
                    }
                    _ => unreachable!("unknown delimited kind!"),
                };

                Ok(Analysis::new(kind, self.span, typing))
            }

            ElementKind::Unary(unary) => {
                if let TokenKind::Operator(operator) = &unary.operator.kind {
                    let operand = unary.operand.analyze(resolver)?;

                    let kind = match operator.as_slice() {
                        [OperatorKind::Exclamation] => AnalysisKind::LogicalNot(Box::new(operand)),
                        [OperatorKind::Tilde] => AnalysisKind::BitwiseNot(Box::new(operand)),
                        [OperatorKind::Plus] => return Ok(operand),
                        [OperatorKind::Minus] => AnalysisKind::Negate(Box::new(operand)),
                        [OperatorKind::Ampersand] => AnalysisKind::AddressOf(Box::new(operand)),
                        [OperatorKind::Star] => AnalysisKind::Dereference(Box::new(operand)),
                        [OperatorKind::Plus, OperatorKind::Plus] => {
                            let step = Analysis::new(
                                AnalysisKind::Integer {
                                    value: 1,
                                    size: 64,
                                    signed: true,
                                },
                                unary.operator.span,
                                typing.clone(),
                            );
                            let value = Analysis::new(
                                AnalysisKind::Add(Box::new(operand.clone()), Box::new(step)),
                                self.span,
                                typing.clone(),
                            );
                            mutate(operand, value, &unary.operator)?
                        }
                        [OperatorKind::Minus, OperatorKind::Minus] => {
                            let step = Analysis::new(
                                AnalysisKind::Integer {
                                    value: 1,
                                    size: 64,
                                    signed: true,
                                },
                                unary.operator.span,
                                typing.clone(),
                            );
                            let value = Analysis::new(
                                AnalysisKind::Subtract(Box::new(operand.clone()), Box::new(step)),
                                self.span,
                                typing.clone(),
                            );
                            mutate(operand, value, &unary.operator)?
                        }
                        _ => {
                            return Err(AnalyzeError::new(
                                ErrorKind::InvalidUnary(
                                    unary.operator.clone(),
                                    unary.operand.typing.clone(),
                                ),
                                unary.operator.span,
                            ))
                        }
                    };

                    return Ok(Analysis::new(kind, self.span, typing));
                }

                Err(AnalyzeError::new(
                    ErrorKind::InvalidUnary(unary.operator.clone(), unary.operand.typing.clone()),
                    unary.operator.span,
                ))
            }

            ElementKind::Binary(binary) => {
                let op_kind = if let TokenKind::Operator(operator) = &binary.operator.kind {
                    operator
                } else {
                    return Err(AnalyzeError::new(
                        ErrorKind::InvalidBinary(
                            binary.operator.clone(),
                            binary.left.typing.clone(),
                            binary.right.typing.clone(),
                        ),
                        binary.operator.span,
                    ));
                };

                let kind = match op_kind.as_slice() {
                    [OperatorKind::Dot] => {
                        if binary.left.typing.kind.is_module()
                            || binary
                                .left
                                .reference
                                .and_then(|reference| resolver.get_symbol(reference))
                                .is_some_and(|symbol| {
                                    !matches!(symbol.kind, SymbolKind::Binding(_))
                                })
                        {
                            return binary.right.analyze(resolver);
                        }

                        let target = binary.left.analyze(resolver)?;
                        let slot = slot(
                            &binary.left.typing,
                            binary.right.reference,
                            binary.right.target(),
                        )
                        .ok_or_else(|| {
                            AnalyzeError::new(ErrorKind::InvalidTarget, binary.operator.span)
                        })?;

                        AnalysisKind::Slot(Box::new(target), slot)
                    }
                    [OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let value = binary.right.analyze(resolver)?;
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Plus, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::Add(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Minus, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::Subtract(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Star, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::Multiply(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Slash, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::Divide(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Percent, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::Modulus(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Ampersand, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::BitwiseAnd(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Pipe, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::BitwiseOr(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Caret, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::LogicalXOr(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::LeftAngle, OperatorKind::LeftAngle, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::ShiftLeft(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::RightAngle, OperatorKind::RightAngle, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(
                            AnalysisKind::ShiftRight(Box::new(target.clone()), Box::new(right)),
                            self.span,
                            typing.clone(),
                        );
                        mutate(target, value, &binary.operator)?
                    }
                    _ => {
                        let left = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;

                        match op_kind.as_slice() {
                            [OperatorKind::Plus] => match (&left.typing.kind, &right.typing.kind) {
                                (TypeKind::Pointer { target }, _) => {
                                    let size = Analysis::new(
                                        AnalysisKind::SizeOf((**target).clone()),
                                        right.span,
                                        right.typing.clone(),
                                    );
                                    let scale = Analysis::new(
                                        AnalysisKind::Multiply(
                                            Box::new(right.clone()),
                                            Box::new(size),
                                        ),
                                        right.span,
                                        right.typing.clone(),
                                    );
                                    AnalysisKind::Add(Box::new(left), Box::new(scale))
                                }
                                (_, TypeKind::Pointer { target }) => {
                                    let size = Analysis::new(
                                        AnalysisKind::SizeOf((**target).clone()),
                                        left.span,
                                        left.typing.clone(),
                                    );
                                    let scale = Analysis::new(
                                        AnalysisKind::Multiply(
                                            Box::new(left.clone()),
                                            Box::new(size),
                                        ),
                                        left.span,
                                        left.typing.clone(),
                                    );
                                    AnalysisKind::Add(Box::new(scale), Box::new(right))
                                }
                                _ => AnalysisKind::Add(Box::new(left), Box::new(right)),
                            },
                            [OperatorKind::Minus] => {
                                match (&left.typing.kind, &right.typing.kind) {
                                    (
                                        TypeKind::Pointer {
                                            target: left_target,
                                        },
                                        TypeKind::Pointer {
                                            target: right_target,
                                        },
                                    ) => {
                                        if left_target == right_target {
                                            let size = Analysis::new(
                                                AnalysisKind::SizeOf((**left_target).clone()),
                                                self.span,
                                                typing.clone(),
                                            );
                                            let value = Analysis::new(
                                                AnalysisKind::Subtract(
                                                    Box::new(left),
                                                    Box::new(right),
                                                ),
                                                self.span,
                                                typing.clone(),
                                            );
                                            AnalysisKind::Divide(Box::new(value), Box::new(size))
                                        } else {
                                            return Err(AnalyzeError::new(
                                                ErrorKind::InvalidBinary(
                                                    binary.operator.clone(),
                                                    left.typing.clone(),
                                                    right.typing.clone(),
                                                ),
                                                binary.operator.span,
                                            ));
                                        }
                                    }
                                    (TypeKind::Pointer { target }, _) => {
                                        let size = Analysis::new(
                                            AnalysisKind::SizeOf((**target).clone()),
                                            right.span,
                                            right.typing.clone(),
                                        );
                                        let scale = Analysis::new(
                                            AnalysisKind::Multiply(
                                                Box::new(right.clone()),
                                                Box::new(size),
                                            ),
                                            right.span,
                                            right.typing.clone(),
                                        );
                                        AnalysisKind::Subtract(Box::new(left), Box::new(scale))
                                    }
                                    _ => AnalysisKind::Subtract(Box::new(left), Box::new(right)),
                                }
                            }
                            [OperatorKind::Star] => {
                                AnalysisKind::Multiply(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Slash] => {
                                AnalysisKind::Divide(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Percent] => {
                                AnalysisKind::Modulus(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Ampersand, OperatorKind::Ampersand] => {
                                AnalysisKind::LogicalAnd(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Pipe, OperatorKind::Pipe] => {
                                AnalysisKind::LogicalOr(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Caret] => {
                                AnalysisKind::LogicalXOr(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Ampersand] => {
                                AnalysisKind::BitwiseAnd(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Pipe] => {
                                AnalysisKind::BitwiseOr(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => {
                                AnalysisKind::ShiftLeft(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                                AnalysisKind::ShiftRight(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Equal, OperatorKind::Equal] => {
                                AnalysisKind::Equal(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::Exclamation, OperatorKind::Equal] => {
                                AnalysisKind::NotEqual(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::LeftAngle] => {
                                AnalysisKind::Less(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::LeftAngle, OperatorKind::Equal] => {
                                AnalysisKind::LessOrEqual(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::RightAngle] => {
                                AnalysisKind::Greater(Box::new(left), Box::new(right))
                            }
                            [OperatorKind::RightAngle, OperatorKind::Equal] => {
                                AnalysisKind::GreaterOrEqual(Box::new(left), Box::new(right))
                            }
                            _ => {
                                return Err(AnalyzeError::new(
                                    ErrorKind::InvalidBinary(
                                        binary.operator.clone(),
                                        binary.left.typing.clone(),
                                        binary.right.typing.clone(),
                                    ),
                                    binary.operator.span,
                                ))
                            }
                        }
                    }
                };

                Ok(Analysis::new(kind, self.span, typing))
            }

            ElementKind::Index(index) => {
                let target = index.target.analyze(resolver)?;
                let members = index
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Analysis::new(
                    AnalysisKind::Index(Index::new(Box::new(target), members)),
                    self.span,
                    typing,
                ))
            }

            ElementKind::Invoke(invoke) => {
                let name = invoke.target.target();

                let kind = match name.as_ref().and_then(|name| name.as_str()) {
                    Some("if") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let then = invoke.members[1].analyze(resolver)?;
                        let otherwise = if invoke.members.len() == 3 {
                            Some(Box::new(invoke.members[2].analyze(resolver)?))
                        } else {
                            None
                        };
                        AnalysisKind::Conditional(Box::new(condition), Box::new(then), otherwise)
                    }
                    Some("while") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let body = invoke.members[1].analyze(resolver)?;
                        AnalysisKind::While(Box::new(condition), Box::new(body))
                    }
                    Some("break") => {
                        let value = if !invoke.members.is_empty() {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else {
                            None
                        };
                        AnalysisKind::Break(value)
                    }
                    Some("continue") => {
                        let value = if !invoke.members.is_empty() {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else {
                            None
                        };
                        AnalysisKind::Continue(value)
                    }
                    Some("return") => {
                        let value = if !invoke.members.is_empty() {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else {
                            None
                        };
                        AnalysisKind::Return(value)
                    }
                    _ => AnalysisKind::Call(
                        target(invoke.target.reference, invoke.target.target()),
                        invoke
                            .members
                            .iter()
                            .map(|member| member.analyze(resolver))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                };

                Ok(Analysis::new(kind, self.span, typing))
            }

            ElementKind::Construct(construct) => {
                let mut values = Vec::with_capacity(construct.members.len());
                let mut next = 0;

                for member in &construct.members {
                    match &member.kind {
                        ElementKind::Binary(binary)
                            if matches!(
                                &binary.operator.kind,
                                TokenKind::Operator(operator)
                                    if operator.as_slice() == [OperatorKind::Equal]
                            ) =>
                        {
                            let slot = slot(&typing, binary.left.reference, binary.left.target())
                                .ok_or_else(|| {
                                AnalyzeError::new(ErrorKind::InvalidTarget, member.span)
                            })?;
                            values.push((slot, binary.right.analyze(resolver)?));
                            next = slot + 1;
                        }
                        _ => {
                            values.push((next, member.analyze(resolver)?));
                            next += 1;
                        }
                    }
                }

                values.sort_by_key(|(slot, _)| *slot);

                Ok(Analysis::new(
                    AnalysisKind::Pack(
                        target(construct.target.reference, construct.target.target()),
                        values,
                    ),
                    self.span,
                    typing,
                ))
            }

            ElementKind::Symbolize(symbol) => symbol.analyze(resolver),
        }
    }
}
