use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn logical_and(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
    ) -> Result<Value, GenerateError<'b>> {
        let left = self.expr(left)?;
        let left = self.truth(left);
        let pass = self.builder.create_block();
        let join = self.builder.create_block();
        let temp = self.builder.declare_var(types::I8);
        let zero = self.builder.ins().iconst(types::I8, 0);
        self.builder.def_var(temp, zero);
        self.builder.ins().brif(left, pass, &[], join, &[]);
        self.builder.switch_to_block(pass);
        let right = self.expr(right)?;
        let right = self.truth(right);
        let right = self.cast_bool(right);
        self.builder.def_var(temp, right);
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(pass);
        self.builder.switch_to_block(join);
        self.builder.seal_block(join);
        Ok(self.builder.use_var(temp))
    }

    pub(super) fn logical_or(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
    ) -> Result<Value, GenerateError<'b>> {
        let left = self.expr(left)?;
        let left = self.truth(left);
        let pass = self.builder.create_block();
        let join = self.builder.create_block();
        let temp = self.builder.declare_var(types::I8);
        let one = self.builder.ins().iconst(types::I8, 1);
        self.builder.def_var(temp, one);
        self.builder.ins().brif(left, join, &[], pass, &[]);
        self.builder.switch_to_block(pass);
        let right = self.expr(right)?;
        let right = self.truth(right);
        let right = self.cast_bool(right);
        self.builder.def_var(temp, right);
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(pass);
        self.builder.switch_to_block(join);
        self.builder.seal_block(join);
        Ok(self.builder.use_var(temp))
    }
}
