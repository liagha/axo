use crate::resolver::analyzer::{Analysis, AnalyzeError, ErrorKind, Instruction};
use crate::resolver::Resolver;
use crate::scanner::Token;
use crate::{data, data::Str, parser::{Element, ElementKind, Symbol, SymbolKind}, scanner::{OperatorKind, TokenKind}, schema::{Assign, Binding, Enumeration, Index, Invoke, Method, Structure}};
use crate::data::Scale;
use crate::schema::{Block, Conditional, Cycle, While};

impl<'analyzer> Resolver<'analyzer> {
    pub fn analyze(
        &mut self,
        element: Element<'analyzer>,
    ) -> Result<Analysis<'analyzer>, AnalyzeError<'analyzer>> {
        match &element.kind {
            ElementKind::Literal(literal) => self.analyze_literal(literal),
            ElementKind::Procedural(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Group(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Sequence(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Collection(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Series(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Bundle(bundle) => {
                let items: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> = bundle
                    .items
                    .iter()
                    .map(|item| self.analyze(item.clone()).map(Box::new))
                    .collect();
                Ok(Analysis::new(Instruction::Block(Block::new(items?))))
            }
            ElementKind::Block(block) => {
                let items: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> = block
                    .items
                    .iter()
                    .map(|item| self.analyze(item.clone()).map(Box::new))
                    .collect();
                Ok(Analysis::new(Instruction::Block(Block::new(items?))))
            }
            ElementKind::Unary(unary) => {
                if let TokenKind::Operator(operator) = &unary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Exclamation] => {
                            let operand = self.analyze(*unary.operand.clone())?;
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
                            let operand = self.analyze(*unary.operand.clone())?;
                            if operand.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseNot(Box::new(operand))))
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
                        [OperatorKind::Plus] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
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
            ElementKind::Label(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span)),
            ElementKind::Access(access) => {
                let target = self.analyze(*access.target.clone())?;
                let member = self.analyze(*access.member.clone())?;

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

                // Fallback to regular Access if not a primitive operation
                Ok(Analysis::new(Instruction::Access(
                    Box::new(target),
                    Box::new(member),
                )))
            }
            ElementKind::Index(index) => {
                let target = self.analyze(*index.target.clone())?;
                let indexes: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> = index
                    .members
                    .iter()
                    .map(|idx| self.analyze(idx.clone()).map(Box::new))
                    .collect();
                Ok(Analysis::new(Instruction::Index(Index::new(
                    Box::new(target),
                    indexes?,
                ))))
            }
            ElementKind::Invoke(invoke) => {
                let target = self.analyze(*invoke.target.clone())?;
                let arguments: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    invoke
                        .members
                        .iter()
                        .map(|arg| self.analyze(arg.clone()).map(Box::new))
                        .collect();
                Ok(Analysis::new(Instruction::Invoke(Invoke::new(
                    Box::new(target),
                    arguments?,
                ))))
            }
            ElementKind::Construct(constructor) => {
                let target_name = constructor
                    .target
                    .brand()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let fields: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    constructor
                        .members
                        .iter()
                        .map(|field| self.analyze(field.clone()).map(Box::new))
                        .collect();
                let fields = fields?;

                match target_name.as_str() {
                    "Integer" => {
                        if fields.len() == 3 {
                            if let (
                                Instruction::Integer { value, .. },
                                Instruction::Integer { size, .. },
                                Instruction::Boolean { value: signed }
                            ) = (&fields[0].instruction, &fields[1].instruction, &fields[2].instruction) {
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
                        if fields.len() == 2 {
                            if let (Instruction::Float { value, .. }, Instruction::Integer { size, .. }) = (&fields[0].instruction, &fields[1].instruction) {
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
                        if fields.len() == 1 {
                            if let Instruction::Boolean { value } = &fields[0].instruction {
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
                        let analyzed = Structure::new(Str::from(target_name), fields);
                        Ok(Analysis::new(Instruction::Constructor(analyzed)))
                    }
                }
            }
            ElementKind::Conditional(conditional) => {
                let condition = self.analyze(*conditional.condition.clone())?;
                let then = self.analyze(*conditional.then.clone())?;
                let alternate = conditional
                    .alternate
                    .clone()
                    .map(|alt| self.analyze(*alt))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Conditional(Conditional::new(
                    Box::new(condition),
                    Box::new(then),
                    alternate.map(Box::new),
                ))))
            }
            ElementKind::While(repeat) => {
                let condition = repeat
                    .condition
                    .clone()
                    .map(|c| self.analyze(*c))
                    .transpose()?;
                let body = self.analyze(*repeat.body.clone())?;
                Ok(Analysis::new(Instruction::While(While::new(
                    condition.map(Box::new),
                    Box::new(body),
                ))))
            }
            ElementKind::Cycle(cycle) => {
                let clause = self.analyze(*cycle.clause.clone())?;
                let body = self.analyze(*cycle.body.clone())?;
                Ok(Analysis::new(Instruction::Cycle(Cycle::new(
                    Box::new(clause),
                    Box::new(body),
                ))))
            }
            ElementKind::Symbolize(symbol) => self.analyze_symbol(symbol.clone()),
            ElementKind::Assign(assign) => {
                let target = assign.target.brand().unwrap().to_string();
                let value = self.analyze(*assign.value.clone())?;
                Ok(Analysis::new(Instruction::Assign(Assign::new(
                    Str::from(target),
                    Box::new(value),
                ))))
            }
            ElementKind::Return(output) => {
                let output = output
                    .clone()
                    .map(|out| self.analyze(*out))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Return(output.map(Box::new))))
            }
            ElementKind::Break(output) => {
                let output = output
                    .clone()
                    .map(|out| self.analyze(*out))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Break(output.map(Box::new))))
            }
            ElementKind::Continue(output) => {
                let output = output
                    .clone()
                    .map(|out| self.analyze(*out))
                    .transpose()?;
                Ok(Analysis::new(Instruction::Continue(output.map(Box::new))))
            }
        }
    }

    pub fn analyze_literal(
        &mut self,
        literal: &Token<'analyzer>,
    ) -> Result<Analysis<'analyzer>, AnalyzeError<'analyzer>> {
        match &literal.kind {
            TokenKind::Float(float) => Ok(Analysis::new(Instruction::Float { value: float.clone(), size: 64 })),
            TokenKind::Integer(integer) => Ok(Analysis::new(Instruction::Integer { value: integer.clone(), size: 64, signed: true })),
            TokenKind::Boolean(boolean) => Ok(Analysis::new(Instruction::Boolean { value: boolean.clone() })),
            TokenKind::Identifier(identifier) => {
                Ok(Analysis::new(Instruction::Usage(identifier.clone())))
            }
            TokenKind::String(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, literal.span)),
            TokenKind::Character(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, literal.span))
            }
            TokenKind::Operator(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, literal.span))
            }
            TokenKind::Punctuation(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, literal.span))
            }
            TokenKind::Comment(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, literal.span)),
        }
    }

    pub fn analyze_symbol(
        &mut self,
        symbol: Symbol<'analyzer>,
    ) -> Result<Analysis<'analyzer>, AnalyzeError<'analyzer>> {
        match &symbol.kind {
            SymbolKind::Inclusion(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span)),
            SymbolKind::Extension(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span)),
            SymbolKind::Binding(binding) => {
                let value = binding
                    .value
                    .clone()
                    .map(|v| self.analyze(*v))
                    .transpose()?;

                let annotation = binding
                    .annotation
                    .clone()
                    .map(|v| self.analyze(*v))
                    .transpose()?;

                let analyzed = Binding::new(
                    Str::from(binding.target.brand().unwrap().to_string()),
                    value.map(Box::new),
                    annotation.map(Box::new),
                    binding.constant,
                );
                Ok(Analysis::new(Instruction::Binding(analyzed)))
            }
            SymbolKind::Structure(structure) => {
                let fields: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    structure
                        .members
                        .iter()
                        .map(|field| self.analyze_symbol(field.clone()).map(Box::new))
                        .collect();
                let analyzed = Structure::new(
                    Str::from(structure.target.brand().unwrap().to_string()),
                    fields?,
                );
                Ok(Analysis::new(Instruction::Structure(analyzed)))
            }
            SymbolKind::Enumeration(enumeration) => {
                let variants: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    enumeration
                        .members
                        .iter()
                        .map(|field| self.analyze_symbol(field.clone()).map(Box::new))
                        .collect();
                let analyzed = Enumeration::new(
                    Str::from(enumeration.target.brand().unwrap().to_string()),
                    variants?,
                );
                Ok(Analysis::new(Instruction::Enumeration(analyzed)))
            }
            SymbolKind::Method(method) => {
                let parameters: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    method
                        .members
                        .iter()
                        .map(|field| self.analyze_symbol(field.clone()).map(Box::new))
                        .collect();
                let body = self.analyze(*method.body.clone())?;
                let output = method
                    .output
                    .clone()
                    .map(|out| self.analyze(*out).map(Box::new))
                    .transpose()?;
                let analyzed = Method::new(
                    Str::from(method.target.brand().unwrap().to_string()),
                    parameters?,
                    Box::new(body),
                    output,
                    method.variadic,
                );
                Ok(Analysis::new(Instruction::Method(analyzed)))
            }
            SymbolKind::Module(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span)),
            SymbolKind::Preference(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span))
            }
        }
    }
}