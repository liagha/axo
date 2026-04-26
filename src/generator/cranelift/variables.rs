// src/generator/cranelift/variables.rs
use {
    crate::{
        analyzer::Analysis,
        data::{Binding, Str},
        generator::{cranelift::CraneliftGenerator, GenerateError},
        resolver::Type,
        tracker::Span,
    },
    cranelift_codegen::ir::{InstBuilder, Value, types},
};

impl<'backend> CraneliftGenerator<'backend> {
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
