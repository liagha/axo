use {
    crate::{
        data::*,
        analyzer::{Analyzable, Analysis, AnalyzeError, ErrorKind},
        format::Show,
        parser::{Element, ElementKind},
        resolver::Resolver,
        scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    },
};

fn primitive<'element>(target: &Element<'element>) -> Option<&'element str> {
    let token = target.brand()?;
    match token.kind {
        TokenKind::Identifier(identifier) => identifier.as_str(),
        _ => None,
    }
}

fn arity<'element>(
    name: &str,
    members: usize,
    expected: &str,
    valid: bool,
    span: crate::tracker::Span<'element>,
) -> Result<(), AnalyzeError<'element>> {
    if valid {
        Ok(())
    } else {
        Err(AnalyzeError::new(
            ErrorKind::InvalidPrimitiveArity {
                name: name.to_string(),
                expected: expected.to_string(),
                found: members,
            },
            span,
        ))
    }
}

impl<'element> Analyzable<'element> for Element<'element> {
    fn analyze(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Analysis<'element>, AnalyzeError<'element>> {
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
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) => {
                        let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = delimited
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
                        let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> =
                            delimited
                                .members
                                .iter()
                                .map(|item| item.analyze(resolver))
                                .collect();

                        Ok(Analysis::Array(items?))
                    }
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        if delimited.members.len() == 1 {
                            delimited.members[0].analyze(resolver)
                        } else {
                            let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> =
                                delimited
                                    .members
                                    .iter()
                                    .map(|item| item.analyze(resolver))
                                    .collect();
                            
                            Ok(Analysis::Tuple(items?))
                        }
                    }
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        Some(_),
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> =
                            delimited
                                .members
                                .iter()
                                .map(|item| item.analyze(resolver))
                                .collect();

                        Ok(Analysis::Tuple(items?))
                    }

                    _ => Err(AnalyzeError::new(ErrorKind::Unimplemented, self.span)),
                }
            }

            ElementKind::Unary(unary) => unary.analyze(resolver),

            ElementKind::Binary(item) => item.analyze(resolver),

            ElementKind::Index(index) => {
                let target = index.target.analyze(resolver)?;
                let indexes: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = index
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
                let name = primitive(&invoke.target);

                match name {
                    Some("if") => {
                        arity(
                            "if",
                            invoke.members.len(),
                            "3 arguments",
                            invoke.members.len() == 3,
                            self.span,
                        )?;

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
                        arity(
                            "while",
                            invoke.members.len(),
                            "2 arguments",
                            invoke.members.len() == 2,
                            self.span,
                        )?;

                        let loop_context = resolver.cycle = true;
                        let condition = invoke.members[0].analyze(resolver)?;
                        let body = invoke.members[1].analyze(resolver)?;

                        Ok(Analysis::While(
                            Box::new(condition),
                            Box::new(body),
                        ))
                    }
                    Some("for") => {
                        arity(
                            "for",
                            invoke.members.len(),
                            "4 arguments",
                            invoke.members.len() == 4,
                            self.span,
                        )?;

                        let init = invoke.members[0].analyze(resolver)?;
                        let condition = invoke.members[1].analyze(resolver)?;
                        let step = invoke.members[2].analyze(resolver)?;
                        let body = invoke.members[3].analyze(resolver)?;

                        let while_body = Analysis::Block(vec![body, step]);
                        Ok(Analysis::Block(vec![
                            init,
                            Analysis::While(
                                Box::new(condition),
                                Box::new(while_body),
                            ),
                        ]))
                    }
                    Some("break") => {
                        arity(
                            "break",
                            invoke.members.len(),
                            "0 arguments",
                            invoke.members.is_empty(),
                            self.span,
                        )?;
                        if !resolver.cycle {
                            return Err(AnalyzeError::new(
                                ErrorKind::InvalidPrimitiveContext {
                                    name: "break".to_string(),
                                    expected: "inside loop body".to_string(),
                                },
                                self.span,
                            ));
                        }
                        Ok(Analysis::Break(None))
                    }
                    Some("continue") => {
                        arity(
                            "continue",
                            invoke.members.len(),
                            "0 arguments",
                            invoke.members.is_empty(),
                            self.span,
                        )?;

                        if !resolver.cycle {
                            return Err(AnalyzeError::new(
                                ErrorKind::InvalidPrimitiveContext {
                                    name: "continue".to_string(),
                                    expected: "inside loop body".to_string(),
                                },
                                self.span,
                            ));
                        }

                        Ok(Analysis::Continue(None))
                    }
                    Some("return") => {
                        arity(
                            "return",
                            invoke.members.len(),
                            "0 or 1 arguments",
                            invoke.members.len() <= 1,
                            self.span,
                        )?;
                        if !resolver.method {
                            return Err(AnalyzeError::new(
                                ErrorKind::InvalidPrimitiveContext {
                                    name: "return".to_string(),
                                    expected: "inside function body".to_string(),
                                },
                                self.span,
                            ));
                        }
                        let value = if invoke.members.is_empty() {
                            None
                        } else {
                            Some(Box::new(invoke.members[0].analyze(resolver)?))
                        };
                        Ok(Analysis::Return(value))
                    }
                    _ => {
                        let target = invoke.target.analyze(resolver)?;
                        
                        let arguments: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = invoke
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
                    .collect::<Result<Vec<Analysis<'element>>, AnalyzeError<'element>>>()?;

                match target.as_str().unwrap() {
                    "Integer" => {
                        let mut value_opt = None;
                        let mut size_opt = None;
                        let mut signed_opt = None;

                        for member in &members {
                            if let Analysis::Assign(field_name, field_value) = &member {
                                match field_name.as_str().unwrap() {
                                    "value" => {
                                        if let Analysis::Integer { value, .. } =
                                            &**field_value
                                        {
                                            value_opt = Some(value.clone());
                                        }
                                    }
                                    "size" => {
                                        if let Analysis::Integer { value: size, .. } =
                                            &**field_value
                                        {
                                            size_opt = Some(*size);
                                        }
                                    }
                                    "signed" => {
                                        if let Analysis::Boolean { value: signed } =
                                            &**field_value
                                        {
                                            signed_opt = Some(*signed);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        match (value_opt, size_opt, signed_opt) {
                            (Some(value), Some(size), Some(signed)) => {
                                Ok(Analysis::Integer {
                                    value,
                                    size: size.try_into().unwrap(),
                                    signed,
                                })
                            }
                            _ => Err(AnalyzeError::new(
                                ErrorKind::InvalidType,
                                constructor.target.span,
                            )),
                        }
                    }
                    "Float" => {
                        let mut value_opt = None;
                        let mut size_opt = None;

                        for member in &members {
                            if let Analysis::Assign(field_name, field_value) = &member {
                                match field_name.as_str().unwrap() {
                                    "value" => {
                                        if let Analysis::Float { value, .. } =
                                            &**field_value
                                        {
                                            value_opt = Some(value.clone());
                                        }
                                    }
                                    "size" => {
                                        if let Analysis::Integer { value: size, .. } =
                                            &**field_value
                                        {
                                            size_opt = Some(*size);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        match (value_opt, size_opt) {
                            (Some(value), Some(size)) => Ok(Analysis::Float {
                                value,
                                size: size.try_into().unwrap(),
                            }),
                            _ => Err(AnalyzeError::new(
                                ErrorKind::InvalidType,
                                constructor.target.span,
                            )),
                        }
                    }
                    "Boolean" => {
                        let mut value_opt = None;

                        for member in &members {
                            if let Analysis::Assign(field_name, field_value) = &member {
                                if field_name.as_str().unwrap() == "value" {
                                    if let Analysis::Boolean { value } = &**field_value {
                                        value_opt = Some(*value);
                                    }
                                }
                            }
                        }

                        match value_opt {
                            Some(value) => Ok(Analysis::Boolean { value }),
                            _ => Err(AnalyzeError::new(
                                ErrorKind::InvalidType,
                                constructor.target.span,
                            )),
                        }
                    }
                    _ => {
                        let analyzed = Structure::new(Str::from(target), members);
                        Ok(Analysis::Constructor(analyzed))
                    }
                }
            }

            ElementKind::Symbolize(symbol) => symbol.analyze(resolver),
        }

    }
}

impl<'binary> Analyzable<'binary> for Binary<Box<Element<'binary>>, Token<'binary>, Box<Element<'binary>>> {
    fn analyze(&self, resolver: &mut Resolver<'binary>) -> Result<Analysis<'binary>, AnalyzeError<'binary>> {
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
                        _ => Err(AnalyzeError::new(
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

                _ => Err(AnalyzeError::new(
                    ErrorKind::InvalidOperation(self.operator.clone()),
                    self.operator.span,
                )),
            }
        } else {
            Err(AnalyzeError::new(
                ErrorKind::InvalidOperation(self.operator.clone()),
                self.operator.span,
            ))
        }

    }
}

impl<'unary> Analyzable<'unary> for Unary<Token<'unary>, Box<Element<'unary>>> {
    fn analyze(&self, resolver: &mut Resolver<'unary>) -> Result<Analysis<'unary>, AnalyzeError<'unary>> {
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
                _ => Err(AnalyzeError::new(
                    ErrorKind::InvalidOperation(self.operator.clone()),
                    self.operator.span,
                )),
            };
        }

        Err(AnalyzeError::new(
            ErrorKind::InvalidOperation(self.operator.clone()),
            self.operator.span,
        ))
    }
}
