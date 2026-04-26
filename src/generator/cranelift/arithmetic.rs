use {
    crate::{
        analyzer::Analysis,
        generator::{CraneliftGenerator, ErrorKind, GenerateError},
        tracker::Span,
    },
    cranelift_codegen::ir::{InstBuilder, Value},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn normalize(
        &self,
        left: Value,
        right: Value,
        span: Span,
    ) -> Result<(Value, Value, bool), GenerateError<'backend>> {
        let left_type = self.builder.func.dfg.value_type(left);
        let right_type = self.builder.func.dfg.value_type(right);

        if left_type != right_type {
            return Err(GenerateError::new(ErrorKind::Normalize, span));
        }

        let floating = left_type.is_float();

        Ok((left, right, floating))
    }

    pub fn add(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fadd(primary, secondary))
        } else {
            Ok(self.builder.ins().iadd(primary, secondary))
        }
    }

    pub fn subtract(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fsub(primary, secondary))
        } else {
            Ok(self.builder.ins().isub(primary, secondary))
        }
    }

    pub fn multiply(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let alpha = self.analysis(*left)?;
        let beta = self.analysis(*right)?;

        let (primary, secondary, floating) = self.normalize(alpha, beta, span)?;

        if floating {
            Ok(self.builder.ins().fmul(primary, secondary))
        } else {
            Ok(self.builder.ins().imul(primary, secondary))
        }
    }

    pub fn divide(
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
            Ok(self.builder.ins().fdiv(primary, secondary))
        } else if signed {
            Ok(self.builder.ins().sdiv(primary, secondary))
        } else {
            Ok(self.builder.ins().udiv(primary, secondary))
        }
    }

    pub fn modulus(
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
            Ok(self.builder.ins().srem(primary, secondary))
        } else {
            Ok(self.builder.ins().urem(primary, secondary))
        }
    }
}
