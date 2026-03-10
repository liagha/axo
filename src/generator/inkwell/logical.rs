use {
    super::Backend,
    crate::analyzer::Analysis,
    inkwell::values::{BasicValueEnum, IntValue},
};

impl<'backend> super::Inkwell<'backend> {
    fn check_is_1bit_int(&self, value: BasicValueEnum<'backend>, operation: &str) -> IntValue<'backend> {
        if !value.is_int_value() || value.into_int_value().get_type().get_bit_width() != 1 {
            panic!("Logical {} requires 1-bit integer (boolean) operands.", operation);
        }
        value.into_int_value()
    }

    pub fn logical_and(
        &mut self,
        left_expr: Box<Analysis<'backend>>,
        right_expr: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left_analyzed = self.analysis(*left_expr);
        let right_analyzed = self.analysis(*right_expr);

        let left_value = self.check_is_1bit_int(left_analyzed, "AND");
        let right_value = self.check_is_1bit_int(right_analyzed, "AND");
        BasicValueEnum::from(
            self.builder
                .build_and(left_value, right_value, "and")
                .unwrap(),
        )
    }

    pub fn logical_or(
        &mut self,
        left_expr: Box<Analysis<'backend>>, 
        right_expr: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left_analyzed = self.analysis(*left_expr);
        let right_analyzed = self.analysis(*right_expr);

        let left_value = self.check_is_1bit_int(left_analyzed, "OR");
        let right_value = self.check_is_1bit_int(right_analyzed, "OR");
        BasicValueEnum::from(
            self.builder
                .build_or(left_value, right_value, "or")
                .unwrap(),
        )
    }

    pub fn logical_not(
        &mut self,
        operand_expr: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let operand_analyzed = self.analysis(*operand_expr);
        let operand_value = self.check_is_1bit_int(operand_analyzed, "NOT");
        BasicValueEnum::from(
            self.builder
                .build_not(operand_value, "not")
                .unwrap(),
        )
    }

    pub fn logical_xor(
        &mut self,
        left_expr: Box<Analysis<'backend>>,
        right_expr: Box<Analysis<'backend>>,
    ) -> BasicValueEnum<'backend> {
        let left_analyzed = self.analysis(*left_expr);
        let right_analyzed = self.analysis(*right_expr);

        let left_value = self.check_is_1bit_int(left_analyzed, "XOR");
        let right_value = self.check_is_1bit_int(right_analyzed, "XOR");
        BasicValueEnum::from(
            self.builder
                .build_xor(left_value, right_value, "xor")
                .unwrap(),
        )
    }
}
