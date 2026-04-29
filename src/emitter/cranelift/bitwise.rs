use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn bitwise<F>(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        apply: F,
    ) -> Result<Value, GenerateError<'b>>
    where
        F: Fn(&mut Self, Value, Value) -> Value,
    {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) || kind.is_float() {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        Ok(apply(self, left, right))
    }
}
