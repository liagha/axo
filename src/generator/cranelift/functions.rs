// src/generator/cranelift/functions.rs
use {
    crate::{
        analyzer::Analysis,
        data::{Function, Invoke, Str},
        generator::{
            cranelift::CraneliftGenerator,
            ErrorKind, GenerateError,
        },
        tracker::Span,
    },
    cranelift_codegen::ir::{
        condcodes::{FloatCC, IntCC},
        InstBuilder, Value, types,
    },
};

impl<'backend> CraneliftGenerator<'backend> {
    fn truth(&mut self, value: Value, _span: Span) -> Result<Value, GenerateError<'backend>> {
        let typing = self.builder.func.dfg.value_type(value);

        if typing.is_int() {
            let zero = self.builder.ins().iconst(typing, 0);
            Ok(self.builder.ins().icmp(IntCC::NotEqual, value, zero))
        } else if typing == types::F32 || typing == types::F64 {
            let zero = self.builder.ins().f32const(0.0);
            Ok(self.builder.ins().fcmp(FloatCC::NotEqual, value, zero))
        } else {
            Ok(self.builder.ins().iconst(types::I8, 0))
        }
    }

    pub fn block(&mut self, analyses: Vec<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        let mut value = self.builder.ins().iconst(types::I64, 0);

        for analysis in analyses {
            if self.builder.is_unreachable() {
                break;
            }
            value = self.analysis(analysis)?;
        }

        Ok(value)
    }

    pub fn conditional(
        &mut self,
        condition: Analysis<'backend>,
        truth: Analysis<'backend>,
        fall: Option<Analysis<'backend>>,
        span: Span,
        needed: bool,
    ) -> Result<Value, GenerateError<'backend>> {
        let check = self.analysis(condition)?;
        let flag = self.truth(check, span)?;

        let pass = self.builder.create_block();
        let fail = self.builder.create_block();
        let merge = self.builder.create_block();
        let mut temp = None;

        self.builder.ins().brif(flag, pass, &[], fail, &[]);

        self.builder.switch_to_block(pass);
        self.builder.seal_block(pass);

        let left = self.analysis(truth)?;

        if needed {
            let typing = self.builder.func.dfg.value_type(left);
            let slot = self.builder.declare_var(typing);
            self.builder.def_var(slot, left);
            temp = Some(slot);
        }

        self.builder.ins().jump(merge, &[]);

        self.builder.switch_to_block(fail);
        self.builder.seal_block(fail);

        let right = if let Some(expression) = fall {
            self.analysis(expression)?
        } else {
            let typing = self.builder.func.dfg.value_type(left);
            self.builder.ins().iconst(typing, 0)
        };

        if let Some(slot) = temp {
            self.builder.def_var(slot, right);
        }

        self.builder.ins().jump(merge, &[]);

        self.builder.switch_to_block(merge);
        self.builder.seal_block(merge);

        if !needed {
            return Ok(left);
        }

        Ok(self.builder.use_var(temp.unwrap()))
    }

    pub fn r#while(
        &mut self,
        condition: Box<Analysis<'backend>>,
        body: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        let start = self.builder.create_block();
        let core = self.builder.create_block();
        let exit = self.builder.create_block();

        self.builder.ins().jump(start, &[]);

        self.builder.switch_to_block(start);
        let check = self.analysis(*condition)?;
        let flag = self.truth(check, span)?;

        self.builder.ins().brif(flag, core, &[], exit, &[]);

        self.builder.switch_to_block(core);
        self.builder.seal_block(core);

        self.analysis(*body)?;

        self.builder.ins().jump(start, &[]);
        self.builder.seal_block(start);

        self.builder.switch_to_block(exit);
        self.builder.seal_block(exit);

        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn negate(&mut self, value: Box<Analysis<'backend>>, span: Span) -> Result<Value, GenerateError<'backend>> {
        let check = self.analysis(*value)?;
        let typing = self.builder.func.dfg.value_type(check);

        if typing.is_int() {
            Ok(self.builder.ins().ineg(check))
        } else if typing.is_float() {
            Ok(self.builder.ins().fneg(check))
        } else {
            Err(GenerateError::new(ErrorKind::Negate, span))
        }
    }

    pub fn define_function(
        &mut self,
        _function: Function<
            Str<'backend>,
            Analysis<'backend>,
            Option<Box<Analysis<'backend>>>,
            Option<crate::resolver::Type<'backend>>,
        >,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn module(&mut self, _name: Str<'backend>, _analyses: Vec<Analysis<'backend>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn invoke(
        &mut self,
        _invoke: Invoke<Box<Analysis<'backend>>, Analysis<'backend>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn r#return(&mut self, _value: Option<Box<Analysis<'backend>>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn r#break(&mut self, _value: Option<Box<Analysis<'backend>>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn r#continue(&mut self, _value: Option<Box<Analysis<'backend>>>, _span: Span) -> Result<Value, GenerateError<'backend>> {
        Ok(self.builder.ins().iconst(types::I64, 0))
    }
}
