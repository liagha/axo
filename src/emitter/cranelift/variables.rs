use super::*;

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    pub(super) fn bind(
        &mut self,
        value: Binding<Box<Analysis<'b>>, Box<Analysis<'b>>, Type<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let AnalysisKind::Symbol(target) = &value.target.kind else {
            return Err(self.error(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                span,
            ));
        };
        let slot = self.stack(&value.annotation);
        let addr = self.addr(slot);
        if let Some(init) = value.value {
            let init = self.expr(*init)?;
            self.write(addr, &value.annotation, init);
        } else if matches!(value.kind, BindingKind::Let) {
            return Err(self.error(
                ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                    name: target.name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            ));
        }
        self.entities.insert(
            target.name,
            Entity::Variable {
                slot,
                typing: value.annotation.clone(),
            },
        );
        if indirect(&value.annotation) {
            Ok(addr)
        } else {
            self.load(addr, &value.annotation)
        }
    }

    pub(super) fn read(&mut self, name: Str<'b>, span: Span) -> Result<Value, GenerateError<'b>> {
        match self.entities.get(&name).cloned() {
            Some(Entity::Variable { slot, typing }) => {
                let addr = self.addr(slot);
                if indirect(&typing) {
                    Ok(addr)
                } else {
                    self.load(addr, &typing)
                }
            }
            Some(Entity::Function(_)) => Err(self.error(
                ErrorKind::Function(FunctionError::Undefined {
                    name: name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            )),
            _ => Err(self.error(
                ErrorKind::Variable(VariableError::Undefined {
                    name: name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            )),
        }
    }

    pub(super) fn assign(
        &mut self,
        name: Str<'b>,
        value: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let target = Analysis::new(AnalysisKind::Usage(name), span, value.typing.clone());
        self.store_target(target, value, span)
    }

    pub(super) fn write_target(
        &mut self,
        target: Target<'b>,
        value: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.store_target(
            Analysis::new(AnalysisKind::Symbol(target), span, value.typing.clone()),
            value,
            span,
        )
    }

    pub(super) fn store_target(
        &mut self,
        target: Analysis<'b>,
        value: Analysis<'b>,
        _span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let (addr, typing) = self.place(&target)?;
        let value = self.expr(value)?;
        self.write(addr, &typing, value);
        if indirect(&typing) {
            Ok(addr)
        } else {
            self.load(addr, &typing)
        }
    }
}