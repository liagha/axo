use {
    super::Backend,
    inkwell::values::{BasicValueEnum, FunctionValue},
};
use crate::analyzer::Analysis;

impl<'backend> super::Inkwell<'backend> {
    pub fn logical_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left, function);
        let right = self.analysis(*right, function);
        BasicValueEnum::from(
            self.builder
                .build_and(left.into_int_value(), right.into_int_value(), "and")
                .unwrap(),
        )
    }

    pub fn logical_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left, function);
        let right = self.analysis(*right, function);
        BasicValueEnum::from(
            self.builder
                .build_or(left.into_int_value(), right.into_int_value(), "or")
                .unwrap(),
        )
    }

    pub fn logical_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let operand_value = self.analysis(*operand, function);
        BasicValueEnum::from(
            self.builder
                .build_not(operand_value.into_int_value(), "not")
                .unwrap(),
        )
    }

    pub fn logical_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left, function);
        let right = self.analysis(*right, function);
        BasicValueEnum::from(
            self.builder
                .build_xor(left.into_int_value(), right.into_int_value(), "xor")
                .unwrap(),
        )
    }
}
