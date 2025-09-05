use {
    inkwell::{
        values::{
            BasicValueEnum,
            FunctionValue,
        },
    },
    crate::{
        resolver::{
            analyzer::Analysis,
        },
    },
    super::Backend,
};

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_bitwise_and(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_and(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "bitwise_and",
        ).unwrap())
    }

    pub fn generate_bitwise_or(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_or(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "bitwise_or",
        ).unwrap())
    }

    pub fn generate_bitwise_not(&mut self, operand: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let operand_value = self.generate_instruction(operand.instruction, function);
        BasicValueEnum::from(self.builder.build_not(
            operand_value.into_int_value(),
            "bitwise_not",
        ).unwrap())
    }

    pub fn generate_bitwise_xor(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_xor(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "bitwise_xor",
        ).unwrap())
    }

    pub fn generate_shift_left(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_left_shift(
            left_value.into_int_value(),
            right_value.into_int_value(),
            "shift_left",
        ).unwrap())
    }

    pub fn generate_shift_right(&mut self, left: Box<Analysis<'backend>>, right: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let left_value = self.generate_instruction(left.instruction, function);
        let right_value = self.generate_instruction(right.instruction, function);
        BasicValueEnum::from(self.builder.build_right_shift(
            left_value.into_int_value(),
            right_value.into_int_value(),
            true,
            "shift_right",
        ).unwrap())
    }
}