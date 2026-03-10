use {
    crate::{
        data::*,
        analyzer::{Analyzable, Analysis, CheckError, ErrorKind},
        format::Show,
        parser::{Element, ElementKind},
        resolver::Resolver,
        scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    },
};

impl<'element> Analyzable<'element> for Element<'element> {
    fn analyze(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Analysis<'element>, CheckError<'element>> {
        match &self.kind {
            ElementKind::Literal(literal) => literal.analyze(resolver),

            ElementKind::Delimited(delimited) => {
                match (
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
                        let items: Result<Vec<Analysis<'element>>, CheckError<'element>> = delimited
                            .members
                            .iter()
                            .map(|item| item.analyze(resolver))
                            .collect();

                        Ok(Analysis::Block(items?))
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                        _,
                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                    ) => {
                        let items: Result<Vec<Analysis<'element>>, CheckError<'element>> =
                            delimited
                                .members
                                .iter()
                                .map(|item| item.analyze(resolver))
                                .collect();

                        Ok(Analysis::Array(items?))
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        _,
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        if delimited.members.len() == 1 {
                            delimited.members[0].analyze(resolver)
                        } else {
                            let items: Result<Vec<Analysis<'element>>, CheckError<'element>> =
                                delimited
                                    .members
                                    .iter()
                                    .map(|item| item.analyze(resolver))
                                    .collect();
                            
                            Ok(Analysis::Tuple(items?))
                        }
                    }

                    _ => Err(CheckError::new(ErrorKind::Unimplemented, self.span)),
                }
            }

            ElementKind::Unary(unary) => unary.analyze(resolver),

            ElementKind::Binary(item) => item.analyze(resolver),

            ElementKind::Index(index) => {
                let target = index.target.analyze(resolver)?;
                let indexes: Result<Vec<Analysis<'element>>, CheckError<'element>> = index
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver))
                    .collect();
                
                Ok(Analysis::Index(Index::new(
                    Box::new(target),
                    indexes?,
                )))
            }

            ElementKind::Invoke(invoke) => {
                let name = if let Some(TokenKind::Identifier(name)) = invoke.target.brand().map(|token| token.kind.clone()) {
                    name
                } else {
                    unimplemented!("expected the head to be Identifier.")
                };

                match name.as_str() {
                    Some("if") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let then = invoke.members[1].analyze(resolver)?;
                        let otherwise = invoke.members[2].analyze(resolver)?;

                        Ok(Analysis::Conditional(
                            Box::new(condition),
                            Box::new(then),
                            Box::new(otherwise),
                        ))
                    }
                    Some("while") => {
                        let condition = invoke.members[0].analyze(resolver)?;
                        let body = invoke.members[1].analyze(resolver)?;

                        Ok(Analysis::While(
                            Box::new(condition),
                            Box::new(body),
                        ))
                    }
                    Some("break") => {
                        let value = if invoke.members.len() == 1 {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else {
                            None
                        };

                        Ok(Analysis::Break(value))
                    }
                    Some("continue") => {
                        let value = if invoke.members.len() == 1 {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else {
                            None
                        };

                        Ok(Analysis::Continue(value))
                    }
                    Some("return") => {
                        let value = if invoke.members.len() == 1 {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        } else {
                            None
                        };

                        Ok(Analysis::Return(value))
                    }
                    _ => {
                        let target = invoke.target.analyze(resolver)?;
                        
                        let arguments: Result<Vec<Analysis<'element>>, CheckError<'element>> = invoke
                            .members
                            .iter()
                            .map(|member| member.analyze(resolver))
                            .collect();
                        
                        Ok(Analysis::Invoke(Invoke::new(
                            Box::new(target),
                            arguments?,
                        )))
                    }
                }
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
                    .collect::<Result<Vec<Analysis<'element>>, CheckError<'element>>>()?;

                let analyzed = Structure::new(Str::from(target), members);
                
                Ok(Analysis::Constructor(analyzed))
            }

            ElementKind::Symbolize(symbol) => symbol.analyze(resolver),
        }

    }
}

impl<'binary> Analyzable<'binary> for Binary<Box<Element<'binary>>, Token<'binary>, Box<Element<'binary>>> {
    fn analyze(&self, resolver: &mut Resolver<'binary>) -> Result<Analysis<'binary>, CheckError<'binary>> {
        if let TokenKind::Operator(operator) = &self.operator.kind {
            match operator.as_slice() {
                [OperatorKind::Dot] => {
                    let target = self.left.analyze(resolver)?;
                    let member = self.right.analyze(resolver)?;

                    Ok(Analysis::Access(
                        Box::new(target),
                        Box::new(member),
                    ))
                }

                [OperatorKind::Equal] => {
                    let target = self.left.analyze(resolver)?;
                    let value = self.right.analyze(resolver)?;

                    match &target {
                        Analysis::Usage(target_name) => Ok(Analysis::Assign(
                            target_name.clone(),
                            Box::new(value),
                        )),
                        Analysis::Dereference(_) => Ok(Analysis::Store(
                            Box::new(target),
                            Box::new(value),
                        )),
                        _ => Err(CheckError::new(
                            ErrorKind::InvalidOperation(self.operator.clone()),
                            self.operator.span,
                        )),
                    }
                }

                [OperatorKind::Plus] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::Add(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Minus] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::Subtract(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Star] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::Multiply(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Slash] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::Divide(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Percent] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::Modulus(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Ampersand, OperatorKind::Ampersand] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::LogicalAnd(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Pipe, OperatorKind::Pipe] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::LogicalOr(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Caret] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::LogicalXOr(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Ampersand] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::BitwiseAnd(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Pipe] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::BitwiseOr(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;

                    Ok(Analysis::ShiftLeft(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::ShiftRight(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Equal, OperatorKind::Equal] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::Equal(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::Exclamation, OperatorKind::Equal] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::NotEqual(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::LeftAngle] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::Less(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::LeftAngle, OperatorKind::Equal] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::LessOrEqual(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::RightAngle] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::Greater(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                [OperatorKind::RightAngle, OperatorKind::Equal] => {
                    let left = self.left.analyze(resolver)?;
                    let right = self.right.analyze(resolver)?;
                    Ok(Analysis::GreaterOrEqual(
                        Box::new(left),
                        Box::new(right),
                    ))
                }

                _ => Err(CheckError::new(
                    ErrorKind::InvalidOperation(self.operator.clone()),
                    self.operator.span,
                )),
            }
        } else {
            Err(CheckError::new(
                ErrorKind::InvalidOperation(self.operator.clone()),
                self.operator.span,
            ))
        }

    }
}

impl<'unary> Analyzable<'unary> for Unary<Token<'unary>, Box<Element<'unary>>> {
    fn analyze(&self, resolver: &mut Resolver<'unary>) -> Result<Analysis<'unary>, CheckError<'unary>> {
        if let TokenKind::Operator(operator) = &self.operator.kind {
            let operand = self.operand.analyze(resolver)?;

            return match operator.as_slice() {
                [OperatorKind::Exclamation] => {
                    Ok(Analysis::LogicalNot(Box::new(operand)))
                }
                [OperatorKind::Tilde] => Ok(Analysis::BitwiseNot(Box::new(operand))),
                [OperatorKind::Plus] => Ok(operand),
                [OperatorKind::Minus] => Ok(Analysis::Subtract(
                    Box::new(Analysis::Integer {
                        value: 0,
                        size: 64,
                        signed: true,
                    }),
                    Box::new(operand),
                )),
                [OperatorKind::Ampersand] => {
                    Ok(Analysis::AddressOf(Box::new(operand)))
                }
                [OperatorKind::Star] => Ok(Analysis::Dereference(Box::new(operand))),
                _ => Err(CheckError::new(
                    ErrorKind::InvalidOperation(self.operator.clone()),
                    self.operator.span,
                )),
            };
        }

        Err(CheckError::new(
            ErrorKind::InvalidOperation(self.operator.clone()),
            self.operator.span,
        ))
    }
}
