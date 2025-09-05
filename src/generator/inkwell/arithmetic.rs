use inkwell::values::BasicValueEnum;

use crate::resolver::analyzer::Analysis;

use inkwell::values::FunctionValue;
use crate::generator::Backend;

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_add(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_add(
                left_value.into_int_value(),
                right_value.into_int_value(),
                "add",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_add(
                left_value.into_float_value(),
                right_value.into_float_value(),
                "add",
            ).unwrap())
        }
    }

    pub fn generate_subtract(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_sub(
                left_value.into_int_value(),
                right_value.into_int_value(),
                "subtract",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_sub(
                left_value.into_float_value(),
                right_value.into_float_value(),
                "subtract",
            ).unwrap())
        }
    }

    pub fn generate_multiply(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_mul(
                left_value.into_int_value(),
                right_value.into_int_value(),
                "multiply",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_mul(
                left_value.into_float_value(),
                right_value.into_float_value(),
                "multiply",
            ).unwrap())
        }
    }

    pub fn generate_divide(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        if left_value.is_int_value() && right_value.is_int_value() {
            BasicValueEnum::from(self.builder.build_int_signed_div(
                left_value.into_int_value(),
                right_value.into_int_value(),
                "divide",
            ).unwrap())
        } else {
            BasicValueEnum::from(self.builder.build_float_div(
                left_value.into_float_value(),
                right_value.into_float_value(),
                "divide",
            ).unwrap())
        }
    }

    pub fn generate_modulus(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_int_signed_rem(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "modulus",
        ).unwrap())
    }
}