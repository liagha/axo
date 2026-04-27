use {
    crate::{
        analyzer::Analysis,
        generator::{BitwiseError, ErrorKind, GenerateError, Generator},
        tracker::Span,
    },
    inkwell::{values::BasicValueEnum, IntPredicate},
};

impl<'backend> Generator<'backend> {
    pub fn bitwise_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(
                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                    instruction: String::from("and"),
                }),
                span,
            ));
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_and(primary.into_int_value(), secondary.into_int_value(), "and")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?,
        ))
    }

    pub fn bitwise_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(
                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                    instruction: String::from("or"),
                }),
                span,
            ));
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_or(primary.into_int_value(), secondary.into_int_value(), "or")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?,
        ))
    }

    pub fn bitwise_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*operand)?;

        if !alpha.is_int_value() {
            return Err(GenerateError::new(
                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                    instruction: String::from("not"),
                }),
                span,
            ));
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_not(alpha.into_int_value(), "not")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?,
        ))
    }

    pub fn bitwise_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(
                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                    instruction: String::from("xor"),
                }),
                span,
            ));
        }

        Ok(BasicValueEnum::from(
            self.builder
                .build_xor(primary.into_int_value(), secondary.into_int_value(), "xor")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?,
        ))
    }

    pub fn shift_left(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(
                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                    instruction: String::from("shift"),
                }),
                span,
            ));
        }

        let base = primary.into_int_value();
        let amount = secondary.into_int_value();

        let width = base.get_type().get_bit_width() as u64;
        let limit = amount.get_type().const_int(width, false);

        let condition = self
            .builder
            .build_int_compare(IntPredicate::UGE, amount, limit, "check")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.trap(Some(condition), span)?;

        Ok(BasicValueEnum::from(
            self.builder
                .build_left_shift(base, amount, "shift")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?,
        ))
    }

    pub fn shift_right(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let first = self.infer_signedness(&left).unwrap_or(true);
        let second = self.infer_signedness(&right).unwrap_or(true);
        let signed = first && second;

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(
                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                    instruction: String::from("shift"),
                }),
                span,
            ));
        }

        let base = primary.into_int_value();
        let amount = secondary.into_int_value();

        let width = base.get_type().get_bit_width() as u64;
        let limit = amount.get_type().const_int(width, false);

        let condition = self
            .builder
            .build_int_compare(IntPredicate::UGE, amount, limit, "check")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

        self.trap(Some(condition), span)?;

        Ok(BasicValueEnum::from(
            self.builder
                .build_right_shift(base, amount, signed, "shift")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?,
        ))
    }
}
