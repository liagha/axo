use {
    crate::{
        data::*,
        analyzer::{Analyzable, Analysis, AnalysisKind, AnalyzeError, ErrorKind},
        format::Show,
        parser::{Element, ElementKind},
        resolver::Resolver,
        scanner::{OperatorKind, PunctuationKind, TokenKind},
    },
};

impl<'element> Analyzable<'element> for Element<'element> {
    fn analyze(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Analysis<'element>, AnalyzeError<'element>> {
        match &self.kind {
            ElementKind::Literal(literal) => literal.analyze(resolver),

            ElementKind::Delimited(delimited) => {
                let kind = match (
                    &delimited.start.kind,
                    delimited.separator.as_ref().map(|token| &token.kind),
                    &delimited.end.kind,
                ) {
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    )
                    | (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
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
                        let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> =
                            delimited
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
                        // A single item in parentheses is just a grouped expression, not a tuple.
                        if delimited.members.len() == 1 {
                            return delimited.members[0].analyze(resolver);
                        } else {
                            let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> =
                                delimited
                                    .members
                                    .iter()
                                    .map(|item| item.analyze(resolver))
                                    .collect();

                            AnalysisKind::Tuple(items?)
                        }
                    }

                    _ => return Err(AnalyzeError::new(ErrorKind::Unimplemented, self.span)),
                };
                Ok(Analysis::new(kind, self.span))
            }

            ElementKind::Unary(unary) => {
                if let TokenKind::Operator(operator) = &unary.operator.kind {
                    let operand = unary.operand.analyze(resolver)?;

                    let kind = match operator.as_slice() {
                        [OperatorKind::Exclamation] => AnalysisKind::LogicalNot(Box::new(operand)),
                        [OperatorKind::Tilde] => AnalysisKind::BitwiseNot(Box::new(operand)),
                        [OperatorKind::Plus] => return Ok(operand), // '+' is a no-op, return operand as is.
                        [OperatorKind::Minus] => {
                            // Create a zero literal with the span of the '-' operator.
                            let zero = Analysis::new(
                                AnalysisKind::Integer {
                                    value: 0,
                                    size: 64,
                                    signed: true,
                                },
                                unary.operator.span,
                            );
                            AnalysisKind::Subtract(Box::new(zero), Box::new(operand))
                        }
                        [OperatorKind::Ampersand] => AnalysisKind::AddressOf(Box::new(operand)),
                        [OperatorKind::Star] => AnalysisKind::Dereference(Box::new(operand)),
                        _ => {
                            return Err(AnalyzeError::new(
                                ErrorKind::InvalidOperation(unary.operator.clone()),
                                unary.operator.span,
                            ))
                        }
                    };
                    
                    return Ok(Analysis::new(kind, self.span));
                }

                Err(AnalyzeError::new(
                    ErrorKind::InvalidOperation(unary.operator.clone()),
                    unary.operator.span,
                ))
            },

            ElementKind::Binary(binary) => {
                let op_kind = if let TokenKind::Operator(operator) = &binary.operator.kind {
                    operator
                } else {
                    return Err(AnalyzeError::new(
                        ErrorKind::InvalidOperation(binary.operator.clone()),
                        binary.operator.span,
                    ));
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

                        match &target.kind {
                            AnalysisKind::Usage(target_name) => {
                                AnalysisKind::Assign(target_name.clone(), Box::new(value))
                            }
                            AnalysisKind::Dereference(_) => {
                                AnalysisKind::Store(Box::new(target), Box::new(value))
                            }
                            t => {
                                println!("---- {:?}", t);
                                return Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                    }

                    // A helper macro could reduce this repetition, but for clarity:
                    _ => {
                        let left = binary.left.analyze(resolver)?;
                        let right = binary.right.analyze(resolver)?;
                        match op_kind.as_slice() {
                            [OperatorKind::Plus] => AnalysisKind::Add(Box::new(left), Box::new(right)),
                            [OperatorKind::Minus] => AnalysisKind::Subtract(Box::new(left), Box::new(right)),
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
                            _ => {
                                return Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                    }
                };

                Ok(Analysis::new(kind, self.span))
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
                Ok(Analysis::new(kind, self.span))
            }

            ElementKind::Invoke(invoke) => {
                let name = invoke.target.brand().and_then(|token| {
                    if let TokenKind::Identifier(name) = token.kind {
                        Some(name)
                    } else {
                        None
                    }
                });

                let kind = match name.as_ref().and_then(|s| s.as_str()) {
                    Some("if") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let then = invoke.members[1].analyze(resolver)?;
                        let otherwise = invoke.members[2].analyze(resolver)?;

                        AnalysisKind::Conditional(
                            Box::new(condition),
                            Box::new(then),
                            Box::new(otherwise),
                        )
                    }
                    Some("while") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let body = invoke.members[1].analyze(resolver)?;

                        AnalysisKind::While(
                            Box::new(condition),
                            Box::new(body),
                        )
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
                    _ => {
                        let target = if let ElementKind::Literal(literal) = &invoke.target.kind {
                            if let TokenKind::Identifier(name) = literal.kind {
                                name
                            } else { 
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

                        AnalysisKind::Invoke(Invoke::new(
                            target,
                            arguments?,
                        ))
                    }
                };
                
                Ok(Analysis::new(kind, self.span))
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

                let analyzed = Structure::new(Str::from(target), members);

                Ok(Analysis::new(AnalysisKind::Constructor(analyzed), self.span))
            }

            ElementKind::Symbolize(symbol) => symbol.analyze(resolver),
        }
    }
}