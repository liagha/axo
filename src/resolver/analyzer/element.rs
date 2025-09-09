use crate::data::Str;
use crate::parser::{Element, ElementKind};
use crate::resolver::analyzer::{Analysis, Analyzable, AnalyzeError, ErrorKind, Instruction};
use crate::resolver::Resolver;
use crate::scanner::{OperatorKind, PunctuationKind, TokenKind};
use crate::schema::{Conditional, Cycle, Index, Invoke, Structure, While};

impl<'element> Analyzable<'element> for Element<'element> {
    fn analyze(&self, resolver: &Resolver<'element>) -> Result<Analysis<'element>, AnalyzeError<'element>> {
        match &self.kind {
            ElementKind::Literal(literal) => literal.analyze(resolver),
            ElementKind::Closure(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
            }
            ElementKind::Delimited(delimited) => {
                match &self.kind {
                    ElementKind::Delimited(delimited) => {
                        match (&delimited.start.kind, delimited.separator.as_ref().map(|token| &token.kind), &delimited.end.kind) {
                            (
                                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                                None,
                                TokenKind::Punctuation(PunctuationKind::RightBrace),
                            ) | (
                                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                                Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                                TokenKind::Punctuation(PunctuationKind::RightBrace),
                            ) => {
                                let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = delimited
                                    .items
                                    .iter()
                                    .map(|item| item.analyze(resolver))
                                    .collect();

                                Ok(Analysis::new(Instruction::Block(items?)))
                            }

                            _ => {
                                Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
                            }
                        }
                    }
                    _ => {
                        Err(AnalyzeError::new(ErrorKind::UnImplemented, self.span))
                    }
                }
            }
            ElementKind::Unary(unary) => {
                if let TokenKind::Operator(operator) = &unary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Exclamation] => {
                            let operand = unary.operand.analyze(resolver)?;
                            if operand.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalNot(Box::new(operand))))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(unary.operator.clone()),
                                    unary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Tilde] => {
                            let operand = unary.operand.analyze(resolver)?;
                            if operand.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseNot(Box::new(operand))))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(unary.operator.clone()),
                                    unary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Minus] => {
                            let operand = unary.operand.analyze(resolver)?;
                            if let Instruction::Integer { value, size, signed } = operand.instruction {
                                Ok(Analysis::new(Instruction::Integer { value: -value, size, signed }))
                            } else if let Instruction::Float { value, size } = operand.instruction {
                                Ok(Analysis::new(Instruction::Float { value: -value, size }))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(unary.operator.clone()),
                                    unary.operator.span,
                                ))
                            }
                        }
                        _ => Err(AnalyzeError::new(
                            ErrorKind::InvalidOperation(unary.operator.clone()),
                            unary.operator.span,
                        )),
                    }
                } else {
                    Err(AnalyzeError::new(
                        ErrorKind::InvalidOperation(unary.operator.clone()),
                        unary.operator.span,
                    ))
                }
            }
            ElementKind::Binary(binary) => {
                if let TokenKind::Operator(operator) = &binary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Dot] => {
                            let target = binary.left.analyze(resolver)?;
                            let member = binary.right.analyze(resolver)?;
                            
                            if let Instruction::Invoke(invoke) = &member.instruction {
                                if let Instruction::Usage(method_name) = &invoke.target.instruction {
                                    if target.instruction.is_value() {
                                        match method_name.as_str().unwrap() {
                                            "add" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Add(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "subtract" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Subtract(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "multiply" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Multiply(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "divide" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Divide(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "modulus" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Modulus(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "and" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::LogicalAnd(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "or" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::LogicalOr(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "xor" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::LogicalXOr(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "bitwise_and" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::BitwiseAnd(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "bitwise_or" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::BitwiseOr(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "shift_left" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::ShiftLeft(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "shift_right" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::ShiftRight(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "equal" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Equal(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "not_equal" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::NotEqual(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "less" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Less(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "less_or_equal" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::LessOrEqual(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "greater" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::Greater(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "greater_or_equal" if invoke.members.len() == 1 => {
                                                if invoke.members[0].instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::GreaterOrEqual(
                                                        Box::new(target),
                                                        Box::new(*invoke.members[0].clone()),
                                                    )));
                                                }
                                            }
                                            "not" if invoke.members.is_empty() => {
                                                if target.instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::LogicalNot(Box::new(target))));
                                                }
                                            }
                                            "negate" if invoke.members.is_empty() => {
                                                if let Instruction::Integer { value, size, signed } = target.instruction {
                                                    return Ok(Analysis::new(Instruction::Integer { value: -value, size, signed }));
                                                } else if let Instruction::Float { value, size } = target.instruction {
                                                    return Ok(Analysis::new(Instruction::Float { value: -value, size }));
                                                }
                                            }
                                            "bitwise_not" if invoke.members.is_empty() => {
                                                if target.instruction.is_value() {
                                                    return Ok(Analysis::new(Instruction::BitwiseNot(Box::new(target))));
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Ok(Analysis::new(Instruction::Access(
                                Box::new(target),
                                Box::new(member),
                            )))
                        }
                        [OperatorKind::Equal] => {
                            let target = binary.left.analyze(resolver)?;
                            let value = binary.right.analyze(resolver)?;
                            if let Instruction::Usage(target_name) = &target.instruction {
                                Ok(Analysis::new(Instruction::Assign(
                                    target_name.clone(),
                                    Box::new(value),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Plus] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Add(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Minus] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Subtract(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Star] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Multiply(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Slash] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Divide(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Percent] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Modulus(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Ampersand, OperatorKind::Ampersand] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalAnd(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Pipe, OperatorKind::Pipe] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalOr(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Ampersand] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseAnd(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Pipe] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseOr(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Caret] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalXOr(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::ShiftLeft(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::ShiftRight(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Equal, OperatorKind::Equal] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Equal(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::Exclamation, OperatorKind::Equal] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::NotEqual(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::LeftAngle] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Less(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::LeftAngle, OperatorKind::Equal] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LessOrEqual(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::RightAngle] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Greater(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        [OperatorKind::RightAngle, OperatorKind::Equal] => {
                            let left = binary.left.analyze(resolver)?;
                            let right = binary.right.analyze(resolver)?;
                            if left.instruction.is_value() && right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::GreaterOrEqual(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.operator.span,
                                ))
                            }
                        }
                        _ => Err(AnalyzeError::new(
                            ErrorKind::InvalidOperation(binary.operator.clone()),
                            binary.operator.span,
                        )),
                    }
                } else {
                    Err(AnalyzeError::new(
                        ErrorKind::InvalidOperation(binary.operator.clone()),
                        binary.operator.span,
                    ))
                }
            }
            ElementKind::Index(index) => {
                let target = index.target.analyze(resolver)?;
                let indexes: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> = index
                    .members
                    .iter()
                    .map(|member| member.analyze(resolver).map(Box::new))
                    .collect();
                Ok(Analysis::new(Instruction::Index(Index::new(
                    Box::new(target),
                    indexes?,
                ))))
            }
            ElementKind::Invoke(invoke) => {
                let target = invoke.target.analyze(resolver)?;
                let arguments: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> =
                    invoke
                        .members
                        .iter()
                        .map(|member| member.analyze(resolver).map(Box::new))
                        .collect();
                Ok(Analysis::new(Instruction::Invoke(Invoke::new(
                    Box::new(target),
                    arguments?,
                ))))
            }
            ElementKind::Construct(constructor) => {
                let target = constructor
                    .target
                    .brand()
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                let members: Vec<Box<Analysis<'element>>> =
                    constructor
                        .members
                        .iter()
                        .map(|member| member.analyze(resolver).map(Box::new))
                        .collect::<
                            Result<
                                Vec<Box<Analysis<'element>>>,
                                AnalyzeError<'element>,
                            >
                        >()?;

                match target.as_str() {
                    "Integer" => {
                        if members.len() == 3 {
                            if let (
                                Instruction::Integer { value, .. },
                                Instruction::Integer { size, .. },
                                Instruction::Boolean { value: signed }
                            ) = (&members[0].instruction, &members[1].instruction, &members[2].instruction) {
                                Ok(
                                    Analysis::new(
                                        Instruction::Integer {
                                            value: value.clone(),
                                            size: (*size).try_into().unwrap(),
                                            signed: signed.clone(),
                                        }
                                    )
                                )
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidType,
                                    constructor.target.span,
                                ))
                            }
                        } else {
                            Err(AnalyzeError::new(
                                ErrorKind::InvalidType,
                                constructor.target.span,
                            ))
                        }
                    }
                    "Float" => {
                        if members.len() == 2 {
                            if let (Instruction::Float { value, .. }, Instruction::Integer { size, .. }) = (&members[0].instruction, &members[1].instruction) {
                                Ok(
                                    Analysis::new(
                                        Instruction::Float {
                                            value: value.clone(),
                                            size: (*size).try_into().unwrap()
                                        }
                                    )
                                )
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidType,
                                    constructor.target.span,
                                ))
                            }
                        } else {
                            Err(AnalyzeError::new(
                                ErrorKind::InvalidType,
                                constructor.target.span,
                            ))
                        }
                    }
                    "Boolean" => {
                        if members.len() == 1 {
                            if let Instruction::Boolean { value } = &members[0].instruction {
                                Ok(Analysis::new(Instruction::Boolean { value: value.clone() }))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidType,
                                    constructor.target.span,
                                ))
                            }
                        } else {
                            Err(AnalyzeError::new(
                                ErrorKind::InvalidType,
                                constructor.target.span,
                            ))
                        }
                    }
                    _ => {
                        let analyzed = Structure::new(Str::from(target), members);
                        Ok(Analysis::new(Instruction::Constructor(analyzed)))
                    }
                }
            }
            ElementKind::Conditional(conditional) => {
                let condition = conditional.guard.analyze(resolver)?;
                let then = conditional.then.analyze(resolver)?;

                let alternate = conditional
                    .alternate
                    .clone()
                    .map(|alternate| alternate.analyze(resolver))
                    .transpose()?;

                Ok(Analysis::new(Instruction::Conditional(Conditional::new(
                    Box::new(condition),
                    Box::new(then),
                    alternate.map(Box::new),
                ))))
            }
            ElementKind::While(repeat) => {
                let condition = repeat
                    .guard
                    .clone()
                    .map(|guard| guard.analyze(resolver))
                    .transpose()?;
                let body = repeat.body.analyze(resolver)?;
                Ok(Analysis::new(Instruction::While(While::new(
                    condition.map(Box::new),
                    Box::new(body),
                ))))
            }
            ElementKind::Cycle(cycle) => {
                let clause = cycle.guard.analyze(resolver)?;
                let body = cycle.body.analyze(resolver)?;
                Ok(Analysis::new(Instruction::Cycle(Cycle::new(
                    Box::new(clause),
                    Box::new(body),
                ))))
            }
            ElementKind::Symbolize(symbol) => symbol.analyze(resolver),
            ElementKind::Return(output) => {
                let output = output
                    .clone()
                    .map(|output| output.analyze(resolver))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Return(output.map(Box::new))))
            }
            ElementKind::Break(output) => {
                let output = output
                    .clone()
                    .map(|output| output.analyze(resolver))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Break(output.map(Box::new))))
            }
            ElementKind::Continue(output) => {
                let output = output
                    .clone()
                    .map(|output| output.analyze(resolver))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Continue(output.map(Box::new))))
            }
        }
    }
} 