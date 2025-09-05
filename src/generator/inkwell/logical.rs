use inkwell::values::BasicValueEnum;

use crate::resolver::analyzer::Analysis;

use inkwell::values::FunctionValue;
use crate::generator::Backend;

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_logical_and(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_and(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "and",
        ).unwrap())
    }

    pub fn generate_logical_or(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_or(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "or",
        ).unwrap())
    }

    pub fn generate_logical_not(&mut self, operand: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let operand_value = self.generate_instruction(operand.instruction, function);
        BasicValueEnum::from(self.builder.build_not(
            operand_value.into_int_value(),
            "not",
        ).unwrap())
    }
}