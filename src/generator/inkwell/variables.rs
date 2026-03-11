use {
    super::{Backend, Entity},
    crate::{
        data::*,
        analyzer::{Analysis, AnalysisKind},
        checker::{Type, TypeKind},
        generator::inkwell::error::VariableError,
        data::Str,
        generator::{ErrorKind, GenerateError},
        tracker::Span,
    },
    inkwell::{
        types::{BasicTypeEnum},
        values::{BasicValueEnum, PointerValue},
    },
};

impl<'backend> super::Inkwell<'backend> {
    fn pointer_pointee_type(&self, analysis: &Analysis<'backend>) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => match self.entities.get(name) {
                Some(Entity::Variable { pointee, .. }) if pointee.is_some() => *pointee,

                Some(Entity::Variable { kind, .. }) if kind.is_pointer_type() => {
                    None
                }
                _ => None,
            },
            AnalysisKind::Dereference(operand) => {
                self.pointer_pointee_type(operand).and_then(|t| {
                    if t.is_pointer_type() {
                        None
                    } else {
                        Some(t)
                    }
                })
            },
            _ => None,
        }
    }

    fn lvalue_pointer(
        &mut self,
        analysis: &Analysis<'backend>,
    ) -> Result<Option<(PointerValue<'backend>, BasicTypeEnum<'backend>)>, GenerateError<'backend>> {
        match &analysis.kind {
            AnalysisKind::Dereference(operand) => {
                let pointee = self.pointer_pointee_type(operand);
                let value = self.analysis(*operand.clone())?;

                match (value, pointee) {
                    (BasicValueEnum::PointerValue(_), None) => {
                        Err(GenerateError::new(
                            ErrorKind::Variable(VariableError::DereferenceNonPointer),
                            analysis.span,
                        ))
                    }
                    (BasicValueEnum::PointerValue(pointer), Some(kind)) => {
                        Ok(Some((pointer, kind)))
                    }
                    (BasicValueEnum::IntValue(addr), Some(kind)) => {
                        let ptr = self.builder.build_int_to_ptr(addr, self.context.ptr_type(inkwell::AddressSpace::default()), "ptr_arith_cast")
                            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, analysis.span))?;
                        Ok(Some((ptr, kind)))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    pub fn address_of(
        &mut self,
        operand: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some((pointer, _)) = self.lvalue_pointer(&operand)? {
            Ok(pointer.into())
        } else {
            Err(GenerateError::new(
                ErrorKind::Variable(VariableError::AddressOfRValue),
                span,
            ))
        }
    }

    pub fn dereference(
        &mut self,
        operand: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let pointee = self.pointer_pointee_type(&operand);
        let value = self.analysis(*operand.clone())?;

        match (value, pointee) {
            (BasicValueEnum::PointerValue(pointer), Some(kind)) => {
                self.builder
                    .build_load(kind, pointer, "deref_value")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))
            }
            _ => Err(GenerateError::new(
                ErrorKind::Variable(VariableError::DereferenceNonPointer),
                span,
            ))
        }
    }

    pub fn usage(
        &self,
        identifier: Str<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(entity) = self.entities.get(&identifier) {
            return match entity {
                Entity::Function(function) => {
                    Ok(BasicValueEnum::from(function.as_global_value().as_pointer_value()))
                }
                Entity::Variable { pointer, kind, .. } => self
                    .builder
                    .build_load(*kind, *pointer, &identifier)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span)),
                _ => Err(GenerateError::new(
                    ErrorKind::Variable(VariableError::NotAValue {
                        name: identifier.to_string(),
                    }),
                    span,
                )),
            };
        }

        if let Some(module) = self.modules.get(&self.current_module) {
            if let Some(global) = module.get_global(&identifier) {
                let basic_type: BasicTypeEnum = match global.get_value_type() {
                    inkwell::types::AnyTypeEnum::ArrayType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::FloatType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::IntType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::PointerType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::StructType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::VectorType(t) => t.into(),
                    _ => {
                        return Err(GenerateError::new(
                            ErrorKind::Variable(VariableError::NotAValue {
                                name: identifier.to_string(),
                            }),
                            span,
                        ));
                    }
                };

                return self.builder
                    .build_load(basic_type, global.as_pointer_value(), &identifier)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span));
            }
        }

        Err(GenerateError::new(
            ErrorKind::Variable(VariableError::Undefined {
                name: identifier.to_string(),
            }),
            span,
        ))
    }

    pub fn assign(
        &mut self,
        target: Str<'backend>,
        value: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let pointee = self.pointer_pointee_type(&value);
        let signed = self.infer_signedness(&value);
        let result = self.analysis(*value)?;

        let existing_pointer = match self.entities.get(&target) {
            Some(Entity::Variable { pointer, .. }) => Some(*pointer),
            _ => None,
        };

        if let Some(slot) = existing_pointer {
            self.builder.build_store(slot, result)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

            self.entities.insert(
                target.clone(),
                Entity::Variable {
                    pointer: slot,
                    kind: result.get_type(),
                    pointee,
                    signed,
                },
            );
        } else {
            let func = self.parent(span)?;
            let pointer = self.build_entry(func, result.get_type(), target.clone());

            self.builder.build_store(pointer, result)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

            self.entities.insert(
                target.clone(),
                Entity::Variable {
                    pointer,
                    kind: result.get_type(),
                    pointee,
                    signed,
                },
            );
        }
        Ok(result)
    }

    pub fn binding(
        &mut self,
        binding: Binding<Str<'backend>, Box<Analysis<'backend>>, Type<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let value = match binding.value {
            Some(v) => v,
            None => {
                return Err(GenerateError::new(
                    ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                        name: binding.target.to_string(),
                    }),
                    span,
                ));
            }
        };

        let pointee = if let Some(annotation) = binding.annotation.as_ref() {
            match &annotation.kind {
                TypeKind::Pointer { target } => {
                    Some(self.llvm_type(target, span)?)
                }
                _ => None,
            }
        } else {
            self.pointer_pointee_type(&value)
        };

        let value = self.analysis(*value)?;

        let declared_kind = if let Some(annotation) = binding.annotation.as_ref() {
            self.llvm_type(annotation, span)?
        } else {
            value.get_type()
        };

        let is_global_scope = self.builder.get_insert_block().is_none();

        let casted = if value.get_type() == declared_kind {
            value
        } else if value.is_int_value() && declared_kind.is_int_type() {
            if is_global_scope {
                value
            } else {
                self.builder
                    .build_int_cast(value.into_int_value(), declared_kind.into_int_type(), "bind_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(value)
            }
        } else if value.is_float_value() && declared_kind.is_float_type() {
            self.builder
                .build_float_cast(value.into_float_value(), declared_kind.into_float_type(), "bind_cast")
                .ok()
                .map(Into::into)
                .unwrap_or(value)
        } else if value.is_pointer_value() && declared_kind.is_int_type() {
            self.builder
                .build_ptr_to_int(value.into_pointer_value(), declared_kind.into_int_type(), "bind_ptr_cast")
                .ok()
                .map(Into::into)
                .unwrap_or(value)
        } else {
            return Err(GenerateError::new(
                ErrorKind::Variable(VariableError::BindingTypeMismatch {
                    name: binding.target.to_string(),
                }),
                span,
            ));
        };

        let signed = binding.annotation.as_ref().and_then(|annotation| match annotation.kind {
            TypeKind::Integer { signed, .. } => Some(signed),
            _ => None,
        });

        let parent_func = self.builder.get_insert_block().and_then(|b| b.get_parent());

        let pointer = if let Some(func) = parent_func {
            let pointer = self.build_entry(func, declared_kind, binding.target.clone());

            self.builder.build_store(pointer, casted)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

            pointer
        } else {
            let module = self.modules.get(&self.current_module).unwrap();
            let global = module.add_global(declared_kind, None, &binding.target);

            global.set_initializer(&casted);
            global.set_constant(true);

            global.as_pointer_value()
        };

        self.entities.insert(
            binding.target.clone(),
            Entity::Variable { pointer, kind: declared_kind, pointee, signed },
        );

        Ok(casted)
    }

    pub fn store(
        &mut self,
        target: Box<Analysis<'backend>>,
        value: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let result = self.analysis(*value.clone())?;

        if let Some((pointer, kind)) = self.lvalue_pointer(&target)? {
            if result.get_type() == kind {
                self.builder.build_store(pointer, result)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
            } else if result.is_int_value() && kind.is_int_type() {
                let casted = self
                    .builder
                    .build_int_cast(result.into_int_value(), kind.into_int_type(), "store_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);

                self.builder.build_store(pointer, casted)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
            } else if result.is_float_value() && kind.is_float_type() {
                let casted = self
                    .builder
                    .build_float_cast(result.into_float_value(), kind.into_float_type(), "store_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);

                self.builder.build_store(pointer, casted)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
            } else if result.is_pointer_value() && kind.is_int_type() {
                let casted = self
                    .builder
                    .build_ptr_to_int(result.into_pointer_value(), kind.into_int_type(), "store_ptr_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);

                self.builder.build_store(pointer, casted)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
            } else {
                return Err(GenerateError::new(
                    ErrorKind::Variable(VariableError::AssignmentTypeMismatch),
                    span,
                ));
            }
        } else {
            return Err(GenerateError::new(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                span,
            ));
        }

        Ok(result)
    }
}
