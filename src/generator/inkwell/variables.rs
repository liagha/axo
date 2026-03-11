use inkwell::types::BasicType;
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
        types::BasicTypeEnum,
        values::{BasicValueEnum, PointerValue},
    },
};

impl<'backend> super::Inkwell<'backend> {
    fn pointer_pointee_type(&self, analysis: &Analysis<'backend>) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => match self.get_entity(name) {
                Some(Entity::Variable { pointee, .. }) if pointee.is_some() => *pointee,
                _ => None,
            },
            AnalysisKind::Dereference(operand) => {
                self.pointer_pointee_type(operand)
            },
            _ => None,
        }
    }

    fn lvalue_pointer(
        &mut self,
        analysis: &Analysis<'backend>,
    ) -> Result<Option<(PointerValue<'backend>, BasicTypeEnum<'backend>)>, GenerateError<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => {
                if let Some(entity) = self.get_entity(name) {
                    match entity {
                        Entity::Variable { pointer, kind, .. } => {
                            Ok(Some((*pointer, *kind)))
                        }
                        Entity::Function(func) => {
                            let ptr = func.as_global_value().as_pointer_value();
                            Ok(Some((ptr, ptr.get_type().into())))
                        }
                        _ => Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
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
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), analysis.span))?;
                        Ok(Some((ptr, kind)))
                    }
                    _ => Ok(None),
                }
            }
            AnalysisKind::Access(target, member) => {
                let field_name = if let AnalysisKind::Usage(identifier) = &member.kind {
                    identifier.clone()
                } else {
                    return Ok(None);
                };

                if let Some((base_ptr, base_kind)) = self.lvalue_pointer(target)? {
                    if base_kind.is_struct_type() {
                        let shape = base_kind.into_struct_type();

                        let mut found = None;
                        for scope in self.entities.iter().rev() {
                            for entity in scope.values() {
                                if let Entity::Struct { struct_type: defined, fields } = entity {
                                    if defined.as_basic_type_enum() == shape.as_basic_type_enum() {
                                        found = Some(fields.clone());
                                        break;
                                    }
                                }
                            }
                            if found.is_some() { break; }
                        }

                        if let Some(fields) = found {
                            if let Some(index) = fields.iter().position(|item| item == &field_name) {
                                let slot = self.builder.build_struct_gep(
                                    shape,
                                    base_ptr,
                                    index as u32,
                                    "pointer",
                                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), analysis.span))?;

                                let resolved = shape.get_field_type_at_index(index as u32).unwrap();
                                return Ok(Some((slot, resolved)));
                            }
                        }
                    }
                }

                Ok(None)
            }
            AnalysisKind::Index(index) => {
                if let Some((base_ptr, base_kind)) = self.lvalue_pointer(&index.target)? {
                    if index.members.is_empty() {
                        return Ok(None);
                    }

                    let offset = self.analysis(index.members[0].clone())?;

                    if base_kind.is_struct_type() {
                        if let BasicValueEnum::IntValue(integer) = offset {
                            if let Some(constant) = integer.get_zero_extended_constant() {
                                let shape = base_kind.into_struct_type();
                                let slot = self.builder.build_struct_gep(
                                    shape,
                                    base_ptr,
                                    constant as u32,
                                    "index_ptr",
                                ).map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), analysis.span))?;

                                let resolved = shape.get_field_type_at_index(constant as u32).unwrap();
                                return Ok(Some((slot, resolved)));
                            }
                        }
                    } else if base_kind.is_array_type() {
                        if let BasicValueEnum::IntValue(integer) = offset {
                            let shape = base_kind.into_array_type();
                            let element_type = shape.get_element_type();
                            let zero = self.context.i32_type().const_zero();
                            let slot = unsafe {
                                self.builder
                                    .build_in_bounds_gep(shape, base_ptr, &[zero, integer], "index_ptr")
                                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), analysis.span))?
                            };

                            return Ok(Some((slot, element_type)));
                        }
                    }
                }

                Ok(None)
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
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
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
        if let Some(entity) = self.get_entity(&identifier) {
            return match entity {
                Entity::Function(function) => {
                    Ok(BasicValueEnum::from(function.as_global_value().as_pointer_value()))
                }
                Entity::Variable { pointer, kind, .. } => {
                    if kind.is_array_type() || kind.is_struct_type() {
                        Ok(BasicValueEnum::from(*pointer))
                    } else {
                        self.builder
                            .build_load(*kind, *pointer, &identifier)
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                    }
                },
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
                    inkwell::types::AnyTypeEnum::ArrayType(_) | inkwell::types::AnyTypeEnum::StructType(_) => {
                        return Ok(BasicValueEnum::from(global.as_pointer_value()));
                    }
                    inkwell::types::AnyTypeEnum::FloatType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::IntType(t) => t.into(),
                    inkwell::types::AnyTypeEnum::PointerType(t) => t.into(),
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
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
            }

            if let Some(function) = module.get_function(&identifier) {
                return Ok(BasicValueEnum::from(function.as_global_value().as_pointer_value()));
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

        let existing_pointer = match self.get_entity(&target) {
            Some(Entity::Variable { pointer, .. }) => Some(*pointer),
            _ => None,
        };

        if let Some(slot) = existing_pointer {
            self.builder.build_store(slot, result)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            let mut updated = false;
            for scope in self.entities.iter_mut().rev() {
                if scope.contains_key(&target) {
                    scope.insert(
                        target.clone(),
                        Entity::Variable {
                            pointer: slot,
                            kind: result.get_type(),
                            pointee,
                            signed,
                        },
                    );
                    updated = true;
                    break;
                }
            }
            if !updated {
                self.insert_entity(target.clone(), Entity::Variable { pointer: slot, kind: result.get_type(), pointee, signed });
            }
        } else {
            return Err(GenerateError::new(
                ErrorKind::Variable(VariableError::Undefined {
                    name: target.to_string(),
                }),
                span,
            ));
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

        let is_global_scope = self.builder.get_insert_block().is_none();

        let dummy_func = if is_global_scope {
            let void_type = self.context.void_type();
            let fn_type = void_type.fn_type(&[], false);
            let func = self.modules.get(&self.current_module).unwrap().add_function("__init_temp", fn_type, None);
            let block = self.context.append_basic_block(func, "entry");
            self.builder.position_at_end(block);
            Some(func)
        } else {
            None
        };

        let value = self.analysis(*value)?;

        if let Some(func) = dummy_func {
            self.builder.clear_insertion_position();
            unsafe { func.delete(); }
        }

        let declared_kind = if let Some(annotation) = binding.annotation.as_ref() {
            self.llvm_type(annotation, span)?
        } else {
            value.get_type()
        };

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
            if is_global_scope {
                value
            } else {
                self.builder
                    .build_float_cast(value.into_float_value(), declared_kind.into_float_type(), "bind_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(value)
            }
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

        let pointer = if !is_global_scope {
            let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let pointer = self.build_entry(func, declared_kind, binding.target.clone());

            self.builder.build_store(pointer, casted)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            pointer
        } else {
            let module = self.modules.get(&self.current_module).unwrap();
            let global = module.add_global(declared_kind, None, &binding.target);

            global.set_initializer(&casted);
            global.set_constant(false);

            global.as_pointer_value()
        };

        self.insert_entity(
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
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            } else if result.is_int_value() && kind.is_int_type() {
                let casted = self
                    .builder
                    .build_int_cast(result.into_int_value(), kind.into_int_type(), "store_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);

                self.builder.build_store(pointer, casted)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            } else if result.is_float_value() && kind.is_float_type() {
                let casted = self
                    .builder
                    .build_float_cast(result.into_float_value(), kind.into_float_type(), "store_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);

                self.builder.build_store(pointer, casted)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
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
