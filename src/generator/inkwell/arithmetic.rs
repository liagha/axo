use {
    super::{
        Backend,
        Inkwell,
    },
    crate::{
        analyzer::Analysis,
    },
    inkwell::values::{BasicValueEnum},
};

impl<'backend> Inkwell<'backend> {
    pub(super) fn coerce_numeric_pair(
        &self,
        left: BasicValueEnum<'backend>,
        right: BasicValueEnum<'backend>,
        name: &str,
    ) -> (BasicValueEnum<'backend>, BasicValueEnum<'backend>, bool) {
        if left.is_int_value() && right.is_int_value() {
            return (left, right, false);
        }

        let float_type = self.context.f64_type();

        let left = if left.is_float_value() {
            left
        } else if left.is_int_value() {
            self.builder
                .build_signed_int_to_float(
                    left.into_int_value(),
                    float_type,
                    &format!("{}_lhs_to_float", name),
                )
                .unwrap()
                .into()
        } else {
            panic!(
                "Invalid left-hand side type for arithmetic operation '{}': {:?}",
                name,
                left.get_type()
            );
        };

        let right = if right.is_float_value() {
            right
        } else if right.is_int_value() {
            self.builder
                .build_signed_int_to_float(
                    right.into_int_value(),
                    float_type,
                    &format!("{}_rhs_to_float", name),
                )
                .unwrap()
                .into()
        } else {
            panic!(
                "Invalid right-hand side type for arithmetic operation '{}': {:?}",
                name,
                right.get_type()
            );
        };

        (left, right, true)
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);

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
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);

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
    ) -> BasicValueEnum<'backend> {
        let left = self.analysis(*left);
        let right = self.analysis(*right);

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
        left_expr: Box<Analysis<'backend>>,
        right_expr: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left_expr)
            .zip(self.infer_signedness(&right_expr))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);

        let left = self.analysis(*left_expr);
        let right = self.analysis(*right_expr);

        let (left, right, floating) = self.coerce_numeric_pair(left, right, "divide");

        if !floating {
            let divisor = right.into_int_value();
            let zero_val = divisor.get_type().const_zero();
            let is_zero = self.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                divisor,
                zero_val,
                "is_div_zero",
            ).unwrap();

            let current_block = self.builder.get_insert_block().unwrap();
            let function = current_block.get_parent().unwrap();

            let trap_block = self.context.append_basic_block(function, "trap_div_zero");
            let continue_block = self.context.append_basic_block(function, "continue_div");

            self.builder.build_conditional_branch(is_zero, trap_block, continue_block);

            self.builder.position_at_end(trap_block);
            self.builder.build_call(self.current_module().get_function("llvm.trap").unwrap(), &[], "trap_call");
            self.builder.build_unreachable();

            self.builder.position_at_end(continue_block);

            if signed {
                BasicValueEnum::from(
                    self.builder
                        .build_int_signed_div(
                            left.into_int_value(),
                            divisor, // Use divisor for the actual division
                            "divide",
                        )
                        .unwrap(),
                )
            } else {
                BasicValueEnum::from(
                    self.builder
                        .build_int_unsigned_div(
                            left.into_int_value(),
                            divisor, // Use divisor for the actual division
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
    ) -> BasicValueEnum<'backend> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);

        let left = self.analysis(*left);
        let right = self.analysis(*right);

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
