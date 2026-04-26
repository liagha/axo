// src/generator/cranelift/logical.rs
use {
    crate::{
        analyzer::Analysis,
        generator::{cranelift::CraneliftGenerator, GenerateError},
        tracker::Span,
    },
    cranelift_codegen::ir::{condcodes::IntCC, InstBuilder, Value},
};

impl<'backend> CraneliftGenerator<'backend> {
    pub fn logical_and(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let primary = self.analysis(*left)?;

        let evaluate = self.builder.create_block();
        let merge = self.builder.create_block();

        let typing = self.builder.func.dfg.value_type(primary);
        let temp = self.builder.declare_var(typing);

        self.builder.def_var(temp, primary);
        self.builder.ins().brif(primary, evaluate, &[], merge, &[]);

        self.builder.switch_to_block(evaluate);
        self.builder.seal_block(evaluate);

        let secondary = self.analysis(*right)?;
        self.builder.def_var(temp, secondary);
        self.builder.ins().jump(merge, &[]);

        self.builder.switch_to_block(merge);
        self.builder.seal_block(merge);

        Ok(self.builder.use_var(temp))
    }

    pub fn logical_or(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let primary = self.analysis(*left)?;

        let evaluate = self.builder.create_block();
        let merge = self.builder.create_block();

        let typing = self.builder.func.dfg.value_type(primary);
        let temp = self.builder.declare_var(typing);

        self.builder.def_var(temp, primary);
        self.builder.ins().brif(primary, merge, &[], evaluate, &[]);

        self.builder.switch_to_block(evaluate);
        self.builder.seal_block(evaluate);

        let secondary = self.analysis(*right)?;
        self.builder.def_var(temp, secondary);
        self.builder.ins().jump(merge, &[]);

        self.builder.switch_to_block(merge);
        self.builder.seal_block(merge);

        Ok(self.builder.use_var(temp))
    }

    pub fn logical_not(
        &mut self,
        operand: Box<Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let primary = self.analysis(*operand)?;
        let typing = self.builder.func.dfg.value_type(primary);

        let zero = self.builder.ins().iconst(typing, 0);
        Ok(self.builder.ins().icmp(IntCC::Equal, primary, zero))
    }

    pub fn logical_xor(
        &mut self,
        left: Box<Analysis<'backend>>,
        right: Box<Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let primary = self.analysis(*left)?;
        let secondary = self.analysis(*right)?;

        Ok(self.builder.ins().bxor(primary, secondary))
    }
}
