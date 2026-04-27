// src/generator/cranelift/variables.rs
use {
    crate::{
        analyzer::{Analysis, Target},
        data::{Binding, Str},
        generator::{cranelift::CraneliftGenerator, GenerateError},
        resolver::Type,
        tracker::Span,
    },
    cranelift_codegen::ir::{InstBuilder, Value, types},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn symbol_value(&mut self, target: Target<'backend>, span: Span) -> Result<Value, GenerateError<'backend>> {
        self.usage(target.name, span)
    }

    pub fn write(&mut self, target: Target<'backend>, value: Box<Analysis<'backend>>, span: Span) -> Result<Value, GenerateError<'backend>> {
        self.assign(target.name, value, span)
    }

    pub fn address_of(&mut self, _operand: Box<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn dereference(&mut self, _operand: Box<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn usage(&mut self, _identifier: Str<'backend>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn assign(&mut self, _target: Str<'backend>, _value: Box<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn store(&mut self, _target: Box<Analysis<'backend>>, _value: Box<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn binding(
        &mut self,
        _binding: Binding<Box<Analysis<'backend>>, Box<Analysis<'backend>>, Type<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }
}
