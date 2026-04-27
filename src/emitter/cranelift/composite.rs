// src/generator/cranelift/composite.rs
use {
    crate::{
        analyzer::{Analysis, Target},
        data::{Aggregate, Index, Scale, Str},
        generator::{cranelift::CraneliftGenerator, GenerateError},
        resolver::Type,
        tracker::Span,
    },
    cranelift_codegen::ir::{InstBuilder, Value, types},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn slot(
        &mut self,
        target: Box<Analysis<'backend>>,
        index: Scale,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let value = Analysis::new(
            crate::analyzer::AnalysisKind::Index(Index::new(
                target,
                vec![Analysis::new(
                    crate::analyzer::AnalysisKind::Integer {
                        value: index as isize,
                        size: 64,
                        signed: true,
                    },
                    span,
                    crate::resolver::Type::from(crate::resolver::TypeKind::Integer {
                        size: 64,
                        signed: true,
                    }),
                )],
            )),
            span,
            crate::resolver::Type::from(crate::resolver::TypeKind::Unknown),
        );
        self.analysis(value)
    }

    pub fn pack(
        &mut self,
        typing: Type<'backend>,
        target: Target<'backend>,
        values: Vec<(Scale, Analysis<'backend>)>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let mut members = Vec::with_capacity(values.len());
        for (_, value) in values {
            members.push(value);
        }
        self.constructor(typing, Aggregate::new(target.name, members), span)
    }

    pub fn composite(
        &mut self,
        composite: Aggregate<Target<'backend>, Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        self.define_structure(Aggregate::new(composite.target.name, composite.members), span)
    }

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
