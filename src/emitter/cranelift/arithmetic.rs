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
        let left_val = self.expr(left)?;
        let right_val = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left_val);
        if kind != self.builder.func.dfg.value_type(right_val) {
            return Err(self.error(ErrorKind::Normalize, span));
        }

        if kind.is_float() {
            return Ok(self.builder.ins().fdiv(left_val, right_val));
        }

        let zero = self.builder.ins().iconst(kind, 0);
        let is_zero = self
            .builder
            .ins()
            .icmp(IntCC::Equal, right_val, zero);
        self.trap_if(is_zero, TrapCode::INTEGER_DIVISION_BY_ZERO);

        if sign {
            let minus_one = self.builder.ins().iconst(kind, -1);
            let is_neg = self
                .builder
                .ins()
                .icmp(IntCC::Equal, right_val, minus_one);
            let bitwidth = kind.bits() as u8;
            let min_shift = self.builder.ins().iconst(types::I8, (bitwidth - 1) as i64);
            let one = self.builder.ins().iconst(kind, 1);
            let min_val = self.builder.ins().ishl(one, min_shift);
            let is_min = self
                .builder
                .ins()
                .icmp(IntCC::Equal, left_val, min_val);
            let overflow = self.builder.ins().band(is_neg, is_min);
            self.trap_if(overflow, TrapCode::INTEGER_OVERFLOW);
            Ok(self.builder.ins().sdiv(left_val, right_val))
        } else {
            Ok(self.builder.ins().udiv(left_val, right_val))
        }
    }

    pub(super) fn modulus(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        let left_val = self.expr(left)?;
        let right_val = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left_val);
        if kind != self.builder.func.dfg.value_type(right_val) || kind.is_float() {
            return Err(self.error(ErrorKind::Normalize, span));
        }

        let zero = self.builder.ins().iconst(kind, 0);
        let is_zero = self
            .builder
            .ins()
            .icmp(IntCC::Equal, right_val, zero);
        self.trap_if(is_zero, TrapCode::INTEGER_DIVISION_BY_ZERO);

        if sign {
            let minus_one = self.builder.ins().iconst(kind, -1);
            let is_neg = self
                .builder
                .ins()
                .icmp(IntCC::Equal, right_val, minus_one);
            let bitwidth = kind.bits() as u8;
            let min_shift = self.builder.ins().iconst(types::I8, (bitwidth - 1) as i64);
            let one = self.builder.ins().iconst(kind, 1);
            let min_val = self.builder.ins().ishl(one, min_shift);
            let is_min = self
                .builder
                .ins()
                .icmp(IntCC::Equal, left_val, min_val);
            let overflow = self.builder.ins().band(is_neg, is_min);
            self.trap_if(overflow, TrapCode::INTEGER_OVERFLOW);
            Ok(self.builder.ins().srem(left_val, right_val))
        } else {
            Ok(self.builder.ins().urem(left_val, right_val))
        }
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