use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn negate(
        &mut self,
        value: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let value = self.expr(value)?;
        let kind = self.builder.func.dfg.value_type(value);
        if kind.is_int() {
            Ok(self.builder.ins().ineg(value))
        } else if kind.is_float() {
            Ok(self.builder.ins().fneg(value))
        } else {
            Err(self.error(ErrorKind::Negate, span))
        }
    }

    pub(super) fn add(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.numeric(left, right, span, |this, left, right, float| {
            if float {
                this.builder.ins().fadd(left, right)
            } else {
                this.builder.ins().iadd(left, right)
            }
        })
    }

    pub(super) fn subtract(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.numeric(left, right, span, |this, left, right, float| {
            if float {
                this.builder.ins().fsub(left, right)
            } else {
                this.builder.ins().isub(left, right)
            }
        })
    }

    pub(super) fn multiply(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.numeric(left, right, span, |this, left, right, float| {
            if float {
                this.builder.ins().fmul(left, right)
            } else {
                this.builder.ins().imul(left, right)
            }
        })
    }

    pub(super) fn divide(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        self.numeric(left, right, span, move |this, left, right, float| {
            if float {
                this.builder.ins().fdiv(left, right)
            } else if sign {
                this.builder.ins().sdiv(left, right)
            } else {
                this.builder.ins().udiv(left, right)
            }
        })
    }

    pub(super) fn modulus(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) || kind.is_float() {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        Ok(if sign {
            self.builder.ins().srem(left, right)
        } else {
            self.builder.ins().urem(left, right)
        })
    }

    pub(super) fn numeric<F>(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        apply: F,
    ) -> Result<Value, GenerateError<'b>>
    where
        F: Fn(&mut Self, Value, Value, bool) -> Value,
    {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let left_kind = self.builder.func.dfg.value_type(left);
        let right_kind = self.builder.func.dfg.value_type(right);
        if left_kind != right_kind {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        Ok(apply(self, left, right, left_kind.is_float()))
    }
}
