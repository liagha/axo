use {
    crate::{
        analyzer::Analysis,
        generator::{Backend, ErrorKind, GenerateError, Generator},
        tracker::Span,
    },
    inkwell::{
        values::BasicValueEnum,
        IntPredicate,
    },
};

impl<'backend> Generator<'backend> {
    pub fn normalize(
        &self,
        left: BasicValueEnum<'backend>,
        right: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<(BasicValueEnum<'backend>, BasicValueEnum<'backend>, bool), GenerateError<'backend>> {
        if left.get_type() != right.get_type() {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        if left.is_float_value() && right.is_float_value() {
            Ok((left, right, true))
        } else if left.is_int_value() && right.is_int_value() {
            Ok((left, right, false))
        } else {
            Err(GenerateError::new(ErrorKind::Normalize, span))
        }
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, span)?;

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
        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, span)?;

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
        let left_value = self.analysis(*left)?;
        let right_value = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(left_value, right_value, span)?;

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

        let (primary, secondary, floating) = self.normalize(left_value, right_value, span)?;

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

        let (primary, secondary, floating) = self.normalize(left_value, right_value, span)?;

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
