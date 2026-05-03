use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn block(&mut self, values: Vec<Analysis<'b>>) -> Result<Value, GenerateError<'b>> {
        let mut last = self.builder.ins().iconst(types::I64, 0);
        for value in values {
            if self.done() {
                break;
            }
            last = self.expr(value)?;
        }
        Ok(last)
    }

    pub(super) fn conditional(
        &mut self,
        typing: Type<'b>,
        condition: Analysis<'b>,
        truth: Analysis<'b>,
        fall: Option<Analysis<'b>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let check = self.expr(condition)?;
        let check = self.truth(check);
        let pass = self.builder.create_block();
        let fail = self.builder.create_block();
        let join = self.builder.create_block();
        self.builder.ins().brif(check, pass, &[], fail, &[]);

        let mut slot = None;
        let mut var = None;

        if indirect(&typing) {
            slot = Some(self.stack(&typing));
        } else if let Some(kind) = scalar_type(&typing, self.pointer) {
            var = Some(self.builder.declare_var(kind));
        }

        self.builder.switch_to_block(pass);
        let left = self.expr(truth)?;
        if let Some(slot) = slot {
            let addr = self.addr(slot);
            self.write(addr, &typing, left);
        }
        if let Some(var) = var {
            self.builder.def_var(var, left);
        }
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(pass);

        self.builder.switch_to_block(fail);
        let right = if let Some(fall) = fall {
            self.expr(fall)?
        } else if indirect(&typing) {
            let slot = self.stack(&typing);
            self.addr(slot)
        } else {
            self.zero(&typing)
        };
        if let Some(slot) = slot {
            let addr = self.addr(slot);
            self.write(addr, &typing, right);
        }
        if let Some(var) = var {
            self.builder.def_var(var, right);
        }
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(fail);

        self.builder.switch_to_block(join);
        self.builder.seal_block(join);

        if let Some(slot) = slot {
            Ok(self.addr(slot))
        } else if let Some(var) = var {
            Ok(self.builder.use_var(var))
        } else {
            Ok(self.zero(&typing))
        }
    }

    pub(super) fn loop_expr(
        &mut self,
        typing: Type<'b>,
        condition: Analysis<'b>,
        body: Analysis<'b>,
    ) -> Result<Value, GenerateError<'b>> {
        let head = self.builder.create_block();
        let core = self.builder.create_block();
        let exit = self.builder.create_block();
        let slot = if matches!(resolved(&typing).kind, TypeKind::Void | TypeKind::Unknown) {
            None
        } else {
            Some(self.stack(&typing))
        };
        self.builder.ins().jump(head, &[]);
        self.builder.switch_to_block(head);
        let check = self.expr(condition)?;
        let check = self.truth(check);
        self.builder.ins().brif(check, core, &[], exit, &[]);
        self.loops.push(Loop { head, exit, slot });

        self.builder.switch_to_block(core);
        let _ = self.expr(body)?;
        if !self.done() {
            self.builder.ins().jump(head, &[]);
        }
        self.builder.seal_block(core);
        self.loops.pop();
        self.builder.seal_block(head);
        self.builder.switch_to_block(exit);
        self.builder.seal_block(exit);
        if let Some(slot) = slot {
            Ok(self.addr(slot))
        } else {
            Ok(self.builder.ins().iconst(types::I64, 0))
        }
    }

    pub(super) fn call(
        &mut self,
        target: Target<'b>,
        values: Vec<Analysis<'b>>,
        typing: &Type<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let Some(Entity::Function(func)) = self.entities.get(&target.name).cloned() else {
            return Err(self.error(
                ErrorKind::Function(FunctionError::Undefined {
                    name: target.name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            ));
        };
        let callee = self
            .module
            .declare_func_in_func(func.id, &mut self.builder.func);
        let mut args = Vec::new();
        let mut slot = None;
        if func.indirect {
            let temp = self.stack(typing);
            let addr = self.addr(temp);
            slot = Some(temp);
            args.push(addr);
        }
        for value in values {
            let value = self.expr(value)?;
            args.push(value);
        }
        let call = self.builder.ins().call(callee, &args);
        if let Some(slot) = slot {
            Ok(self.addr(slot))
        } else if let Some(value) = self.builder.inst_results(call).first().copied() {
            Ok(value)
        } else {
            Ok(self.builder.ins().iconst(types::I64, 0))
        }
    }

    pub(super) fn return_value(
        &mut self,
        value: Option<Analysis<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        if self.done() {
            return Ok(self.builder.ins().iconst(types::I64, 0));
        }
        match (self.func.output.clone(), value) {
            (Some(output), Some(value)) => {
                let value = self.expr(value)?;
                if self.func.indirect {
                    let Some(ret) = self.ret else {
                        return Err(self.error(
                            ErrorKind::Function(FunctionError::IncompatibleReturnType),
                            span,
                        ));
                    };
                    self.write(ret, &output, value);
                    self.builder.ins().return_(&[]);
                } else {
                    self.builder.ins().return_(&[value]);
                }
                Ok(value)
            }
            (None, _) => {
                self.builder.ins().return_(&[]);
                Ok(self.builder.ins().iconst(types::I64, 0))
            }
            _ => Err(self.error(
                ErrorKind::Function(FunctionError::IncompatibleReturnType),
                span,
            )),
        }
    }

    pub(super) fn break_value(
        &mut self,
        value: Option<Analysis<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let looped = self.loops.last().copied().ok_or_else(|| {
            self.error(
                ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                span,
            )
        })?;
        if let (Some(value), Some(slot)) = (value, looped.slot) {
            let value = self.expr(value)?;
            let addr = self.addr(slot);
            self.builder.ins().store(MemFlags::new(), value, addr, 0);
        }
        if !self.done() {
            self.builder.ins().jump(looped.exit, &[]);
        }
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub(super) fn continue_value(&mut self, span: Span) -> Result<Value, GenerateError<'b>> {
        let looped = self.loops.last().copied().ok_or_else(|| {
            self.error(
                ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                span,
            )
        })?;
        if !self.done() {
            self.builder.ins().jump(looped.head, &[]);
        }
        Ok(self.builder.ins().iconst(types::I64, 0))
    }
}
