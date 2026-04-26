// src/generator/cranelift/composite.rs
use {
    crate::{
        analyzer::Analysis,
        data::{Aggregate, Index, Str},
        generator::{cranelift::CraneliftGenerator, GenerateError},
        resolver::Type,
        tracker::Span,
    },
    cranelift_codegen::ir::{InstBuilder, Value, types},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn define_structure(
        &mut self,
        _structure: Aggregate<Str<'backend>, Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn define_union(
        &mut self,
        _structure: Aggregate<Str<'backend>, Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn array(&mut self, _values: Vec<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn tuple(&mut self, _values: Vec<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn access(
        &mut self,
        _target: Box<Analysis<'backend>>,
        _member: Box<Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn index(
        &mut self,
        _index: Index<Box<Analysis<'backend>>, Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn constructor(
        &mut self,
        _typing: Type<'backend>,
        _structure: Aggregate<Str<'backend>, Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn size_of(&mut self, _layout: Type<'backend>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 8))
    }

    pub fn trap(&mut self, condition: Option<Value>, _span: Span) -> Result<(), GenerateError<'backend>> {
        let code = cranelift_codegen::ir::TrapCode::user(0).unwrap();

        if let Some(check) = condition {
            let fail = self.builder.create_block();
            let pass = self.builder.create_block();

            self.builder.ins().brif(check, pass, &[], fail, &[]);

            self.builder.switch_to_block(fail);
            self.builder.seal_block(fail);
            self.builder.ins().trap(code);

            self.builder.switch_to_block(pass);
            self.builder.seal_block(pass);
        } else {
            self.builder.ins().trap(code);
        }

        Ok(())
    }
}
