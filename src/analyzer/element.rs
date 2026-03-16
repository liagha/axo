use crate::{
    data::*,
    analyzer::{Analyzable, Analysis, AnalysisKind, AnalyzeError, ErrorKind},
    format::Show,
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, TokenKind, Token},
    resolver::{Resolver, TypeKind},
};

fn mutate<'element>(
    target: Analysis<'element>,
    value: Analysis<'element>,
    operator: &Token<'element>,
) -> Result<AnalysisKind<'element>, AnalyzeError<'element>> {
    match &target.kind {
        AnalysisKind::Usage(name) => Ok(AnalysisKind::Assign(name.clone(), Box::new(value))),
        AnalysisKind::Dereference(_) | AnalysisKind::Access(_, _) | AnalysisKind::Index(_) => {
            Ok(AnalysisKind::Store(Box::new(target), Box::new(value)))
        }
        _ => Err(AnalyzeError::new(
            ErrorKind::InvalidOperation(operator.clone()),
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
                        AnalysisKind::Integer { value: *value, size, signed }
                    }
                    TokenKind::Float(value) => {
                        let size = match &typing.kind {
                            TypeKind::Float { size } => *size,
                            _ => 64,
                        };
                        AnalysisKind::Float { value: *value, size }
                    }
                    TokenKind::Boolean(value) => AnalysisKind::Boolean { value: *value },
                    TokenKind::String(value) => AnalysisKind::String { value: *value },
                    TokenKind::Character(value) => AnalysisKind::Character { value: *value },
                    TokenKind::Identifier(identifier) => AnalysisKind::Usage(identifier.clone()),
                    _ => return Err(AnalyzeError::new(ErrorKind::Unimplemented, self.span)),
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
                    ) => {
                        let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = delimited
                            .members
                            .iter()
                            .map(|item| item.analyze(resolver))
                            .collect();

                        AnalysisKind::Block(items?)
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                        _,
                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                    ) => {
                        let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = delimited
                            .members
                            .iter()
                            .map(|item| item.analyze(resolver))
                            .collect();

                        AnalysisKind::Array(items?)
                    }

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
                            let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = delimited
                                .members
                                .iter()
                                .map(|item| item.analyze(resolver))
                                .collect();

                            AnalysisKind::Tuple(items?)
                        }
                    }

                    _ => return Err(AnalyzeError::new(ErrorKind::Unimplemented, self.span)),
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
                            let step = Analysis::new(AnalysisKind::Integer { value: 1, size: 64, signed: true }, unary.operator.span, typing.clone());
                            let value = Analysis::new(AnalysisKind::Add(Box::new(operand.clone()), Box::new(step)), self.span, typing.clone());
                            mutate(operand, value, &unary.operator)?
                        }
                        [OperatorKind::Minus, OperatorKind::Minus] => {
                            let step = Analysis::new(AnalysisKind::Integer { value: 1, size: 64, signed: true }, unary.operator.span, typing.clone());
                            let value = Analysis::new(AnalysisKind::Subtract(Box::new(operand.clone()), Box::new(step)), self.span, typing.clone());
                            mutate(operand, value, &unary.operator)?
                        }
                        _ => return Err(AnalyzeError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span))
                    };

                    return Ok(Analysis::new(kind, self.span, typing));
                }

                Err(AnalyzeError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span))
            },

            ElementKind::Binary(binary) => {
                let op_kind = if let TokenKind::Operator(operator) = &binary.operator.kind {
                    operator
                } else {
                    return Err(AnalyzeError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                };

                let kind = match op_kind.as_slice() {
                    [OperatorKind::Dot] => {
                        let target = binary.left.analyze(resolver)?;
                        let member = binary.right.analyze(resolver)?;
                        AnalysisKind::Access(Box::new(target), Box::new(member))
                    }

                    [OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let value = binary.right.analyze(resolver)?;
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Plus, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::Add(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Minus, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::Subtract(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Star, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::Multiply(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Slash, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::Divide(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Percent, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::Modulus(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Ampersand, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::BitwiseAnd(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Pipe, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::BitwiseOr(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::Caret, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::LogicalXOr(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::LeftAngle, OperatorKind::LeftAngle, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::ShiftLeft(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }
                    [OperatorKind::RightAngle, OperatorKind::RightAngle, OperatorKind::Equal] => {
                        let target = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        let value = Analysis::new(AnalysisKind::ShiftRight(Box::new(target.clone()), Box::new(right)), self.span, typing.clone());
                        mutate(target, value, &binary.operator)?
                    }

                    _ => {
                        let left = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;

                        match op_kind.as_slice() {
                            [OperatorKind::Plus] => {
                                match (&left.typing.kind, &right.typing.kind) {
                                    (TypeKind::Pointer { target }, _) => {
                                        let size = Analysis::new(AnalysisKind::SizeOf((**target).clone()), right.span, right.typing.clone());
                                        let scale = Analysis::new(AnalysisKind::Multiply(Box::new(right.clone()), Box::new(size)), right.span, right.typing.clone());
                                        AnalysisKind::Add(Box::new(left), Box::new(scale))
                                    }
                                    (_, TypeKind::Pointer { target }) => {
                                        let size = Analysis::new(AnalysisKind::SizeOf((**target).clone()), left.span, left.typing.clone());
                                        let scale = Analysis::new(AnalysisKind::Multiply(Box::new(left.clone()), Box::new(size)), left.span, left.typing.clone());
                                        AnalysisKind::Add(Box::new(scale), Box::new(right))
                                    }
                                    _ => AnalysisKind::Add(Box::new(left), Box::new(right))
                                }
                            },
                            [OperatorKind::Minus] => {
                                match (&left.typing.kind, &right.typing.kind) {
                                    (TypeKind::Pointer { target: left_target }, TypeKind::Pointer { target: right_target }) => {
                                        if left_target == right_target {
                                            let size = Analysis::new(AnalysisKind::SizeOf((**left_target).clone()), self.span, typing.clone());
                                            let difference = Analysis::new(AnalysisKind::Subtract(Box::new(left), Box::new(right)), self.span, typing.clone());
                                            AnalysisKind::Divide(Box::new(difference), Box::new(size))
                                        } else {
                                            return Err(AnalyzeError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                                        }
                                    }
                                    (TypeKind::Pointer { target }, _) => {
                                        let size = Analysis::new(AnalysisKind::SizeOf((**target).clone()), right.span, right.typing.clone());
                                        let scale = Analysis::new(AnalysisKind::Multiply(Box::new(right.clone()), Box::new(size)), right.span, right.typing.clone());
                                        AnalysisKind::Subtract(Box::new(left), Box::new(scale))
                                    }
                                    _ => AnalysisKind::Subtract(Box::new(left), Box::new(right))
                                }
                            },
                            [OperatorKind::Star] => AnalysisKind::Multiply(Box::new(left), Box::new(right)),
                            [OperatorKind::Slash] => AnalysisKind::Divide(Box::new(left), Box::new(right)),
                            [OperatorKind::Percent] => AnalysisKind::Modulus(Box::new(left), Box::new(right)),

                            [OperatorKind::Ampersand, OperatorKind::Ampersand] => AnalysisKind::LogicalAnd(Box::new(left), Box::new(right)),
                            [OperatorKind::Pipe, OperatorKind::Pipe] => AnalysisKind::LogicalOr(Box::new(left), Box::new(right)),
                            [OperatorKind::Caret] => AnalysisKind::LogicalXOr(Box::new(left), Box::new(right)),
                            [OperatorKind::Ampersand] => AnalysisKind::BitwiseAnd(Box::new(left), Box::new(right)),
                            [OperatorKind::Pipe] => AnalysisKind::BitwiseOr(Box::new(left), Box::new(right)),
                            [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => AnalysisKind::ShiftLeft(Box::new(left), Box::new(right)),
                            [OperatorKind::RightAngle, OperatorKind::RightAngle] => AnalysisKind::ShiftRight(Box::new(left), Box::new(right)),

                            [OperatorKind::Equal, OperatorKind::Equal] => AnalysisKind::Equal(Box::new(left), Box::new(right)),
                            [OperatorKind::Exclamation, OperatorKind::Equal] => AnalysisKind::NotEqual(Box::new(left), Box::new(right)),
                            [OperatorKind::LeftAngle] => AnalysisKind::Less(Box::new(left), Box::new(right)),
                            [OperatorKind::LeftAngle, OperatorKind::Equal] => AnalysisKind::LessOrEqual(Box::new(left), Box::new(right)),
                            [OperatorKind::RightAngle] => AnalysisKind::Greater(Box::new(left), Box::new(right)),
                            [OperatorKind::RightAngle, OperatorKind::Equal] => AnalysisKind::GreaterOrEqual(Box::new(left), Box::new(right)),
                            _ => return Err(AnalyzeError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span))
                        }
                    }
                };

                Ok(Analysis::new(kind, self.span, typing))
            },

            ElementKind::Index(index) => {
                let target = index.target.analyze(resolver)?;
                let indexes: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = index
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();

                let kind = AnalysisKind::Index(Index::new(
                    Box::new(target),
                    indexes?,
                ));
                Ok(Analysis::new(kind, self.span, typing))
            }

            ElementKind::Invoke(invoke) => {
                let name = invoke.target.brand().and_then(|token| {
                    if let TokenKind::Identifier(name) = token.kind { Some(name) } else { None }
                });

                let kind = match name.as_ref().and_then(|s| s.as_str()) {
                    Some("if") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let then = invoke.members[1].analyze(resolver)?;
                        let otherwise = invoke.members[2].analyze(resolver)?;

                        AnalysisKind::Conditional(
                            Box::new(condition),
                            Box::new(then),
                            Some(Box::new(otherwise)),
                        )
                    }
                    Some("while") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let body = invoke.members[1].analyze(resolver)?;

                        AnalysisKind::While(Box::new(condition), Box::new(body))
                    }
                    Some("break") => {
                        let value = if !invoke.members.is_empty() {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else { None };

                        AnalysisKind::Break(value)
                    }
                    Some("continue") => {
                        let value = if !invoke.members.is_empty() {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else { None };

                        AnalysisKind::Continue(value)
                    }
                    Some("return") => {
                        let value = if !invoke.members.is_empty() {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else { None };

                        AnalysisKind::Return(value)
                    }
                    _ => {
                        let target = if let ElementKind::Literal(literal) = &invoke.target.kind {
                            if let TokenKind::Identifier(name) = literal.kind { name } else {
                                return Err(AnalyzeError::new(ErrorKind::InvalidTarget, literal.span));
                            }
                        } else {
                            return Err(AnalyzeError::new(ErrorKind::InvalidTarget, invoke.target.span));
                        };

                        let arguments: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = invoke
                            .members
                            .iter()
                            .map(|member| member.analyze(resolver))
                            .collect();

                        AnalysisKind::Invoke(Invoke::new(target, arguments?))
                    }
                };

                Ok(Analysis::new(kind, self.span, typing))
            },

            ElementKind::Construct(constructor) => {
                let target = constructor
                    .target
                    .brand()
                    .map(|s| s.format(1))
                    .unwrap_or_default();

                let members: Vec<Analysis<'element>> = constructor
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect::<Result<Vec<Analysis<'element>>, AnalyzeError<'element>>>()?;

                let analyzed = Aggregate::new(Str::from(target), members);
                Ok(Analysis::new(AnalysisKind::Constructor(analyzed), self.span, typing))
            }

            ElementKind::Symbolize(symbol) => {
                symbol.analyze(resolver)
            },
        }
    }
}
