use {
    crate::{
        data::Str,
        analyzer::{
            Analysis, Instruction,
            AnalyzeError,
        },
        parser::{Element, ElementKind, Symbolic},
        scanner::{OperatorKind, Token, TokenKind},
        schema::Binding,
    },
};
use crate::analyzer::ErrorKind;

pub struct Analyzer<'analyzer> {
    pub input: Vec<Element<'analyzer>>,
    pub output: Vec<Analysis<'analyzer>>,
    pub errors: Vec<AnalyzeError<'analyzer>>,
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn new() -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn with_input(&mut self, input: Vec<Element<'analyzer>>) {
        self.input = input;
    }

    pub fn process(&mut self) -> Vec<Analysis<'analyzer>> {
        for element in self.input.clone() {
            let analysis = self.analyze(element.clone());

            match analysis {
                Ok(analysis) => {
                    self.output.push(analysis);
                }
                Err(error) => {
                    self.errors.push(error);
                }
            }
        }
        
        self.output.clone()
    }

    pub fn analyze(&mut self, element: Element<'analyzer>) -> Result<Analysis<'analyzer>, AnalyzeError<'analyzer>> {
        match &element.kind {
            ElementKind::Literal(literal) => {
                match literal.kind {
                    TokenKind::Float(float) => {
                        Ok(Analysis::new(Instruction::Float(float.clone())))
                    }
                    TokenKind::Integer(integer) => {
                        Ok(Analysis::new(Instruction::Integer(integer.clone())))
                    }
                    TokenKind::Boolean(boolean) => {
                        Ok(Analysis::new(Instruction::Boolean(boolean.clone())))
                    }
                    TokenKind::Identifier(identifier) => {
                        Ok(Analysis::new(Instruction::Usage(identifier.clone())))
                    }
                    TokenKind::String(_) => { unimplemented!() }
                    TokenKind::Character(_) => { unimplemented!() }
                    TokenKind::Operator(_) => { unimplemented!() }
                    TokenKind::Punctuation(_) => { unimplemented!() }
                    TokenKind::Comment(_) => { unimplemented!() }
                }
            }
            ElementKind::Procedural(_) => { unimplemented!() }
            ElementKind::Group(_) => { unimplemented!() }
            ElementKind::Sequence(_) => { unimplemented!() }
            ElementKind::Collection(_) => { unimplemented!() }
            ElementKind::Series(_) => { unimplemented!() }
            ElementKind::Bundle(_) => { unimplemented!() }
            ElementKind::Block(_) => { unimplemented!() }
            ElementKind::Unary(unary) => {
                if let TokenKind::Operator(operator) = &unary.get_operator().kind {
                    match operator.as_slice() {
                        [OperatorKind::Exclamation] => {
                            let operand = self.analyze(*unary.get_operand().clone())?;
                            let operator = unary.get_operator();

                            if operand.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalNot(Box::from(operand))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }
                        _ => {
                            let operator = unary.get_operator();

                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }
                } else {
                    let operator = unary.get_operator();

                    Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                }
            }
            ElementKind::Binary(binary) => {
                if let TokenKind::Operator(operator) = &binary.get_operator().kind {
                    match operator.as_slice() {
                        [OperatorKind::Plus] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Minus] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Subtract(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Star] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Multiply(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Slash] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Divide(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Percent] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Modulus(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Ampersand, OperatorKind::Ampersand] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalAnd(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Pipe, OperatorKind::Pipe] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LogicalOr(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Ampersand] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseAnd(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Pipe] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseOr(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Caret] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::BitwiseXOr(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::ShiftLeft(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::ShiftRight(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Equal, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Equal(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::Exclamation, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::NotEqual(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::LeftAngle] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Less(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::LeftAngle, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::LessOrEqual(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::RightAngle] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::Greater(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        [OperatorKind::RightAngle, OperatorKind::Equal] => {
                            let left = self.analyze(*binary.get_left().clone())?;
                            let right = self.analyze(*binary.get_right().clone())?;
                            let operator = binary.get_operator();

                            if left.instruction.is_value() || right.instruction.is_value() {
                                Ok(Analysis::new(Instruction::GreaterOrEqual(Box::from(left), Box::from(right))))
                            } else {
                                Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                            }
                        }

                        _ => {
                            let operator = binary.get_operator();

                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                } else {
                    let operator = binary.get_operator();

                    Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                }
            }
            ElementKind::Label(_) => { unimplemented!() }
            ElementKind::Access(_) => { unimplemented!() }
            ElementKind::Index(_) => { unimplemented!() }
            ElementKind::Invoke(_) => { unimplemented!() }
            ElementKind::Construct(_) => { unimplemented!() }
            ElementKind::Conditional(_) => { unimplemented!() }
            ElementKind::While(_) => { unimplemented!() }
            ElementKind::Cycle(_) => { unimplemented!() }
            ElementKind::Symbolize(symbol) => {
                match &symbol.value {
                    Symbolic::Inclusion(_) => { unimplemented!() }
                    Symbolic::Extension(_) => { unimplemented!() }
                    Symbolic::Binding(binding) => {
                        let value = self.analyze(*(binding.get_value().unwrap().clone()))?;
                        let transformed = Binding::new(
                            Str::from(binding.get_target().brand().unwrap().to_string()),
                            Some(Box::new(value)),
                            None,
                            binding.is_constant()
                        );

                        Ok(Analysis::new(Instruction::Binding(transformed)))
                    }
                    Symbolic::Structure(_) => { unimplemented!() }
                    Symbolic::Enumeration(_) => { unimplemented!() }
                    Symbolic::Method(_) => { unimplemented!() }
                    Symbolic::Module(_) => { unimplemented!() }
                    Symbolic::Preference(_) => { unimplemented!() }
                }
            }
            ElementKind::Assign(assign) => {
                let target = assign.get_target().brand().unwrap().to_string();
                let value = self.analyze(*assign.get_value().clone())?;

                Ok(Analysis::new(Instruction::Assign(Str::from(target), Box::from(value))))
            }
            ElementKind::Return(_) => { unimplemented!() }
            ElementKind::Break(_) => { unimplemented!() }
            ElementKind::Continue(_) => { unimplemented!() }
        }
    }
}