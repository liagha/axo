use {
    crate::{
        analyzer::{
            Analysis,
        },
        generator::{
            Inkwell,
            Backend,
            ErrorKind,
            GenerateError,
            inkwell::{
                error::{
                    BitwiseError,
                },
            },
        },
        tracker::{
            Span,
        },
    },
    inkwell::{
        values::{
            BasicValueEnum,
        },
        IntPredicate,
    },
};

impl<'backend> Inkwell<'backend> {
    pub fn bitwise_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: "and".to_string() }), span));
        }

        Ok(BasicValueEnum::from(
            self.builder.build_and(primary.into_int_value(), secondary.into_int_value(), "and")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }

    pub fn bitwise_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: "or".to_string() }), span));
        }

        Ok(BasicValueEnum::from(
            self.builder.build_or(primary.into_int_value(), secondary.into_int_value(), "or")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }

    pub fn bitwise_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*operand)?;

        if !alpha.is_int_value() {
            return Err(GenerateError::new(ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: "not".to_string() }), span));
        }

        Ok(BasicValueEnum::from(
            self.builder.build_not(alpha.into_int_value(), "not")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }

    pub fn bitwise_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: "xor".to_string() }), span));
        }

        Ok(BasicValueEnum::from(
            self.builder.build_xor(primary.into_int_value(), secondary.into_int_value(), "xor")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }

    pub fn shift_left(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        if !alpha.is_int_value() || !beta.is_int_value() {
            return Err(GenerateError::new(ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: "shift".to_string() }), span));
        }

        let primary = alpha.into_int_value();
        let secondary = beta.into_int_value();

        let width = primary.get_type().get_bit_width() as u64;
        let limit = secondary.get_type().const_int(width, false);

        let condition = self.builder.build_int_compare(IntPredicate::UGE, secondary, limit, "check")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.trap(Some(condition), span)?;

        Ok(BasicValueEnum::from(
            self.builder.build_left_shift(primary, secondary, "shift")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }

    pub fn shift_right(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);
        let signed = first && second;

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        if !alpha.is_int_value() || !beta.is_int_value() {
            return Err(GenerateError::new(ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: "shift".to_string() }), span));
        }

        let primary = alpha.into_int_value();
        let secondary = beta.into_int_value();

        let width = primary.get_type().get_bit_width() as u64;
        let limit = secondary.get_type().const_int(width, false);

        let condition = self.builder.build_int_compare(IntPredicate::UGE, secondary, limit, "check")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.trap(Some(condition), span)?;

        Ok(BasicValueEnum::from(
            self.builder.build_right_shift(primary, secondary, signed, "shift")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
        ))
    }
}