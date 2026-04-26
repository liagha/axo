use {
    crate::{
        data::{Boolean, Char, Float, Integer, Scale, Str},
        generator::{cranelift::CraneliftGenerator, ErrorKind, GenerateError},
        tracker::Span,
    },
    cranelift_codegen::ir::{types, InstBuilder, Value},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn integer(
        &mut self,
        number: Integer,
        scale: Scale,
        _signed: Boolean,
    ) -> Result<Value, GenerateError<'backend>> {
        let kind = match scale {
            8 => types::I8,
            16 => types::I16,
            32 => types::I32,
            64 => types::I64,
            _ => types::I64,
        };

        Ok(self.builder.ins().iconst(kind, number as i64))
    }

    pub fn float(
        &mut self,
        number: Float,
        scale: Scale,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        match scale {
            32 => Ok(self.builder.ins().f32const(number.0 as f32)),
            64 => Ok(self.builder.ins().f64const(number.0)),
            width => Err(GenerateError::new(
                ErrorKind::UnsupportedFloatWidth(width),
                span,
            )),
        }
    }

    pub fn boolean(&mut self, value: bool) -> Result<Value, GenerateError<'backend>> {
        let bits = if value { 1 } else { 0 };
        Ok(self.builder.ins().iconst(types::I8, bits))
    }

    pub fn character(&mut self, value: Char) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I32, value as i64))
    }

    pub fn string(
        &mut self,
        _value: Str<'backend>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }
}
