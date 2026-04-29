use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn string(&mut self, value: Str<'b>, span: Span) -> Result<Value, GenerateError<'b>> {
        let text = value.as_str().unwrap_or_default();
        let name = format!(".str.{}", self.builder.func.dfg.num_values());
        let id = self
            .module
            .declare_data(&name, Linkage::Local, false, false)
            .map_err(|error| self.error(ErrorKind::Verification(error.to_string()), span))?;
        let mut data = cranelift_module::DataDescription::new();
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        data.define(bytes.into_boxed_slice());
        self.module
            .define_data(id, &data)
            .map_err(|error| self.error(ErrorKind::Verification(error.to_string()), span))?;
        let value = self.module.declare_data_in_func(id, &mut self.builder.func);
        Ok(self.builder.ins().global_value(self.pointer, value))
    }

    pub(super) fn array(
        &mut self,
        typing: &Type<'b>,
        values: Vec<Analysis<'b>>,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        if let TypeKind::Array { member, .. } = &resolved(typing).kind {
            let step = layout(member).size;
            for (index, value) in values.into_iter().enumerate() {
                let item = if step == 0 {
                    addr
                } else {
                    self.builder
                        .ins()
                        .iadd_imm(addr, (index as u32 * step) as i64)
                };
                let value = self.expr(value)?;
                self.write(item, member, value);
            }
        }
        Ok(addr)
    }

    pub(super) fn tuple(
        &mut self,
        typing: &Type<'b>,
        values: Vec<Analysis<'b>>,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        if let TypeKind::Tuple { members } = &resolved(typing).kind {
            for (index, value) in values.into_iter().enumerate() {
                if let Some(item) = members.get(index) {
                    let offs = field_offset(typing, index).unwrap_or(0);
                    let place = if offs == 0 {
                        addr
                    } else {
                        self.builder.ins().iadd_imm(addr, offs as i64)
                    };
                    let value = self.expr(value)?;
                    self.write(place, item, value);
                }
            }
        }
        Ok(addr)
    }

    pub(super) fn constructor(
        &mut self,
        typing: &Type<'b>,
        value: Aggregate<Str<'b>, Analysis<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        let names = member_names_of(typing);
        for (index, item) in value.members.into_iter().enumerate() {
            match item.kind {
                AnalysisKind::Assign(name, value) => {
                    if let Some(slot) = names.iter().position(|item| *item == name) {
                        let place = self.field_addr(addr, typing, slot, span)?;
                        let item = field_type(typing, slot).unwrap();
                        let value = self.expr(*value)?;
                        self.write(place, &item, value);
                    }
                }
                _ => {
                    let place = self.field_addr(addr, typing, index, span)?;
                    let field = field_type(typing, index).unwrap();
                    let value = self.expr(item)?;
                    self.write(place, &field, value);
                }
            }
        }
        Ok(addr)
    }

    pub(super) fn pack(
        &mut self,
        typing: &Type<'b>,
        _target: Target<'b>,
        values: Vec<(Scale, Analysis<'b>)>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        for (index, value) in values {
            let slot = index as usize;
            let place = self.field_addr(addr, typing, slot, span)?;
            let item = field_type(typing, slot).unwrap();
            let value = self.expr(value)?;
            self.write(place, &item, value);
        }
        Ok(addr)
    }

    pub(super) fn field_addr(
        &mut self,
        addr: Value,
        typing: &Type<'b>,
        index: usize,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let offs = field_offset(typing, index).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    target: String::new(),
                    member: index.to_string(),
                }),
                span,
            )
        })?;
        Ok(if offs == 0 {
            addr
        } else {
            self.builder.ins().iadd_imm(addr, offs as i64)
        })
    }
}
