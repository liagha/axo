use {
    inkwell::{
        values::BasicValueEnum,
    },

    crate::{
        data::{
            Float, Integer, Boolean, Scale,
        },
        resolver::{
            analyzer::Instruction,
        },
    },
};

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_integer(&self, number: Integer, scale: Scale, signed: Boolean) -> BasicValueEnum<'backend> {
        let kind = match scale {
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            _ => self.context.i64_type()
        };

        let unsigned = number as u64;

        BasicValueEnum::from(kind.const_int(unsigned, false))
    }

    pub fn generate_float(&self, number: Float, scale: Scale) -> BasicValueEnum<'backend> {
        let kind = match scale {
            32 => self.context.f32_type(),
            64 => self.context.f64_type(),
            _ => self.context.f64_type()
        };
        BasicValueEnum::from(kind.const_float(number.0))
    }

    pub fn generate_boolean(&self, value: bool) -> BasicValueEnum<'backend> {
        BasicValueEnum::from(self.context.bool_type().const_int(value as u64, false))
    }
}