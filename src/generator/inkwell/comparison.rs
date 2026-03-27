use {
    crate::{
        analyzer::Analysis,
        generator::{Backend, ErrorKind, GenerateError, Generator},
        tracker::Span,
    },
    inkwell::{values::BasicValueEnum, FloatPredicate, IntPredicate},
};

impl<'backend> Generator<'backend> {
    pub fn tag(
        &self,
        value: BasicValueEnum<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if value.is_struct_value() {
            let shape = value.into_struct_value();
            let extract = self
                .builder
                .build_extract_value(shape, 0, "tag")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            Ok(extract)
        } else {
            Ok(value)
        }
    }

    pub fn equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let primary = self.tag(alpha, span)?;
        let secondary = self.tag(beta, span)?;

        let (primary_value, secondary_value, floating) =
            self.normalize(primary, secondary, span)?;

        if !floating {
            let truth = self
                .builder
                .build_int_compare(
                    IntPredicate::EQ,
                    primary_value.into_int_value(),
                    secondary_value.into_int_value(),
                    "equal",
                )
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            Ok(truth.into())
        } else {
            let truth = self
                .builder
                .build_float_compare(
                    FloatPredicate::OEQ,
                    primary_value.into_float_value(),
                    secondary_value.into_float_value(),
                    "equal",
                )
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            Ok(truth.into())
        }
    }

    pub fn not_equal(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let primary = self.tag(alpha, span)?;
        let secondary = self.tag(beta, span)?;

        let (primary_value, secondary_value, floating) =
            self.normalize(primary, secondary, span)?;

        if !floating {
            let truth = self
                .builder
                .build_int_compare(
                    IntPredicate::NE,
                    primary_value.into_int_value(),
                    secondary_value.into_int_value(),
                    "unequal",
                )
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            Ok(truth.into())
        } else {
            let truth = self
                .builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    primary_value.into_float_value(),
                    secondary_value.into_float_value(),
                    "unequal",
                )
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            Ok(truth.into())
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if !floating {
            let limit = if signed {
                IntPredicate::SLT
            } else {
                IntPredicate::ULT
            };
            Ok(BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        limit,
                        primary.into_int_value(),
                        secondary.into_int_value(),
                        "less",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OLT,
                        primary.into_float_value(),
                        secondary.into_float_value(),
                        "less",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if !floating {
            let limit = if signed {
                IntPredicate::SLE
            } else {
                IntPredicate::ULE
            };
            Ok(BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        limit,
                        primary.into_int_value(),
                        secondary.into_int_value(),
                        "less_equal",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OLE,
                        primary.into_float_value(),
                        secondary.into_float_value(),
                        "less_equal",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if !floating {
            let limit = if signed {
                IntPredicate::SGT
            } else {
                IntPredicate::UGT
            };
            Ok(BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        limit,
                        primary.into_int_value(),
                        secondary.into_int_value(),
                        "greater",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OGT,
                        primary.into_float_value(),
                        secondary.into_float_value(),
                        "greater",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
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

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if !floating {
            let limit = if signed {
                IntPredicate::SGE
            } else {
                IntPredicate::UGE
            };
            Ok(BasicValueEnum::from(
                self.builder
                    .build_int_compare(
                        limit,
                        primary.into_int_value(),
                        secondary.into_int_value(),
                        "greater_equal",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
            ))
        } else {
            Ok(BasicValueEnum::from(
                self.builder
                    .build_float_compare(
                        FloatPredicate::OGE,
                        primary.into_float_value(),
                        secondary.into_float_value(),
                        "greater_equal",
                    )
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?,
            ))
        }
    }
}
