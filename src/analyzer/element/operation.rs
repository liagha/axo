use crate::{
    parser::Element,
    resolver::Resolver,
    scanner::{OperatorKind, Token, TokenKind},
};
use crate::analyzer::{Analysis, AnalyzeError, ErrorKind, Instruction};
use crate::data::schema::{Binary, Unary};
use super::{analyze, Analyzer};

pub(crate) fn binary<'binary>(
    node: &Binary<Box<Element<'binary>>, Token<'binary>, Box<Element<'binary>>>,
    resolver: &Resolver<'binary>,
    context: Analyzer,
) -> Result<Analysis<'binary>, AnalyzeError<'binary>> {
    if let TokenKind::Operator(operator) = &node.operator.kind {
        match operator.as_slice() {
            [OperatorKind::Dot] => {
                let target = analyze(&node.left, resolver, context)?;
                let member = analyze(&node.right, resolver, context)?;

                Ok(Analysis::new(Instruction::Access(
                    Box::new(target),
                    Box::new(member),
                )))
            }

            [OperatorKind::Equal] => {
                let target = analyze(&node.left, resolver, context)?;
                let value = analyze(&node.right, resolver, context)?;

                match &target.instruction {
                    Instruction::Usage(target_name) => Ok(Analysis::new(Instruction::Assign(
                        target_name.clone(),
                        Box::new(value),
                    ))),
                    Instruction::Dereference(_) => Ok(Analysis::new(Instruction::Store(
                        Box::new(target),
                        Box::new(value),
                    ))),
                    _ => Err(AnalyzeError::new(
                        ErrorKind::InvalidOperation(node.operator.clone()),
                        node.operator.span,
                    )),
                }
            }

            [OperatorKind::Plus] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Add(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Minus] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Subtract(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Star] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Multiply(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Slash] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Divide(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Percent] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Modulus(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Ampersand, OperatorKind::Ampersand] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::LogicalAnd(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Pipe, OperatorKind::Pipe] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::LogicalOr(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Caret] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::LogicalXOr(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Ampersand] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::BitwiseAnd(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Pipe] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::BitwiseOr(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::ShiftLeft(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::ShiftRight(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Equal, OperatorKind::Equal] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Equal(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::Exclamation, OperatorKind::Equal] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::NotEqual(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::LeftAngle] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Less(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::LeftAngle, OperatorKind::Equal] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::LessOrEqual(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::RightAngle] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::Greater(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            [OperatorKind::RightAngle, OperatorKind::Equal] => {
                let left = analyze(&node.left, resolver, context)?;
                let right = analyze(&node.right, resolver, context)?;
                Ok(Analysis::new(Instruction::GreaterOrEqual(
                    Box::new(left),
                    Box::new(right),
                )))
            }

            _ => Err(AnalyzeError::new(
                ErrorKind::InvalidOperation(node.operator.clone()),
                node.operator.span,
            )),
        }
    } else {
        Err(AnalyzeError::new(
            ErrorKind::InvalidOperation(node.operator.clone()),
            node.operator.span,
        ))
    }
}

pub(crate) fn analyze_unary<'unary>(
    node: &Unary<Token<'unary>, Box<Element<'unary>>>,
    resolver: &Resolver<'unary>,
    context: Analyzer,
) -> Result<Analysis<'unary>, AnalyzeError<'unary>> {
    if let TokenKind::Operator(operator) = &node.operator.kind {
        let operand = analyze(&node.operand, resolver, context)?;

        return match operator.as_slice() {
            [OperatorKind::Exclamation] => {
                Ok(Analysis::new(Instruction::LogicalNot(Box::new(operand))))
            }
            [OperatorKind::Tilde] => Ok(Analysis::new(Instruction::BitwiseNot(Box::new(operand)))),
            [OperatorKind::Plus] => Ok(operand),
            [OperatorKind::Minus] => Ok(Analysis::new(Instruction::Subtract(
                Box::new(Analysis::new(Instruction::Integer {
                    value: 0,
                    size: 64,
                    signed: true,
                })),
                Box::new(operand),
            ))),
            [OperatorKind::Ampersand] => {
                Ok(Analysis::new(Instruction::AddressOf(Box::new(operand))))
            }
            [OperatorKind::Star] => Ok(Analysis::new(Instruction::Dereference(Box::new(operand)))),
            _ => Err(AnalyzeError::new(
                ErrorKind::InvalidOperation(node.operator.clone()),
                node.operator.span,
            )),
        };
    }

    Err(AnalyzeError::new(
        ErrorKind::InvalidOperation(node.operator.clone()),
        node.operator.span,
    ))
}
