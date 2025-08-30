mod error;

use inkwell::types::AnyType;
use inkwell::values::{AnyValue, AsValueRef, BasicMetadataValueEnum};
use inkwell::{builder::Builder, context::Context, module::Module, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum}, values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue}, AddressSpace, IntPredicate, FloatPredicate};
use std::collections::HashMap;

use crate::{parser::{Element, ElementKind, Symbol}, scanner::{Operator, Token, TokenKind}, schema::{Binding, Method, Structure, Enumeration}};
use crate::analyzer::Analysis;
use crate::analyzer::Instruction;
use crate::data::{Boolean, Integer, Scale};
use crate::data::float::Float;
use crate::data::Str;
use crate::parser::Symbolic;
use crate::scanner::OperatorKind;
use crate::schema::{Binary, Unary};
use error::*;

pub struct Inkwell<'backend> {
    context: &'backend Context,
    builder: Builder<'backend>,
    module: Module<'backend>,
    variables: HashMap<Str<'backend>, PointerValue<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn new(module: Str<'backend>, context: &'backend Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(&module);

        Self {
            context,
            builder,
            module,
            variables: HashMap::new(),
        }
    }

    pub fn instruct(&mut self, analyses: Analysis<'backend>) -> Result<(), Error> {
        Ok(())
    }

    pub fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        println!("Analyses: {:?}", analyses); // Debug print
        let function_type = self.context.i64_type().fn_type(&[], false); // Change to i64 return type
        let function = self.module.add_function("main", function_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let mut last_value = self.context.i64_type().const_zero().into();
        for analysis in analyses {
            last_value = self.generate_instruction(analysis.instruction, function);
        }

        self.builder.build_return(Some(&last_value));
    }

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        match instruction {
            Instruction::Integer(int) => {
                self.context.i64_type().const_int(int.try_into().unwrap(), false).into()
            }
            Instruction::Float(float) => {
                self.context.f64_type().const_float(float.0).into()
            }
            Instruction::Boolean(boolean) => {
                self.context.bool_type().const_int(boolean as u64, false).into()
            }
            Instruction::Add(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_add(
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "add",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_add(
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "add",
                    ).unwrap().into()
                }
            }
            Instruction::Subtract(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_sub(
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "sub",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_sub(
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "sub",
                    ).unwrap().into()
                }
            }
            Instruction::Multiply(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_mul(
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "mul",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_mul(
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "mul",
                    ).unwrap().into()
                }
            }
            Instruction::Divide(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_signed_div(
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "div",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_div(
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "div",
                    ).unwrap().into()
                }
            }
            Instruction::Modulus(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_int_signed_rem(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "mod",
                ).unwrap().into()
            }
            Instruction::LogicalAnd(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_and(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "and",
                ).unwrap().into()
            }
            Instruction::LogicalOr(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_or(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "or",
                ).unwrap().into()
            }
            Instruction::LogicalNot(operand) => {
                let operand_val = self.generate_instruction(operand.instruction, function);

                self.builder.build_not(
                    operand_val.into_int_value(),
                    "not",
                ).unwrap().into()
            }
            Instruction::BitwiseAnd(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_and(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "bitand",
                ).unwrap().into()
            }
            Instruction::BitwiseOr(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_or(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "bitor",
                ).unwrap().into()
            }
            Instruction::BitwiseNot(operand) => {
                let operand_val = self.generate_instruction(operand.instruction, function);

                self.builder.build_not(
                    operand_val.into_int_value(),
                    "bitnot",
                ).unwrap().into()
            }
            Instruction::BitwiseXOr(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_xor(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "bitxor",
                ).unwrap().into()
            }
            Instruction::ShiftLeft(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_left_shift(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    "shl",
                ).unwrap().into()
            }
            Instruction::ShiftRight(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                self.builder.build_right_shift(
                    left_val.into_int_value(),
                    right_val.into_int_value(),
                    true,
                    "shr",
                ).unwrap().into()
            }
            Instruction::Equal(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_compare(
                        IntPredicate::EQ,
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "eq",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_compare(
                        FloatPredicate::OEQ,
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "eq",
                    ).unwrap().into()
                }
            }
            Instruction::NotEqual(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_compare(
                        IntPredicate::NE,
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "ne",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_compare(
                        FloatPredicate::ONE,
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "ne",
                    ).unwrap().into()
                }
            }
            Instruction::Less(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_compare(
                        IntPredicate::SLT,
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "lt",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_compare(
                        FloatPredicate::OLT,
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "lt",
                    ).unwrap().into()
                }
            }
            Instruction::LessOrEqual(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_compare(
                        IntPredicate::SLE,
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "le",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_compare(
                        FloatPredicate::OLE,
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "le",
                    ).unwrap().into()
                }
            }
            Instruction::Greater(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_compare(
                        IntPredicate::SGT,
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "gt",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_compare(
                        FloatPredicate::OGT,
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "gt",
                    ).unwrap().into()
                }
            }
            Instruction::GreaterOrEqual(left, right) => {
                let left_val = self.generate_instruction(left.instruction, function);
                let right_val = self.generate_instruction(right.instruction, function);

                if left_val.is_int_value() && right_val.is_int_value() {
                    self.builder.build_int_compare(
                        IntPredicate::SGE,
                        left_val.into_int_value(),
                        right_val.into_int_value(),
                        "ge",
                    ).unwrap().into()
                } else {
                    self.builder.build_float_compare(
                        FloatPredicate::OGE,
                        left_val.into_float_value(),
                        right_val.into_float_value(),
                        "ge",
                    ).unwrap().into()
                }
            }
            Instruction::Usage(identifier) => {
                let ptr = self.variables.get(&identifier).unwrap();
                self.builder.build_load(ptr.get_type(), *ptr, &identifier).unwrap().into()
            }
            Instruction::Binding(binding) => {
                let value = self.generate_instruction(binding.get_value().unwrap().instruction.clone(), function);

                let ptr = if value.is_int_value() {
                    self.builder.build_alloca(self.context.i64_type(), &binding.get_target())
                } else if value.is_float_value() {
                    self.builder.build_alloca(self.context.f64_type(), &binding.get_target())
                } else {
                    self.builder.build_alloca(self.context.bool_type(), &binding.get_target())
                }.unwrap();

                self.builder.build_store(ptr, value);
                self.variables.insert(*binding.get_target(), ptr);
                value
            }
            Instruction::Module(name, analyses) => {
                let function_type = self.context.void_type().fn_type(&[], false);
                let function = self.module.add_function(&name, function_type, None);
                let basic_block = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(basic_block);

                for analysis in analyses {
                    self.generate_instruction(analysis.instruction, function);
                }

                self.builder.build_return(None);
                self.context.i64_type().const_zero().into()
            }
            _ => self.context.i64_type().const_zero().into()
        }
    }

    pub fn print_ir(&self) {
        let ir = self.module.print_to_string();
        println!("{}", ir.to_string());
    }
}