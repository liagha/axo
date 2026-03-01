use {
    super::Backend,
    inkwell::{
        values::{BasicValueEnum, FunctionValue},
        FloatPredicate, IntPredicate,
    },
};
use crate::analyzer::Analysis;

impl<'backend> super::Inkwell<'backend> {
    pub fn equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);
        if left.is_int_value() && right.is_int_value() {
            BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        left.into_int_value(),
                        right.into_int_value(),
                        "equal",
                    )
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OEQ,
                        left.into_float_value(),
                        right.into_float_value(),
                        "equal",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn not_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);
        if left.is_int_value() && right.is_int_value() {
            BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        left.into_int_value(),
                        right.into_int_value(),
                        "not_equal",
                    )
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::ONE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "not_equal",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn less(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);
        if left.is_int_value() && right.is_int_value() {
            BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        if signed {
                            IntPredicate::SLT
                        } else {
                            IntPredicate::ULT
                        },
                        left.into_int_value(),
                        right.into_int_value(),
                        "less",
                    )
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OLT,
                        left.into_float_value(),
                        right.into_float_value(),
                        "less",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn less_or_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);
        if left.is_int_value() && right.is_int_value() {
            BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        if signed {
                            IntPredicate::SLE
                        } else {
                            IntPredicate::ULE
                        },
                        left.into_int_value(),
                        right.into_int_value(),
                        "less_or_equal",
                    )
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OLE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "less_or_equal",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn greater(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);
        if left.is_int_value() && right.is_int_value() {
            BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        if signed {
                            IntPredicate::SGT
                        } else {
                            IntPredicate::UGT
                        },
                        left.into_int_value(),
                        right.into_int_value(),
                        "greater",
                    )
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OGT,
                        left.into_float_value(),
                        right.into_float_value(),
                        "greater",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn greater_or_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);
        if left.is_int_value() && right.is_int_value() {
            BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        if signed {
                            IntPredicate::SGE
                        } else {
                            IntPredicate::UGE
                        },
                        left.into_int_value(),
                        right.into_int_value(),
                        "greater_or_equal",
                    )
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OGE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "greater_or_equal",
                    )
                    .unwrap(),
            )
        }
    }
}
