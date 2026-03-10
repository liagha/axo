use {
    super::{
        Backend,
        GenerateError,
        super::ErrorKind,
    },
    crate::{
        analyzer::Analysis,
    },
    inkwell::{
        values::BasicValueEnum,
        FloatPredicate, IntPredicate,
    },
};
use crate::tracker::Span;

impl<'backend> super::Inkwell<'backend> {
    pub fn equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "equal", span)?;

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(
                    IntPredicate::EQ, left.into_int_value(), right.into_int_value(), "equal",
                ).map_err(
                    |error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(
                    FloatPredicate::OEQ, left.into_float_value(), right.into_float_value(), "equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        }
    }

    pub fn not_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "not_equal", span)?;

        if !floating {
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(
                    IntPredicate::NE, left.into_int_value(), right.into_int_value(), "not_equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(
                    FloatPredicate::ONE, left.into_float_value(), right.into_float_value(), "not_equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        }
    }

    pub fn less(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);
        let signed = left_signed && right_signed;

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "less", span)?;

        if !floating {
            let predicate = if signed { IntPredicate::SLT } else { IntPredicate::ULT };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(
                    predicate, left.into_int_value(), right.into_int_value(), "less",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(
                    FloatPredicate::OLT, left.into_float_value(), right.into_float_value(), "less",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        }
    }

    pub fn less_or_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);
        let signed = left_signed && right_signed;

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "less_or_equal", span)?;

        if !floating {
            let predicate = if signed { IntPredicate::SLE } else { IntPredicate::ULE };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(
                    predicate, left.into_int_value(), right.into_int_value(), "less_or_equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(
                    FloatPredicate::OLE, left.into_float_value(), right.into_float_value(), "less_or_equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        }
    }

    pub fn greater(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);
        let signed = left_signed && right_signed;

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "greater", span)?;

        if !floating {
            let predicate = if signed { IntPredicate::SGT } else { IntPredicate::UGT };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(
                    predicate, left.into_int_value(), right.into_int_value(), "greater",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(
                    FloatPredicate::OGT, left.into_float_value(), right.into_float_value(), "greater",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        }
    }

    pub fn greater_or_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let left_signed = self.infer_signedness(&left).unwrap_or(true);
        let right_signed = self.infer_signedness(&right).unwrap_or(true);
        let signed = left_signed && right_signed;

        let left = self.analysis(*left)?;
        let right = self.analysis(*right)?;

        let (left, right, floating) = self.normalize_pair(left, right, left_signed, right_signed, "greater_or_equal", span)?;

        if !floating {
            let predicate = if signed { IntPredicate::SGE } else { IntPredicate::UGE };
            Ok(BasicValueEnum::from(
                self.builder.build_int_compare(
                    predicate, left.into_int_value(), right.into_int_value(), "greater_or_equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder.build_float_compare(
                    FloatPredicate::OGE, left.into_float_value(), right.into_float_value(), "greater_or_equal",
                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            ))
        }
    }
}
