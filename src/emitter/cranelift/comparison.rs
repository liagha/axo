use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn compare(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        float: FloatCC,
        ints: IntCC,
        _uints: IntCC,
    ) -> Result<Value, GenerateError<'b>> {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        let value = if kind.is_float() {
            self.builder.ins().fcmp(float, left, right)
        } else {
            self.builder.ins().icmp(ints, left, right)
        };
        Ok(self.cast_bool(value))
    }

    pub(super) fn ordered(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        float: FloatCC,
        ints: IntCC,
        uints: IntCC,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        let value = if kind.is_float() {
            self.builder.ins().fcmp(float, left, right)
        } else if sign {
            self.builder.ins().icmp(ints, left, right)
        } else {
            self.builder.ins().icmp(uints, left, right)
        };
        Ok(self.cast_bool(value))
    }
}
