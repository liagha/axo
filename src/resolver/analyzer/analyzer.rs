use crate::resolver::analyzer::{Analysis, AnalyzeError, ErrorKind, Instruction};
use crate::resolver::Resolver;
use crate::scanner::Token;
use crate::{
    data::Str,
    parser::{Element, ElementKind, Symbol, Symbolic},
    scanner::{OperatorKind, TokenKind},
    schema::{Assign, Binding, Enumeration, Index, Invoke, Method, Structure},
};

impl<'analyzer> Resolver<'analyzer> {
    pub fn analyze(
        &mut self,
        element: Element<'analyzer>,
    ) -> Result<Analysis<'analyzer>, AnalyzeError<'analyzer>> {
        match &element.kind {
            ElementKind::Literal(literal) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Procedural(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Group(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            },
            ElementKind::Sequence(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Collection(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Series(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Bundle(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::Block(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span)),
            ElementKind::Unary(unary) => {
                if let TokenKind::Operator(operator) = &unary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Exclamation] => {
                            let operand = self.analyze(*unary.operand.clone())?;
                            let operator = &unary.operator;

                            if operand.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalNot(Box::new(operand))))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }
                        _ => {
                            let operator = &unary.operator;

                            Err(AnalyzeError::new(
                                ErrorKind::InvalidOperation(operator.clone()),
                                operator.span,
                            ))
                        }
                    }
                } else {
                    let operator = &unary.operator;

                    Err(AnalyzeError::new(
                        ErrorKind::InvalidOperation(operator.clone()),
                        operator.span,
                    ))
                }
            }
            ElementKind::Binary(binary) => {
                if let TokenKind::Operator(operator) = &binary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Plus] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Add(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Minus] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Subtract(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Star] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Multiply(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Slash] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Divide(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Percent] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Modulus(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Ampersand, OperatorKind::Ampersand] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalAnd(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Pipe, OperatorKind::Pipe] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalOr(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Ampersand] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseAnd(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Pipe] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseOr(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Caret] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseXOr(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::ShiftLeft(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::ShiftRight(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Equal, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Equal(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::Exclamation, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::NotEqual(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::LeftAngle] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Less(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::LeftAngle, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LessOrEqual(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::RightAngle] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Greater(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        [OperatorKind::RightAngle, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.left.clone())?;
                            let right = self.analyze(*binary.right.clone())?;
                            let operator = &binary.operator;

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::GreaterOrEqual(
                                    Box::new(left),
                                    Box::new(right),
                                )))
                            } else {
                                Err(AnalyzeError::new(
                                    ErrorKind::InvalidOperation(operator.clone()),
                                    operator.span,
                                ))
                            }
                        }

                        _ => {
                            let operator = &binary.operator;

                            Err(AnalyzeError::new(
                                ErrorKind::InvalidOperation(operator.clone()),
                                operator.span,
                            ))
                        }
                    }
                } else {
                    let operator = &binary.operator;

                    Err(AnalyzeError::new(
                        ErrorKind::InvalidOperation(operator.clone()),
                        operator.span,
                    ))
                }
            }
            ElementKind::Label(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span)),
            ElementKind::Access(access) => {
                let left = self.analyze(*access.target.clone())?;
                let right = self.analyze(*access.member.clone())?;

                Ok(Analysis::new(Instruction::Access(
                    Box::new(left),
                    Box::new(right),
                )))
            }
            ElementKind::Index(index) => {
                let target = self.analyze(*index.target.clone())?;
                let index = self.analyze(index.indexes[0].clone())?;

                let analyzed = Index::new(Box::new(target), vec![Box::new(index)]);

                Ok(Analysis::new(Instruction::Index(analyzed)))
            }
            ElementKind::Invoke(invoke) => {
                let target = self.analyze(*invoke.target.clone())?;
                let arguments: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    invoke
                        .arguments
                        .iter()
                        .map(|argument| {
                            let analysis = self.analyze(argument.clone())?;
                            Ok(Box::new(analysis))
                        })
                        .collect();

                let analyzed = Invoke::new(Box::new(target), arguments?);

                Ok(Analysis::new(Instruction::Invoke(analyzed)))
            }
            ElementKind::Construct(constructor) => {
                let fields: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    constructor
                        .members
                        .iter()
                        .map(|field| {
                            let analysis = self.analyze(field.clone())?;
                            Ok(Box::new(analysis))
                        })
                        .collect();

                let analyzed = Structure::new(
                    Str::from(constructor.target.brand().unwrap().to_string()),
                    fields?,
                );

                Ok(Analysis::new(Instruction::Constructor(analyzed)))
            }
            ElementKind::Conditional(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span))
            }
            ElementKind::While(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span)),
            ElementKind::Cycle(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, element.span)),
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
                    .map(|output| self.analyze(*output.clone()))
                    .transpose()?;

                Ok(Analysis::new(Instruction::Return(output.map(Box::new))))
            }
            ElementKind::Break(output) => {
                let output = output
                    .clone()
                    .map(|output| self.analyze(*output.clone()))
                    .transpose()?;

                Ok(Analysis::new(Instruction::Break(output.map(Box::new))))
            }
            ElementKind::Continue(output) => {
                let output = output
                    .clone()
                    .map(|output| self.analyze(*output.clone()))
                    .transpose()?;

                Ok(Analysis::new(Instruction::Continue(output.map(Box::new))))
            }
        }
    }

    pub fn analyze_literal(
        &mut self,
        literal: &Token<'analyzer>,
    ) -> Result<Analysis<'analyzer>, AnalyzeError<'analyzer>> {
        match literal.kind {
            TokenKind::Float(float) => Ok(Analysis::new(Instruction::Float(float.clone()))),
            TokenKind::Integer(integer) => Ok(Analysis::new(Instruction::Integer(integer.clone()))),
            TokenKind::Boolean(boolean) => Ok(Analysis::new(Instruction::Boolean(boolean.clone()))),
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
            Symbolic::Inclusion(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span)),
            Symbolic::Extension(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span)),
            Symbolic::Binding(binding) => {
                let value = self.analyze(*binding.clone().value.unwrap())?;
                let analyzed = Binding::new(
                    Str::from(binding.target.brand().unwrap().to_string()),
                    Some(Box::new(value)),
                    None,
                    binding.constant,
                );
                Ok(Analysis::new(Instruction::Binding(analyzed)))
            }
            Symbolic::Structure(structure) => {
                let fields: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    structure
                        .members
                        .iter()
                        .map(|field| {
                            let analysis = self.analyze_symbol(field.clone())?;
                            Ok(Box::new(analysis))
                        })
                        .collect();

                let analyzed = Structure::new(
                    Str::from(structure.target.brand().unwrap().to_string()),
                    fields?,
                );

                Ok(Analysis::new(Instruction::Structure(analyzed)))
            }
            Symbolic::Enumeration(enumeration) => {
                let variants: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    enumeration
                        .members
                        .iter()
                        .map(|field| {
                            let analysis = self.analyze_symbol(field.clone())?;
                            Ok(Box::new(analysis))
                        })
                        .collect();

                let analyzed = Enumeration::new(
                    Str::from(enumeration.target.brand().unwrap().to_string()),
                    variants?,
                );

                Ok(Analysis::new(Instruction::Enumeration(analyzed)))
            }
            Symbolic::Method(method) => {
                let parameters: Result<Vec<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    method
                        .members
                        .iter()
                        .map(|field| {
                            let analysis = self.analyze_symbol(field.clone())?;
                            Ok(Box::new(analysis))
                        })
                        .collect();

                let body = self.analyze(*method.body.clone())?;

                let output: Result<Option<Box<Analysis<'analyzer>>>, AnalyzeError<'analyzer>> =
                    method
                        .output
                        .clone()
                        .map(|output| self.analyze(*output).map(Box::new))
                        .transpose();

                let analyzed = Method::new(
                    Str::from(method.target.brand().unwrap().to_string()),
                    parameters?,
                    Box::new(body),
                    output?,
                );

                Ok(Analysis::new(Instruction::Method(analyzed)))
            }
            Symbolic::Module(_) => Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span)),
            Symbolic::Preference(_) => {
                Err(AnalyzeError::new(ErrorKind::UnImplemented, symbol.span))
            }
        }
    }
}
