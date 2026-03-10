use {
    super::{Backend, Inkwell},
    crate::{
        data::{Str, Boolean},
        analyzer::Analysis,
        generator::{ErrorKind, ArithmeticError, GenerateError},
    },
    inkwell::{
        values::{
            IntValue,
            BasicValueEnum,
        },
        IntPredicate
    },
};
use crate::tracker::Span;

impl<'backend> Inkwell<'backend> {
    fn zero_trap(
        &self,
        divisor: IntValue<'backend>,
        name: Str<'backend>, 
        span: Span<'backend>
    ) -> Result<(), GenerateError<'backend>> {
        let zero_value = divisor.get_type().const_zero();
        let is_zero = self.builder.build_int_compare(
            IntPredicate::EQ,
            divisor,
            zero_value,
            &format!("is_{}_zero", name),
        ).map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        let current_block = self.builder.get_insert_block()
            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError { reason: "No current basic block".to_string() }, span))?;

        let function = current_block.get_parent()
            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError { reason: "No parent function for block".to_string()}, span))?;

        let trap_block = self.context.append_basic_block(function, &format!("trap_{}_zero", name));

        let continue_block = self.context.append_basic_block(function, &format!("continue_{}", name));

        self.builder.build_conditional_branch(is_zero, trap_block, continue_block)
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        self.builder.position_at_end(trap_block);

        let trap_function = self.current_module().get_function("llvm.trap")
            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError { reason: "llvm.trap intrinsic not found".to_string() }, span))?;

        self.builder.build_call(trap_function, &[], "trap_call")
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        self.builder.build_unreachable()
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        self.builder.position_at_end(continue_block);

        Ok(())
    }

    pub fn normalize_pair(
        &self,
        left: BasicValueEnum<'backend>,
        right: BasicValueEnum<'backend>,
        left_signed: bool,
        right_signed: bool,
        name: &str, 
        span: Span<'backend>
    ) -> Result<(BasicValueEnum<'backend>, BasicValueEnum<'backend>, Boolean), GenerateError<'backend>> {
        if left.is_int_value() && right.is_int_value() {
            let left_int = left.into_int_value();
            let right_int = right.into_int_value();

            let left_width = left_int.get_type().get_bit_width();
            let right_width = right_int.get_type().get_bit_width();

            if left_width < right_width {
                let new_left = if left_signed {
                    self.builder.build_int_s_extend(left_int, right_int.get_type(), &format!("{}_ext_lhs", name))
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                } else {
                    self.builder.build_int_z_extend(left_int, right_int.get_type(), &format!("{}_ext_lhs", name))
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                };
                return Ok((new_left.into(), right, false));
            } else if right_width < left_width {
                let new_right = if right_signed {
                    self.builder.build_int_s_extend(right_int, left_int.get_type(), &format!("{}_ext_rhs", name))
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                } else {
                    self.builder.build_int_z_extend(right_int, left_int.get_type(), &format!("{}_ext_rhs", name))
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                };
                return Ok((left, new_right.into(), false));
            }

            return Ok((left, right, false));
        }

        let mut float_type = self.context.f64_type();

        if left.is_float_value() {
            float_type = left.into_float_value().get_type();
        }

        if right.is_float_value() {
            let right_float_type = right.into_float_value().get_type();
            if right_float_type != float_type && right_float_type == self.context.f64_type() {
                float_type = self.context.f64_type();
            }
        }

        let left_normalized = if left.is_float_value() {
            let float_val = left.into_float_value();
            if float_val.get_type() != float_type {
                self.builder.build_float_ext(float_val, float_type, &format!("{}_ext_lhs", name))
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?.into()
            } else {
                left
            }
        } else if left.is_int_value() {
            if left_signed {
                self.builder.build_signed_int_to_float(left.into_int_value(), float_type, &format!("{}_lhs_to_float", name))
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?.into()
            } else {
                self.builder.build_unsigned_int_to_float(left.into_int_value(), float_type, &format!("{}_lhs_to_float", name))
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?.into()
            }
        } else {
            return Err(GenerateError::new(ErrorKind::Arithmetic(ArithmeticError::InvalidOperandType { side: "LHS".to_string(), instruction: name.to_string() }), span));
        };

        let right_normalized = if right.is_float_value() {
            let float_val = right.into_float_value();
            if float_val.get_type() != float_type {
                self.builder.build_float_ext(float_val, float_type, &format!("{}_ext_rhs", name))
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?.into()
            } else {
                right
            }
        } else if right.is_int_value() {
            if right_signed {
                self.builder.build_signed_int_to_float(right.into_int_value(), float_type, &format!("{}_rhs_to_float", name))
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?.into()
            } else {
                self.builder.build_unsigned_int_to_float(right.into_int_value(), float_type, &format!("{}_rhs_to_float", name))
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?.into()
            }
        } else {
            return Err(GenerateError::new(ErrorKind::Arithmetic(ArithmeticError::InvalidOperandType { side: "RHS".to_string(), instruction: name.to_string() }), span));
        };

        Ok((left_normalized, right_normalized, true))
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "add", span)?;

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_add(left.into_int_value(), right.into_int_value(), "add")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_add(left.into_float_value(), right.into_float_value(), "add")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        }
    }

    pub fn subtract(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "subtract", span)?;

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_sub(left.into_int_value(), right.into_int_value(), "subtract")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_sub(left.into_float_value(), right.into_float_value(), "subtract")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        }
    }

    pub fn multiply(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "multiply", span)?;

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_mul(left.into_int_value(), right.into_int_value(), "multiply")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_mul(left.into_float_value(), right.into_float_value(), "multiply")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        }
    }

    pub fn divide(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);
        let combined_signed = left_signed && right_signed;

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "divide", span)?;

        if !floating {
            let divisor = right.into_int_value();
            self.zero_trap(divisor, Str::from("div"), span)?;

            if combined_signed {
                Ok(BasicValueEnum::from(
                    self.builder.build_int_signed_div(left.into_int_value(), divisor, "divide")
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                ))
            } else {
                Ok(BasicValueEnum::from(
                    self.builder.build_int_unsigned_div(left.into_int_value(), divisor, "divide")
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                ))
            }
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_div(left.into_float_value(), right.into_float_value(), "divide")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        }
    }

    pub fn modulus(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);
        let combined_signed = left_signed && right_signed;

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "modulus", span)?;

        if floating {
            Ok(BasicValueEnum::from(
                self.builder.build_float_rem(left.into_float_value(), right.into_float_value(), "modulus")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            ))
        } else {
            let divisor = right.into_int_value();
            self.zero_trap(divisor, Str::from("mod"), span)?;

            if combined_signed {
                Ok(BasicValueEnum::from(
                    self.builder.build_int_signed_rem(left.into_int_value(), divisor, "modulus")
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                ))
            } else {
                Ok(BasicValueEnum::from(
                    self.builder.build_int_unsigned_rem(left.into_int_value(), divisor, "modulus")
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                ))
            }
        }
    }
}