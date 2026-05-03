use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn integer(
        &mut self,
        value: isize,
        size: Scale,
    ) -> Result<Value, GenerateError<'b>> {
        Ok(self.builder.ins().iconst(int_type(size), value as i64))
    }

    pub(super) fn float(
        &mut self,
        value: crate::data::Float,
        size: Scale,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        match size {
            32 => Ok(self.builder.ins().f32const(value.0 as f32)),
            64 => Ok(self.builder.ins().f64const(value.0)),
            width => Err(self.error(ErrorKind::UnsupportedFloatWidth(width), span)),
        }
    }

    pub(super) fn boolean(&mut self, value: bool) -> Result<Value, GenerateError<'b>> {
        Ok(self.builder.ins().iconst(types::I8, i64::from(value)))
    }

    pub(super) fn character(
        &mut self,
        value: crate::data::Char,
    ) -> Result<Value, GenerateError<'b>> {
        Ok(self.builder.ins().iconst(types::I32, value as i64))
    }
}
