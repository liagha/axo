use {
    super::Backend,
    inkwell::values::{BasicValueEnum},
};
use crate::analyzer::Analysis;

impl<'backend> super::Inkwell<'backend> {
    pub fn bitwise_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);

        BasicValueEnum::from(
            self.builder
                .build_and(left.into_int_value(), right.into_int_value(), "bitwise_and")
                .unwrap(),
        )
    }

    pub fn bitwise_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);
        BasicValueEnum::from(
            self.builder
                .build_or(left.into_int_value(), right.into_int_value(), "bitwise_or")
                .unwrap(),
        )
    }

    pub fn bitwise_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let operand_value = self.analysis(*operand);
        BasicValueEnum::from(
            self.builder
                .build_not(operand_value.into_int_value(), "bitwise_not")
                .unwrap(),
        )
    }

    pub fn bitwise_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);
        BasicValueEnum::from(
            self.builder
                .build_xor(left.into_int_value(), right.into_int_value(), "bitwise_xor")
                .unwrap(),
        )
    }

    pub fn shift_left(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);
        BasicValueEnum::from(
            self.builder
                .build_left_shift(left.into_int_value(), right.into_int_value(), "shift_left")
                .unwrap(),
        )
    }

    pub fn shift_right(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);
        let left = self.analysis(*left);
        let right = self.analysis(*right);
        BasicValueEnum::from(
            self.builder
                .build_right_shift(
                    left.into_int_value(),
                    right.into_int_value(),
                    signed,
                    "shift_right",
                )
                .unwrap(),
        )
    }
}
