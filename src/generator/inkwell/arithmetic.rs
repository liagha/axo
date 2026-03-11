use {
    super::{
        Backend,
        Inkwell,
    },
    crate::{
        analyzer::{
            Analysis,
        },
        generator::{
            ErrorKind,
            GenerateError,
        },
        tracker::{
            Span,
        },
    },
    inkwell::{
        values::{
            BasicValueEnum as Value,
            IntValue as Integer,
        },
        IntPredicate as Decision,
    },
};
use crate::generator::BuilderError;

impl<'backend> Inkwell<'backend> {
    pub fn trap(
        &self,
        condition: Integer<'backend>,
        span: Span<'backend>,
    ) -> Result<(), GenerateError<'backend>> {
        let block = self.builder.get_insert_block().ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

        let parent = block.get_parent().ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

        let failure = self.context.append_basic_block(parent, "failure");
        let success = self.context.append_basic_block(parent, "success");

        self.builder.build_conditional_branch(condition, failure, success)
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(failure);

        let intrinsic = self.current_module().get_function("llvm.trap")
            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Function), span))?;

        self.builder.build_call(intrinsic, &[], "call")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.build_unreachable()
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.builder.position_at_end(success);

        Ok(())
    }

    pub fn normalize(
        &self,
        mut primary: Value<'backend>,
        mut secondary: Value<'backend>,
        signs: [bool; 2],
        span: Span<'backend>,
    ) -> Result<(Value<'backend>, Value<'backend>, bool), GenerateError<'backend>> {
        let pointer = self.context.i64_type();

        if primary.is_pointer_value() {
            primary = self.builder.build_ptr_to_int(primary.into_pointer_value(), pointer, "cast")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into();
        }

        if secondary.is_pointer_value() {
            secondary = self.builder.build_ptr_to_int(secondary.into_pointer_value(), pointer, "cast")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into();
        }

        if primary.is_int_value() && secondary.is_int_value() {
            let alpha = primary.into_int_value();
            let beta = secondary.into_int_value();

            let sizes = [alpha.get_type().get_bit_width(), beta.get_type().get_bit_width()];

            if sizes[0] < sizes[1] {
                let extended = if signs[0] {
                    self.builder.build_int_s_extend(alpha, beta.get_type(), "extend")
                } else {
                    self.builder.build_int_z_extend(alpha, beta.get_type(), "extend")
                }.map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                return Ok((extended.into(), secondary, false));
            } else if sizes[1] < sizes[0] {
                let extended = if signs[1] {
                    self.builder.build_int_s_extend(beta, alpha.get_type(), "extend")
                } else {
                    self.builder.build_int_z_extend(beta, alpha.get_type(), "extend")
                }.map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                return Ok((primary, extended.into(), false));
            }

            return Ok((primary, secondary, false));
        }

        let mut kind = self.context.f64_type();

        if primary.is_float_value() && secondary.is_float_value() {
            let types = [primary.into_float_value().get_type(), secondary.into_float_value().get_type()];
            if types[0] != types[1] {
                kind = self.context.f64_type();
            } else {
                kind = types[0];
            }
        } else if primary.is_float_value() {
            kind = primary.into_float_value().get_type();
        } else if secondary.is_float_value() {
            kind = secondary.into_float_value().get_type();
        }

        let first = if primary.is_float_value() {
            let float = primary.into_float_value();
            if float.get_type() != kind {
                self.builder.build_float_ext(float, kind, "extend")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into()
            } else {
                primary
            }
        } else if primary.is_int_value() {
            if signs[0] {
                self.builder.build_signed_int_to_float(primary.into_int_value(), kind, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into()
            } else {
                self.builder.build_unsigned_int_to_float(primary.into_int_value(), kind, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into()
            }
        } else {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        };

        let second = if secondary.is_float_value() {
            let float = secondary.into_float_value();
            if float.get_type() != kind {
                self.builder.build_float_ext(float, kind, "extend")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into()
            } else {
                secondary
            }
        } else if secondary.is_int_value() {
            if signs[1] {
                self.builder.build_signed_int_to_float(secondary.into_int_value(), kind, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into()
            } else {
                self.builder.build_unsigned_int_to_float(secondary.into_int_value(), kind, "convert")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?.into()
            }
        } else {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        };

        Ok((first, second, true))
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<Value<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            Ok(Value::from(
                self.builder.build_int_add(primary.into_int_value(), secondary.into_int_value(), "add")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(Value::from(
                self.builder.build_float_add(primary.into_float_value(), secondary.into_float_value(), "add")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn subtract(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<Value<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            Ok(Value::from(
                self.builder.build_int_sub(primary.into_int_value(), secondary.into_int_value(), "subtract")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(Value::from(
                self.builder.build_float_sub(primary.into_float_value(), secondary.into_float_value(), "subtract")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn multiply(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<Value<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            Ok(Value::from(
                self.builder.build_int_mul(primary.into_int_value(), secondary.into_int_value(), "multiply")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(Value::from(
                self.builder.build_float_mul(primary.into_float_value(), secondary.into_float_value(), "multiply")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn divide(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<Value<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            let divisor = secondary.into_int_value();
            let zero = divisor.get_type().const_zero();
            let condition = self.builder.build_int_compare(Decision::EQ, divisor, zero, "check")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            self.trap(condition, span)?;

            if first && second {
                Ok(Value::from(
                    self.builder.build_int_signed_div(primary.into_int_value(), divisor, "divide")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                ))
            } else {
                Ok(Value::from(
                    self.builder.build_int_unsigned_div(primary.into_int_value(), divisor, "divide")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                ))
            }
        } else {
            Ok(Value::from(
                self.builder.build_float_div(primary.into_float_value(), secondary.into_float_value(), "divide")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn modulus(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<Value<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if floating {
            Ok(Value::from(
                self.builder.build_float_rem(primary.into_float_value(), secondary.into_float_value(), "modulus")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            let divisor = secondary.into_int_value();
            let zero = divisor.get_type().const_zero();
            let condition = self.builder.build_int_compare(Decision::EQ, divisor, zero, "check")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            self.trap(condition, span)?;

            if first && second {
                Ok(Value::from(
                    self.builder.build_int_signed_rem(primary.into_int_value(), divisor, "modulus")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                ))
            } else {
                Ok(Value::from(
                    self.builder.build_int_unsigned_rem(primary.into_int_value(), divisor, "modulus")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                ))
            }
        }
    }
}