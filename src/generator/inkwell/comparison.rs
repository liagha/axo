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
            BasicValueEnum,
        },
        FloatPredicate,
        IntPredicate,
    },
};

impl<'backend> Inkwell<'backend> {
    pub fn equal(
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

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(IntPredicate::EQ, primary.into_int_value(), secondary.into_int_value(), "equal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(FloatPredicate::OEQ, primary.into_float_value(), secondary.into_float_value(), "equal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn not_equal(
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

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(IntPredicate::NE, primary.into_int_value(), secondary.into_int_value(), "unequal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(FloatPredicate::ONE, primary.into_float_value(), secondary.into_float_value(), "unequal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn less(
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            let limit = if signed { IntPredicate::SLT } else { IntPredicate::ULT };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(limit, primary.into_int_value(), secondary.into_int_value(), "less")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(FloatPredicate::OLT, primary.into_float_value(), secondary.into_float_value(), "less")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn less_or_equal(
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            let limit = if signed { IntPredicate::SLE } else { IntPredicate::ULE };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(limit, primary.into_int_value(), secondary.into_int_value(), "less_equal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(FloatPredicate::OLE, primary.into_float_value(), secondary.into_float_value(), "less_equal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn greater(
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            let limit = if signed { IntPredicate::SGT } else { IntPredicate::UGT };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(limit, primary.into_int_value(), secondary.into_int_value(), "greater")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(FloatPredicate::OGT, primary.into_float_value(), secondary.into_float_value(), "greater")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }

    pub fn greater_or_equal(
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, [first, second], span)?;

        if !floating {
            let limit = if signed { IntPredicate::SGE } else { IntPredicate::UGE };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(limit, primary.into_int_value(), secondary.into_int_value(), "greater_equal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(FloatPredicate::OGE, primary.into_float_value(), secondary.into_float_value(), "greater_equal")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            ))
        }
    }
}
