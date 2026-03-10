use {
    super::Backend,
    crate::analyzer::Analysis,
    inkwell::values::{BasicValueEnum, IntValue},
};
use crate::generator::GenerateError;

impl<'backend> super::Inkwell<'backend> {
    fn check_is_1bit_int(&self, value: BasicValueEnum<'backend>, operation: &str) -> IntValue<'backend> {
        if !value.is_int_value() || value.into_int_value().get_type().get_bit_width() != 1 {
            panic!("Logical {} requires 1-bit integer (boolean) operands.", operation);
        }
        value.into_int_value()
    }

    pub fn logical_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_analyzed = self.analysis(*left)?;
        let right_analyzed = self.analysis(*right)?;

        let left_value = self.check_is_1bit_int(left_analyzed, "AND");
        let right_value = self.check_is_1bit_int(right_analyzed, "AND");
        Ok(BasicValueEnum::from(
            self.builder
                .build_and(left_value, right_value, "and")
                .unwrap(),
        ))
    }

    pub fn logical_or(
        &mut self,
        left: Box<Analysis<'backend>>, 
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_analyzed = self.analysis(*left)?;
        let right_analyzed = self.analysis(*right)?;

        let left_value = self.check_is_1bit_int(left_analyzed, "OR");
        let right_value = self.check_is_1bit_int(right_analyzed, "OR");
        Ok(BasicValueEnum::from(
            self.builder
                .build_or(left_value, right_value, "or")
                .unwrap(),
        ))
    }

    pub fn logical_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let operand_analyzed = self.analysis(*operand)?;
        
        let operand_value = self.check_is_1bit_int(operand_analyzed, "NOT");
        
        Ok(BasicValueEnum::from(
            self.builder
                .build_not(operand_value, "not")
                .unwrap(),
        ))
    }

    pub fn logical_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_analyzed = self.analysis(*left)?;
        let right_analyzed = self.analysis(*right)?;

        let left_value = self.check_is_1bit_int(left_analyzed, "XOR");
        let right_value = self.check_is_1bit_int(right_analyzed, "XOR");
        Ok(BasicValueEnum::from(
            self.builder
                .build_xor(left_value, right_value, "xor")
                .unwrap(),
        ))
    }
}
