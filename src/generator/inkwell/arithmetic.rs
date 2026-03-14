use {
    crate::{
        analyzer::Analysis,
        generator::{Backend, ErrorKind, GenerateError, Inkwell},
        tracker::Span,
    },
    inkwell::{
        values::BasicValueEnum,
        IntPredicate,
    },
};

impl<'backend> Inkwell<'backend> {
    pub fn normalize(
        &self,
        mut left: BasicValueEnum<'backend>,
        mut right: BasicValueEnum<'backend>,
        mut signs: [bool; 2],
        span: Span<'backend>,
    ) -> Result<(BasicValueEnum<'backend>, BasicValueEnum<'backend>, bool), GenerateError<'backend>> {
        let pointer = self.context.i64_type();

        if left.is_pointer_value() {
            signs[0] = false;
            left = self.builder
                .build_ptr_to_int(left.into_pointer_value(), pointer, "cast")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                .into();
        }

        if right.is_pointer_value() {
            signs[1] = false;
            right = self.builder
                .build_ptr_to_int(right.into_pointer_value(), pointer, "cast")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                .into();
        }

        if left.is_int_value() && right.is_int_value() {
            let first = left.into_int_value();
            let second = right.into_int_value();

            let first_size = first.get_type().get_bit_width();
            let second_size = second.get_type().get_bit_width();

            if first_size < second_size {
                let target = second.get_type();
                let extended = if signs[0] {
                    self.builder.build_int_s_extend(first, target, "extend")
                } else {
                    self.builder.build_int_z_extend(first, target, "extend")
                }
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                return Ok((extended.into(), right, false));
            }

            if second_size < first_size {
                let target = first.get_type();
                let extended = if signs[1] {
                    self.builder.build_int_s_extend(second, target, "extend")
                } else {
                    self.builder.build_int_z_extend(second, target, "extend")
                }
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                return Ok((left, extended.into(), false));
            }

            return Ok((left, right, false));
        }

        let mut target = self.context.f64_type();

        if left.is_float_value() && right.is_float_value() {
            let first_type = left.into_float_value().get_type();
            let second_type = right.into_float_value().get_type();

            if first_type == second_type {
                target = first_type;
            }
        } else if left.is_float_value() {
            target = left.into_float_value().get_type();
        } else if right.is_float_value() {
            target = right.into_float_value().get_type();
        }

        let primary = if left.is_float_value() {
            let value = left.into_float_value();

            if value.get_type() != target {
                self.builder
                    .build_float_ext(value, target, "extend")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into()
            } else {
                left
            }
        } else if left.is_int_value() {
            let value = left.into_int_value();

            if signs[0] {
                self.builder
                    .build_signed_int_to_float(value, target, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into()
            } else {
                self.builder
                    .build_unsigned_int_to_float(value, target, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into()
            }
        } else {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        };

        let secondary = if right.is_float_value() {
            let value = right.into_float_value();

            if value.get_type() != target {
                self.builder
                    .build_float_ext(value, target, "extend")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into()
            } else {
                right
            }
        } else if right.is_int_value() {
            let value = right.into_int_value();

            if signs[1] {
                self.builder
                    .build_signed_int_to_float(value, target, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into()
            } else {
                self.builder
                    .build_unsigned_int_to_float(value, target, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                    .into()
            }
        } else {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        };

        Ok((primary, secondary, true))
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_sign = self.infer_signedness(&left).unwrap_or(true);
        let right_sign = self.infer_signedness(&right).unwrap_or(true);

        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, [left_sign, right_sign], span)?;

        if floating {
            let result = self.builder
                .build_float_add(primary.into_float_value(), secondary.into_float_value(), "add")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return Ok(result.into());
        }

        let result = self.builder
            .build_int_add(primary.into_int_value(), secondary.into_int_value(), "add")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        Ok(result.into())
    }

    pub fn subtract(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_sign = self.infer_signedness(&left).unwrap_or(true);
        let right_sign = self.infer_signedness(&right).unwrap_or(true);

        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, [left_sign, right_sign], span)?;

        if floating {
            let result = self.builder
                .build_float_sub(primary.into_float_value(), secondary.into_float_value(), "subtract")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return Ok(result.into());
        }

        let result = self.builder
            .build_int_sub(primary.into_int_value(), secondary.into_int_value(), "subtract")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        Ok(result.into())
    }

    pub fn multiply(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_sign = self.infer_signedness(&left).unwrap_or(true);
        let right_sign = self.infer_signedness(&right).unwrap_or(true);

        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, [left_sign, right_sign], span)?;

        if floating {
            let result = self.builder
                .build_float_mul(primary.into_float_value(), secondary.into_float_value(), "multiply")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return Ok(result.into());
        }

        let result = self.builder
            .build_int_mul(primary.into_int_value(), secondary.into_int_value(), "multiply")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        Ok(result.into())
    }

    pub fn divide(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_sign = self.infer_signedness(&left).unwrap_or(true);
        let right_sign = self.infer_signedness(&right).unwrap_or(true);

        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, [left_sign, right_sign], span)?;

        if floating {
            let result = self.builder
                .build_float_div(primary.into_float_value(), secondary.into_float_value(), "divide")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return Ok(result.into());
        }

        let dividend = primary.into_int_value();
        let divisor = secondary.into_int_value();
        let zero = divisor.get_type().const_zero();

        let is_zero = self.builder
            .build_int_compare(IntPredicate::EQ, divisor, zero, "check")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        if left_sign && right_sign {
            let negative_one = divisor.get_type().const_all_ones();
            let is_negative = self.builder
                .build_int_compare(IntPredicate::EQ, divisor, negative_one, "check")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let one = divisor.get_type().const_int(1, false);
            let shift = divisor.get_type().const_int((divisor.get_type().get_bit_width() - 1) as u64, false);
            let minimum = self.builder
                .build_left_shift(one, shift, "minimum")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let is_minimum = self.builder
                .build_int_compare(IntPredicate::EQ, dividend, minimum, "check")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let is_overflow = self.builder
                .build_and(is_negative, is_minimum, "overflow")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let condition = self.builder
                .build_or(is_zero, is_overflow, "condition")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            self.trap(Some(condition), span)?;

            let result = self.builder
                .build_int_signed_div(dividend, divisor, "divide")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            Ok(result.into())
        } else {
            self.trap(Some(is_zero), span)?;

            let result = self.builder
                .build_int_unsigned_div(dividend, divisor, "divide")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            Ok(result.into())
        }
    }

    pub fn modulus(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_sign = self.infer_signedness(&left).unwrap_or(true);
        let right_sign = self.infer_signedness(&right).unwrap_or(true);

        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, [left_sign, right_sign], span)?;

        if floating {
            let result = self.builder
                .build_float_rem(primary.into_float_value(), secondary.into_float_value(), "modulus")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            return Ok(result.into());
        }

        let dividend = primary.into_int_value();
        let divisor = secondary.into_int_value();
        let zero = divisor.get_type().const_zero();

        let is_zero = self.builder
            .build_int_compare(IntPredicate::EQ, divisor, zero, "check")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        if left_sign && right_sign {
            let negative_one = divisor.get_type().const_all_ones();
            let is_negative = self.builder
                .build_int_compare(IntPredicate::EQ, divisor, negative_one, "check")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let one = divisor.get_type().const_int(1, false);
            let shift = divisor.get_type().const_int((divisor.get_type().get_bit_width() - 1) as u64, false);
            let minimum = self.builder
                .build_left_shift(one, shift, "minimum")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let is_minimum = self.builder
                .build_int_compare(IntPredicate::EQ, dividend, minimum, "check")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let is_overflow = self.builder
                .build_and(is_negative, is_minimum, "overflow")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let condition = self.builder
                .build_or(is_zero, is_overflow, "condition")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            self.trap(Some(condition), span)?;

            let result = self.builder
                .build_int_signed_rem(dividend, divisor, "modulus")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            Ok(result.into())
        } else {
            self.trap(Some(is_zero), span)?;

            let result = self.builder
                .build_int_unsigned_rem(dividend, divisor, "modulus")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            Ok(result.into())
        }
    }
}