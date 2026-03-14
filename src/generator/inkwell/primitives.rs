use {
    super::{
        Generator,
    },
    crate::data::{Boolean, Char, Float, Integer, Scale, Str},
    inkwell::values::BasicValueEnum,
};
use crate::generator::{ErrorKind, GenerateError};
use crate::tracker::Span;

impl<'backend> Generator<'backend> {
    pub fn integer(
        &self,
        number: Integer,
        scale: Scale,
        signed: Boolean,
    ) -> BasicValueEnum<'backend> {
        let kind = match scale {
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            bits => self.context.custom_width_int_type(bits as u32),
        };

        let bits = number as u64;

        BasicValueEnum::from(kind.const_int(bits, signed))
    }

    pub fn float(&self, number: Float, scale: Scale, span: Span<'backend>) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let kind = match scale {
            32 => self.context.f32_type(),
            64 => self.context.f64_type(),
            width => {
                return Err(
                    GenerateError::new(
                        ErrorKind::UnsupportedFloatWidth(width),
                        span
                    )
                )
            },
        };

        Ok(BasicValueEnum::from(kind.const_float(number.0)))
    }

    pub fn boolean(&self, value: bool) -> BasicValueEnum<'backend> {
        BasicValueEnum::from(self.context.bool_type().const_int(value as u64, false))
    }

    pub fn character(&self, value: Char) -> BasicValueEnum<'backend> {
        BasicValueEnum::from(self.context.i32_type().const_int(value as u64, false))
    }

    pub fn string(&self, value: Str<'backend>, span: Span<'backend>) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let raw = value.as_str().unwrap_or("");

        let pointer = self
            .builder
            .build_global_string_ptr(raw, "string_literal")
            .map(|value| value.as_pointer_value())
            .map_err(|error| {
                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
            })?;

        Ok(BasicValueEnum::from(pointer))
    }
}
