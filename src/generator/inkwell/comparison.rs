use inkwell::{FloatPredicate, IntPredicate, values::BasicValueEnum};

use crate::resolver::analyzer::Analysis;

use inkwell::values::FunctionValue;
use crate::generator::Backend;

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_equal(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_compare(
                IntPredicate::EQ,
                left_value.into_int_value(),
                right_value.into_int_value(),
                "equal",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_compare(
                FloatPredicate::OEQ,
                left_value.into_float_value(),
                right_value.into_float_value(),
                "equal",
            ).unwrap())
        }
    }

    pub fn generate_not_equal(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_compare(
                IntPredicate::NE,
                left_value.into_int_value(),
                right_value.into_int_value(),
                "not_equal",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_compare(
                FloatPredicate::ONE,
                left_value.into_float_value(),
                right_value.into_float_value(),
                "not_equal",
            ).unwrap())
        }
    }

    pub fn generate_less(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_compare(
                IntPredicate::SLT,
                left_value.into_int_value(),
                right_value.into_int_value(),
                "less",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_compare(
                FloatPredicate::OLT,
                left_value.into_float_value(),
                right_value.into_float_value(),
                "less",
            ).unwrap())
        }
    }

    pub fn generate_less_or_equal(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_compare(
                IntPredicate::SLE,
                left_value.into_int_value(),
                right_value.into_int_value(),
                "less_or_equal",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_compare(
                FloatPredicate::OLE,
                left_value.into_float_value(),
                right_value.into_float_value(),
                "less_or_equal",
            ).unwrap())
        }
    }

    pub fn generate_greater(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_compare(
                IntPredicate::SGT,
                left_value.into_int_value(),
                right_value.into_int_value(),
                "greater",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_compare(
                FloatPredicate::OGT,
                left_value.into_float_value(),
                right_value.into_float_value(),
                "greater",
            ).unwrap())
        }
    }

    pub fn generate_greater_or_equal(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_compare(
                IntPredicate::SGE,
                left_value.into_int_value(),
                right_value.into_int_value(),
                "greater_or_equal",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_compare(
                FloatPredicate::OGE,
                left_value.into_float_value(),
                right_value.into_float_value(),
                "greater_or_equal",
            ).unwrap())
        }
    }
}