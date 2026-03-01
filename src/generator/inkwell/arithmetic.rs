use {
    super::Backend,
    inkwell::values::{BasicValueEnum, FunctionValue},
};
use crate::analyzer::Analysis;

impl<'backend> super::Inkwell<'backend> {
    fn coerce_numeric_pair(
        &self,
        left: BasicValueEnum<'backend>,
        right: BasicValueEnum<'backend>,
        name: &str,
    ) -> (BasicValueEnum<'backend>, BasicValueEnum<'backend>, bool) {
        if left.is_int_value() && right.is_int_value() {
            return (left, right, false);
        }

        let float = self.context.f64_type();
        let left = if left.is_float_value() {
            left
        } else if left.is_int_value() {
            self.builder
                .build_signed_int_to_float(
                    left.into_int_value(),
                    float,
                    &format!("{}_lhs_to_float", name),
                )
                .unwrap()
                .into()
        } else {
            float.const_zero().into()
        };

        let right = if right.is_float_value() {
            right
        } else if right.is_int_value() {
            self.builder
                .build_signed_int_to_float(
                    right.into_int_value(),
                    float,
                    &format!("{}_rhs_to_float", name),
                )
                .unwrap()
                .into()
        } else {
            float.const_zero().into()
        };

        (left, right, true)
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);

        let (left, right, floating) = self.coerce_numeric_pair(left, right, "add");

        if !floating {
            BasicValueEnum::from(
                self.builder
                    .build_int_add(left.into_int_value(), right.into_int_value(), "add")
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_add(left.into_float_value(), right.into_float_value(), "add")
                    .unwrap(),
            )
        }
    }

    pub fn subtract(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);

        let (left, right, floating) = self.coerce_numeric_pair(left, right, "subtract");

        if !floating {
            BasicValueEnum::from(
                self.builder
                    .build_int_sub(left.into_int_value(), right.into_int_value(), "subtract")
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_sub(
                        left.into_float_value(),
                        right.into_float_value(),
                        "subtract",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn multiply(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let left = self.instruction(left.instruction, function);
        let right = self.instruction(right.instruction, function);

        let (left, right, floating) = self.coerce_numeric_pair(left, right, "multiply");

        if !floating {
            BasicValueEnum::from(
                self.builder
                    .build_int_mul(left.into_int_value(), right.into_int_value(), "multiply")
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_mul(
                        left.into_float_value(),
                        right.into_float_value(),
                        "multiply",
                    )
                    .unwrap(),
            )
        }
    }

    pub fn divide(
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

        let (left, right, floating) = self.coerce_numeric_pair(left, right, "divide");

        if !floating {
            if signed {
                BasicValueEnum::from(
                    self.builder
                        .build_int_signed_div(
                            left.into_int_value(),
                            right.into_int_value(),
                            "divide",
                        )
                        .unwrap(),
                )
            } else {
                BasicValueEnum::from(
                    self.builder
                        .build_int_unsigned_div(
                            left.into_int_value(),
                            right.into_int_value(),
                            "divide",
                        )
                        .unwrap(),
                )
            }
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_float_div(left.into_float_value(), right.into_float_value(), "divide")
                    .unwrap(),
            )
        }
    }

    pub fn modulus(
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

        let (left, right, floating) = self.coerce_numeric_pair(left, right, "modulus");

        if floating {
            BasicValueEnum::from(
                self.builder
                    .build_float_rem(left.into_float_value(), right.into_float_value(), "modulus")
                    .unwrap(),
            )
        } else if signed {
            BasicValueEnum::from(
                self.builder
                    .build_int_signed_rem(left.into_int_value(), right.into_int_value(), "modulus")
                    .unwrap(),
            )
        } else {
            BasicValueEnum::from(
                self.builder
                    .build_int_unsigned_rem(
                        left.into_int_value(),
                        right.into_int_value(),
                        "modulus",
                    )
                    .unwrap(),
            )
        }
    }
}
