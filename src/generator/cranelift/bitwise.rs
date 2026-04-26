use {
    crate::{
        analyzer::Analysis,
        generator::{CraneliftGenerator, ErrorKind, GenerateError},
        tracker::Span,
    },
    cranelift_codegen::ir::{InstBuilder, Value},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn bitwise_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        Ok(self.builder.ins().band(primary, secondary))
    }

    pub fn bitwise_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        Ok(self.builder.ins().bor(primary, secondary))
    }

    pub fn bitwise_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*operand)?;
        let typing = self.builder.func.dfg.value_type(alpha);

        if typing.is_float() {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        Ok(self.builder.ins().bnot(alpha))
    }

    pub fn bitwise_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        Ok(self.builder.ins().bxor(primary, secondary))
    }

    pub fn shift_left(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        Ok(self.builder.ins().ishl(primary, secondary))
    }

    pub fn shift_right(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let signed = self.infer_signedness(&left).unwrap_or(true)
            && self.infer_signedness(&right).unwrap_or(true);

        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        if signed {
            Ok(self.builder.ins().sshr(primary, secondary))
        } else {
            Ok(self.builder.ins().ushr(primary, secondary))
        }
    }
}
