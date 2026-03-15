use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::Str,
        data::*,
        generator::{
            inkwell::{
                Backend, Entity,
                error::VariableError,
            },
            ErrorKind,
            GenerateError,
        },
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    inkwell::{
        types::{BasicType, BasicTypeEnum},
        values::{BasicValue, BasicValueEnum, PointerValue},
    },
};

impl<'backend> super::Generator<'backend> {
    fn pointee(&self, analysis: &Analysis<'backend>) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => match self.get_entity(name) {
                Some(Entity::Variable { typing, .. }) => {
                    if let TypeKind::Pointer { target } = &typing.kind {
                        self.to_basic_type(target, analysis.span).ok()
                    } else {
                        None
                    }
                }
                _ => None,
            },
            AnalysisKind::Dereference(operand) => self.pointee(operand),
            _ => None,
        }
    }

    fn lvalue(
        &mut self,
        analysis: &Analysis<'backend>,
    ) -> Result<Option<(PointerValue<'backend>, BasicTypeEnum<'backend>)>, GenerateError<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => {
                if let Some(entity) = self.get_entity(name) {
                    match entity {
                        Entity::Variable { pointer, typing } => {
                            let kind = self.to_basic_type(typing, analysis.span)?;
                            Ok(Some((*pointer, kind)))
                        }
                        Entity::Function(func) => {
                            let ptr = func.as_global_value().as_pointer_value();
                            Ok(Some((ptr, ptr.get_type().into())))
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            AnalysisKind::Dereference(operand) => {
                let kind = self.pointee(operand);
                let value = self.analysis(*operand.clone())?;

                match (value, kind) {
                    (BasicValueEnum::PointerValue(_), None) => Err(GenerateError::new(
                        ErrorKind::Variable(VariableError::DereferenceNonPointer),
                        analysis.span,
                    )),
                    (BasicValueEnum::PointerValue(pointer), Some(resolved)) => {
                        Ok(Some((pointer, resolved)))
                    }
                    _ => Ok(None),
                }
            }
            AnalysisKind::Access(target, member) => {
                let field = if let AnalysisKind::Usage(name) = &member.kind {
                    name.clone()
                } else {
                    return Ok(None);
                };

                if let Some((base, kind)) = self.lvalue(target)? {
                    if kind.is_struct_type() {
                        let shape = kind.into_struct_type();

                        let found = self.find_entity(|entity| {
                            matches!(entity, Entity::Structure { shape: defined, .. } if defined.as_basic_type_enum() == shape.as_basic_type_enum())
                        });

                        if let Some(Entity::Structure { members: fields, .. }) = found {
                            if let Some(index) = fields.iter().position(|item| item == &field) {
                                let slot = self
                                    .builder
                                    .build_struct_gep(shape, base, index as u32, "pointer")
                                    .map_err(|error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    })?;

                                let resolved = shape.get_field_type_at_index(index as u32).unwrap();
                                return Ok(Some((slot, resolved)));
                            }
                        }
                    } else if kind.is_pointer_type() {
                        if let Some(resolved) = self.pointee(target) {
                            if resolved.is_struct_type() {
                                let shape = resolved.into_struct_type();
                                let load = self
                                    .builder
                                    .build_load(kind, base, "load")
                                    .map_err(|error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    })?;

                                if let Some(inst) = load.as_instruction_value() {
                                    inst.set_alignment(self.align(kind)).ok();
                                }

                                let loaded = load.into_pointer_value();
                                let found = self.find_entity(|entity| {
                                    matches!(entity, Entity::Structure { shape: defined, .. } if defined.as_basic_type_enum() == shape.as_basic_type_enum())
                                });

                                if let Some(Entity::Structure { members: fields, .. }) = found {
                                    if let Some(index) =
                                        fields.iter().position(|item| item == &field)
                                    {
                                        let slot = self
                                            .builder
                                            .build_struct_gep(
                                                shape,
                                                loaded,
                                                index as u32,
                                                "pointer",
                                            )
                                            .map_err(|error| {
                                                GenerateError::new(
                                                    ErrorKind::BuilderError(error.into()),
                                                    analysis.span,
                                                )
                                            })?;

                                        let resolved =
                                            shape.get_field_type_at_index(index as u32).unwrap();
                                        return Ok(Some((slot, resolved)));
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback for namespaced Globals and Methods (e.g. Color.Red or Point.print_position)
                if let AnalysisKind::Usage(target_name) = &target.kind {
                    let full_name = format!("{}.{}", target_name, field);
                    let module = self.current_module();

                    if let Some(global) = module.get_global(&full_name) {
                        let ptr = global.as_pointer_value();
                        let kind: BasicTypeEnum = match global.get_value_type() {
                            inkwell::types::AnyTypeEnum::ArrayType(t) => t.into(),
                            inkwell::types::AnyTypeEnum::StructType(t) => t.into(),
                            inkwell::types::AnyTypeEnum::FloatType(t) => t.into(),
                            inkwell::types::AnyTypeEnum::IntType(t) => t.into(),
                            inkwell::types::AnyTypeEnum::PointerType(t) => t.into(),
                            inkwell::types::AnyTypeEnum::VectorType(t) => t.into(),
                            _ => return Ok(None),
                        };
                        return Ok(Some((ptr, kind)));
                    }

                    if let Some(func) = module.get_function(&full_name) {
                        let ptr = func.as_global_value().as_pointer_value();
                        return Ok(Some((ptr, ptr.get_type().into())));
                    }
                }

                Ok(None)
            }
            AnalysisKind::Index(index) => {
                if let Some((base, kind)) = self.lvalue(&index.target)? {
                    if index.members.is_empty() {
                        return Ok(None);
                    }

                    let offset = self.analysis(index.members[0].clone())?;

                    if kind.is_struct_type() {
                        if let BasicValueEnum::IntValue(integer) = offset {
                            if let Some(constant) = integer.get_zero_extended_constant() {
                                let shape = kind.into_struct_type();
                                let slot = self
                                    .builder
                                    .build_struct_gep(shape, base, constant as u32, "index")
                                    .map_err(|error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    })?;

                                let resolved =
                                    shape.get_field_type_at_index(constant as u32).unwrap();
                                return Ok(Some((slot, resolved)));
                            }
                        }
                    } else if kind.is_array_type() {
                        if let BasicValueEnum::IntValue(integer) = offset {
                            let shape = kind.into_array_type();
                            let element = shape.get_element_type();
                            let zero = self.context.i32_type().const_zero();
                            let slot = unsafe {
                                self.builder
                                    .build_in_bounds_gep(
                                        shape,
                                        base,
                                        &[zero, integer],
                                        "index",
                                    )
                                    .map_err(|error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    })?
                            };

                            return Ok(Some((slot, element)));
                        }
                    } else if kind.is_pointer_type() {
                        if let BasicValueEnum::IntValue(integer) = offset {
                            if let Some(element) = self.pointee(&index.target) {
                                let load = self
                                    .builder
                                    .build_load(kind, base, "load")
                                    .map_err(|error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    })?;

                                if let Some(inst) = load.as_instruction_value() {
                                    inst.set_alignment(self.align(kind)).ok();
                                }

                                let loaded = load.into_pointer_value();
                                let slot = unsafe {
                                    self.builder
                                        .build_in_bounds_gep(element, loaded, &[integer], "index")
                                        .map_err(|error| {
                                            GenerateError::new(
                                                ErrorKind::BuilderError(error.into()),
                                                analysis.span,
                                            )
                                        })?
                                };

                                return Ok(Some((slot, element)));
                            }
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
        if let Some((pointer, _)) = self.lvalue(&operand)? {
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
        let pointee = self.pointee(&operand);
        let value = self.analysis(*operand.clone())?;

        match (value, pointee) {
            (BasicValueEnum::PointerValue(pointer), Some(kind)) => {
                let load = self
                    .builder
                    .build_load(kind, pointer, "deref")
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?;

                if let Some(inst) = load.as_instruction_value() {
                    inst.set_alignment(self.align(kind)).ok();
                }

                Ok(load)
            }
            _ => Err(GenerateError::new(
                ErrorKind::Variable(VariableError::DereferenceNonPointer),
                span,
            )),
        }
    }

    pub fn usage(
        &self,
        identifier: Str<'backend>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(entity) = self.get_entity(&identifier) {
            return match entity {
                Entity::Function(func) => {
                    Ok(BasicValueEnum::from(func.as_global_value().as_pointer_value()))
                }
                Entity::Variable { pointer, typing } => {
                    let kind = self.to_basic_type(typing, span)?;

                    if kind.is_array_type() || kind.is_struct_type() {
                        Ok(BasicValueEnum::from(*pointer))
                    } else {
                        let load = self
                            .builder
                            .build_load(kind, *pointer, &identifier)
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            })?;

                        if let Some(inst) = load.as_instruction_value() {
                            inst.set_alignment(self.align(kind)).ok();
                        }

                        Ok(load)
                    }
                }
                _ => Err(GenerateError::new(
                    ErrorKind::Variable(VariableError::NotAValue {
                        name: identifier.to_string(),
                    }),
                    span,
                )),
            };
        }

        let module = self.current_module();

        if let Some(global) = module.get_global(&identifier) {
            let kind: BasicTypeEnum = match global.get_value_type() {
                inkwell::types::AnyTypeEnum::ArrayType(_)
                | inkwell::types::AnyTypeEnum::StructType(_) => {
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

            let load = self
                .builder
                .build_load(kind, global.as_pointer_value(), &identifier)
                .map_err(|error| {
                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                })?;

            if let Some(inst) = load.as_instruction_value() {
                inst.set_alignment(self.align(kind)).ok();
            }

            return Ok(load);
        }

        if let Some(func) = module.get_function(&identifier) {
            return Ok(BasicValueEnum::from(func.as_global_value().as_pointer_value()));
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
        let result = self.analysis(*value)?;

        let existing = match self.get_entity(&target) {
            Some(Entity::Variable { pointer, typing }) => Some((*pointer, typing.clone())),
            _ => None,
        };

        if let Some((slot, typing)) = existing {
            let declared = self.to_basic_type(&typing, span)?;

            let store = self
                .builder
                .build_store(slot, result)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            store.set_alignment(self.align(declared)).ok();

            let entity = Entity::Variable {
                pointer: slot,
                typing,
            };

            if !self.update_entity(&target, entity.clone()) {
                self.insert_entity(target.clone(), entity);
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
        bind: Binding<Str<'backend>, Box<Analysis<'backend>>, Type<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let expression = bind.value.ok_or_else(|| {
            GenerateError::new(
                ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                    name: bind.target.to_string(),
                }),
                span,
            )
        })?;

        let typing = bind.annotation.clone();
        let global = self.builder.get_insert_block().is_none();

        let scope = if global {
            let void = self.context.void_type();
            let signature = void.fn_type(&[], false);
            let func = self.current_module().add_function("init", signature, None);
            let block = self.context.append_basic_block(func, "entry");

            self.builder.position_at_end(block);
            Some(func)
        } else {
            None
        };

        let result = self.analysis(*expression)?;

        if let Some(func) = scope {
            self.builder.clear_insertion_position();
            unsafe {
                func.delete();
            }
        }

        let declared = self.to_basic_type(&typing, span)?;

        let pointer = if global {
            let variable = self.current_module().add_global(declared, None, &bind.target);
            variable.set_initializer(&result);
            variable.set_alignment(self.align(declared));
            variable.as_pointer_value()
        } else {
            let allocate = self
                .builder
                .build_alloca(declared, &bind.target)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            if let Some(inst) = allocate.as_instruction_value() {
                inst.set_alignment(self.align(declared)).ok();
            }

            let store = self
                .builder
                .build_store(allocate, result)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            store.set_alignment(self.align(declared)).ok();

            allocate
        };

        self.insert_entity(
            bind.target.clone(),
            Entity::Variable {
                pointer,
                typing,
            },
        );

        Ok(result)
    }

    pub fn store(
        &mut self,
        target: Box<Analysis<'backend>>,
        value: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let result = self.analysis(*value.clone())?;

        if let Some((pointer, kind)) = self.lvalue(&target)? {

            if result.get_type() != kind {
                return Err(GenerateError::new(
                    ErrorKind::Variable(VariableError::AssignmentTypeMismatch),
                    span,
                ));
            }

            let store = self
                .builder
                .build_store(pointer, result)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            store.set_alignment(self.align(kind)).ok();
        } else {
            return Err(GenerateError::new(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                span,
            ));
        }

        Ok(result)
    }
}
