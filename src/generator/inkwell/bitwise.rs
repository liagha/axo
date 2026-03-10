use inkwell::IntPredicate;
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
use crate::generator::GenerateError;

impl<'backend> Inkwell<'backend> {
    pub fn bitwise_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        if !left.is_int_value() || !right.is_int_value() {
            panic!("Bitwise AND requires integer operands.");
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_and(left.into_int_value(), right.into_int_value(), "bitwise_and")
                .unwrap(),
        ))
    }

    pub fn bitwise_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        if !left.is_int_value() || !right.is_int_value() {
            panic!("Bitwise OR requires integer operands.");
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_or(left.into_int_value(), right.into_int_value(), "bitwise_or")
                .unwrap(),
        ))
    }

    pub fn bitwise_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let operand_value = self.analysis(*operand)?;

        if !operand_value.is_int_value() {
            panic!("Bitwise NOT requires an integer operand.");
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_not(operand_value.into_int_value(), "bitwise_not")
                .unwrap(),
        ))
    }

    pub fn bitwise_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        if !left.is_int_value() || !right.is_int_value() {
            panic!("Bitwise XOR requires integer operands.");
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_xor(left.into_int_value(), right.into_int_value(), "bitwise_xor")
                .unwrap(),
        ))
    }

    pub fn shift_left(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        if !left.is_int_value() || !right.is_int_value() {
            panic!("Left shift requires integer operands.");
        }

        let shift_amt = right.into_int_value();
        let operand_bit_width = left.into_int_value().get_type().get_bit_width() as u64;
        let max_shift_amt = self.context.i32_type().const_int(operand_bit_width, false);

        let is_shift_invalid = self.builder.build_int_compare(
            IntPredicate::UGE,
            shift_amt,
            max_shift_amt,
            "shift_left_bound_check"
        ).unwrap();

        let current_block = self.builder.get_insert_block().unwrap();
        let function = current_block.get_parent().unwrap();

        let trap_block = self.context.append_basic_block(function, "trap_shift_invalid");
        let continue_block = self.context.append_basic_block(function, "continue_shift_left");

        self.builder.build_conditional_branch(is_shift_invalid, trap_block, continue_block);

        self.builder.position_at_end(trap_block);
        self.builder.build_call(self.current_module().get_function("llvm.trap").unwrap(), &[], "trap_call");
        self.builder.build_unreachable();

        self.builder.position_at_end(continue_block);

        Ok(BasicValueEnum::from(
            self.builder
                .build_left_shift(left.into_int_value(), shift_amt, "shift_left")
                .unwrap(),
        ))
    }

    pub fn shift_right(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let signed = self
            .infer_signedness(&left)
            .zip(self.infer_signedness(&right))
            .map(|(lhs, rhs)| lhs && rhs)
            .unwrap_or(true);

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        if !left.is_int_value() || !right.is_int_value() {
            panic!("Right shift requires integer operands.");
        }

        let shift_amt = right.into_int_value();
        let operand_bit_width = left.into_int_value().get_type().get_bit_width() as u64;
        let max_shift_amt = self.context.i32_type().const_int(operand_bit_width, false);

        let is_shift_invalid = self.builder.build_int_compare(
            IntPredicate::UGE,
            shift_amt,
            max_shift_amt,
            "shift_right_bound_check"
        ).unwrap();

        let current_block = self.builder.get_insert_block().unwrap();
        let function = current_block.get_parent().unwrap();

        let trap_block = self.context.append_basic_block(function, "trap_shift_invalid");
        let continue_block = self.context.append_basic_block(function, "continue_shift_right");

        self.builder.build_conditional_branch(is_shift_invalid, trap_block, continue_block);

        self.builder.position_at_end(trap_block);
        self.builder.build_call(self.current_module().get_function("llvm.trap").unwrap(), &[], "trap_call");
        self.builder.build_unreachable();

        self.builder.position_at_end(continue_block);

        Ok(BasicValueEnum::from(
            self.builder
                .build_right_shift(
                    left.into_int_value(),
                    shift_amt,
                    signed,
                    "shift_right",
                )
                .unwrap(),
        ))
    }
}
