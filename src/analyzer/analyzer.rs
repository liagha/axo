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

    pub fn process(&mut self) {
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
            ElementKind::Unary(_) => { unimplemented!() }
            ElementKind::Binary(binary) => {
                match binary.get_operator().kind {
                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Minus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Subtract(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Star) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Multiply(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Slash) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Divide(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Percent) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    TokenKind::Operator(OperatorKind::Plus) => {
                        let left = self.analyze(*binary.get_left().clone())?;
                        let right = self.analyze(*binary.get_right().clone())?;
                        let operator = binary.get_operator();

                        if left.instruction.is_value() && right.instruction.is_value() {
                            Ok(Analysis::new(Instruction::Add(Box::from(left), Box::from(right))))
                        } else {
                            Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                        }
                    }

                    _ => {
                        let operator = binary.get_operator();

                        Err(AnalyzeError::new(ErrorKind::InvalidOperation(operator.clone()), operator.span))
                    }
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
            ElementKind::Assign(_) => { unimplemented!() }
            ElementKind::Return(_) => { unimplemented!() }
            ElementKind::Break(_) => { unimplemented!() }
            ElementKind::Continue(_) => { unimplemented!() }
        }
    }
}