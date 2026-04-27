use {
    crate::{
        analyzer::{Analysis, AnalysisKind, Target},
        data::Str,
        data::*,
        generator::{
            inkwell::{error::VariableError, Entity},
            ErrorKind, GenerateError, Generator,
        },
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    inkwell::{
        types::BasicTypeEnum,
        values::{BasicValue, BasicValueEnum, PointerValue},
    },
};

impl<'backend> Generator<'backend> {
    fn target_name(target: &Target<'backend>) -> Str<'backend> {
        target.name
    }

    fn alias(&mut self, target: Str<'backend>, value: &Analysis<'backend>) -> bool {
        if let Some(entity) = self.entity(value) {
            self.insert_entity(target, entity);
            true
        } else {
            false
        }
    }

    fn pointee(&self, analysis: &Analysis<'backend>) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(identifier) => match self.get_entity(identifier) {
                Some(Entity::Variable { typing, .. }) => {
                    if let TypeKind::Pointer { target } = &typing.kind {
                        self.to_basic_type(target, analysis.span).ok()
                    } else {
                        None
                    }
                }
                _ => None,
            },
            AnalysisKind::Symbol(target) => match self.get_entity(&target.name) {
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
            AnalysisKind::Slot(target, _) => self.pointee(target),
            _ => None,
        }
    }

    pub(crate) fn lvalue(
        &mut self,
        analysis: &Analysis<'backend>,
    ) -> Result<Option<(PointerValue<'backend>, BasicTypeEnum<'backend>)>, GenerateError<'backend>>
    {
        match &analysis.kind {
            AnalysisKind::Usage(identifier) => {
                if let Some(entity) = self.get_entity(identifier) {
                    match entity {
                        Entity::Variable { pointer, typing } => {
                            let kind = self.to_basic_type(typing, analysis.span)?;
                            Ok(Some((*pointer, kind)))
                        }
                        Entity::Function(function) => {
                            let pointer = function.as_global_value().as_pointer_value();
                            Ok(Some((pointer, pointer.get_type().into())))
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            AnalysisKind::Symbol(target) => {
                if let Some(entity) = self.get_entity(&target.name) {
                    match entity {
                        Entity::Variable { pointer, typing } => {
                            let kind = self.to_basic_type(typing, analysis.span)?;
                            Ok(Some((*pointer, kind)))
                        }
                        Entity::Function(function) => {
                            let pointer = function.as_global_value().as_pointer_value();
                            Ok(Some((pointer, pointer.get_type().into())))
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
            AnalysisKind::Slot(target, index) => {
                if let Some((base, kind)) = self.lvalue(target)? {
                    let typing = self.value_type(&target.typing);

                    if let TypeKind::Union(aggregate) = &typing.kind {
                        if let Some(member) = aggregate.members.get(*index) {
                            let resolved = self.to_basic_type(member, analysis.span)?;
                            let space = base.get_type().get_address_space();
                            let destination = self.context.ptr_type(space);
                            let slot = self
                                .builder
                                .build_pointer_cast(base, destination, "pointer")
                                .map_err(|error| {
                                    GenerateError::new(
                                        ErrorKind::BuilderError(error.into()),
                                        analysis.span,
                                    )
                                })?;

                            return Ok(Some((slot, resolved)));
                        }
                    }

                    if kind.is_struct_type() {
                        let shape = kind.into_struct_type();
                        let slot = self
                            .builder
                            .build_struct_gep(shape, base, *index as u32, "pointer")
                            .map_err(|error| {
                                GenerateError::new(
                                    ErrorKind::BuilderError(error.into()),
                                    analysis.span,
                                )
                            })?;

                        let resolved = shape.get_field_type_at_index(*index as u32).unwrap();
                        return Ok(Some((slot, resolved)));
                    } else if kind.is_pointer_type() {
                        if let Some(resolved) = self.pointee(target) {
                            if resolved.is_struct_type() {
                                let shape = resolved.into_struct_type();
                                let load = self.builder.build_load(kind, base, "load").map_err(
                                    |error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    },
                                )?;

                                if let Some(instruction) = load.as_instruction_value() {
                                    instruction.set_alignment(self.align(kind)).ok();
                                }

                                let loaded = load.into_pointer_value();
                                let slot = self
                                    .builder
                                    .build_struct_gep(shape, loaded, *index as u32, "pointer")
                                    .map_err(|error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    })?;

                                let resolved = shape.get_field_type_at_index(*index as u32).unwrap();
                                return Ok(Some((slot, resolved)));
                            }
                        }
                    }
                }

                Ok(None)
            }
            AnalysisKind::Access(target, member) => {
                let field = if let AnalysisKind::Usage(identifier) = &member.kind {
                    identifier.clone()
                } else {
                    return Ok(None);
                };
                let typing = self.value_type(&target.typing);

                if let Some((base, kind)) = self.lvalue(target)? {
                    if kind.is_struct_type() {
                        let shape = kind.into_struct_type();
                        if let Some(index) = self.field(&typing, &field) {
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
                    } else if kind.is_pointer_type() {
                        if let Some(resolved) = self.pointee(target) {
                            if resolved.is_struct_type() {
                                let shape = resolved.into_struct_type();
                                let load = self.builder.build_load(kind, base, "load").map_err(
                                    |error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    },
                                )?;

                                if let Some(instruction) = load.as_instruction_value() {
                                    instruction.set_alignment(self.align(kind)).ok();
                                }

                                let loaded = load.into_pointer_value();

                                if let Some(index) = self.field(&typing, &field) {
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

                if let AnalysisKind::Usage(identifier) = &target.kind {
                    let path = format!("{}.{}", identifier, field);
                    let module = self.current_module();

                    if let Some(global) = module.get_global(&path) {
                        let pointer = global.as_pointer_value();
                        let kind: BasicTypeEnum = match global.get_value_type() {
                            inkwell::types::AnyTypeEnum::ArrayType(defined) => defined.into(),
                            inkwell::types::AnyTypeEnum::StructType(defined) => defined.into(),
                            inkwell::types::AnyTypeEnum::FloatType(defined) => defined.into(),
                            inkwell::types::AnyTypeEnum::IntType(defined) => defined.into(),
                            inkwell::types::AnyTypeEnum::PointerType(defined) => defined.into(),
                            inkwell::types::AnyTypeEnum::VectorType(defined) => defined.into(),
                            _ => return Ok(None),
                        };
                        return Ok(Some((pointer, kind)));
                    }

                    if let Some(function) = module.get_function(&path) {
                        let pointer = function.as_global_value().as_pointer_value();
                        return Ok(Some((pointer, pointer.get_type().into())));
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
                                    .build_in_bounds_gep(shape, base, &[zero, integer], "index")
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
                                let load = self.builder.build_load(kind, base, "load").map_err(
                                    |error| {
                                        GenerateError::new(
                                            ErrorKind::BuilderError(error.into()),
                                            analysis.span,
                                        )
                                    },
                                )?;

                                if let Some(instruction) = load.as_instruction_value() {
                                    instruction.set_alignment(self.align(kind)).ok();
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
        span: Span,
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
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let pointee = self.pointee(&operand);
        let value = self.analysis(*operand.clone())?;

        match (value, pointee) {
            (BasicValueEnum::PointerValue(pointer), Some(kind)) => {
                let load = self
                    .builder
                    .build_load(kind, pointer, "dereference")
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?;

                if let Some(instruction) = load.as_instruction_value() {
                    instruction.set_alignment(self.align(kind)).ok();
                }

                Ok(load)
            }
            _ => Err(GenerateError::new(
                ErrorKind::Variable(VariableError::DereferenceNonPointer),
                span,
            )),
        }
    }

    pub fn symbol_value(
        &self,
        target: Target<'backend>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        self.usage(Self::target_name(&target), span)
    }

    pub fn usage(
        &self,
        identifier: Str<'backend>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let Some(entity) = self.get_entity(&identifier) {
            return match entity {
                Entity::Function(function) => Ok(BasicValueEnum::from(
                    self.linked(identifier, *function).as_global_value().as_pointer_value(),
                )),
                Entity::Variable { pointer, typing } => {
                    let kind = self.to_basic_type(typing, span)?;

                    if kind.is_array_type() {
                        Ok(BasicValueEnum::from(*pointer))
                    } else {
                        let load = self
                            .builder
                            .build_load(kind, *pointer, &identifier)
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            })?;

                        if let Some(instruction) = load.as_instruction_value() {
                            instruction.set_alignment(self.align(kind)).ok();
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
                inkwell::types::AnyTypeEnum::ArrayType(_) => {
                    return Ok(BasicValueEnum::from(global.as_pointer_value()));
                }
                inkwell::types::AnyTypeEnum::StructType(defined) => defined.into(),
                inkwell::types::AnyTypeEnum::FloatType(defined) => defined.into(),
                inkwell::types::AnyTypeEnum::IntType(defined) => defined.into(),
                inkwell::types::AnyTypeEnum::PointerType(defined) => defined.into(),
                inkwell::types::AnyTypeEnum::VectorType(defined) => defined.into(),
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
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

            if let Some(instruction) = load.as_instruction_value() {
                instruction.set_alignment(self.align(kind)).ok();
            }

            return Ok(load);
        }

        if let Some(function) = module.get_function(&identifier) {
            return Ok(BasicValueEnum::from(
                function.as_global_value().as_pointer_value(),
            ));
        }

        Err(GenerateError::new(
            ErrorKind::Variable(VariableError::Undefined {
                name: identifier.to_string(),
            }),
            span,
        ))
    }

    pub fn write(
        &mut self,
        target: Target<'backend>,
        value: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        self.assign(Self::target_name(&target), value, span)
    }

    pub fn assign(
        &mut self,
        target: Str<'backend>,
        value: Box<Analysis<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let result = self.analysis(*value)?;

        let existing = match self.get_entity(&target) {
            Some(Entity::Variable { pointer, typing }) => Some((*pointer, typing.clone())),
            _ => None,
        };

        if let Some((slot, typing)) = existing {
            let declared = result.get_type();

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
        binding: Binding<Box<Analysis<'backend>>, Box<Analysis<'backend>>, Type<'backend>>,
        span: Span,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        match binding.target.kind {
            AnalysisKind::Usage(target) => match binding.kind {
                BindingKind::Static => {
                    let expression = binding.value.ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                                name: target.to_string(),
                            }),
                            span,
                        )
                    })?;

                    if self.alias(target.clone(), &expression) {
                        return Ok(self.context.i64_type().const_zero().into());
                    }

                    let result = self.analysis(*expression)?;

                    let declared = result.get_type();

                    let variable = self.current_module().add_global(declared, None, &target);
                    variable.set_initializer(&result);
                    variable.set_alignment(self.align(declared));

                    let pointer = variable.as_pointer_value();

                    let typing = binding.annotation.clone();

                    self.insert_entity(target.clone(), Entity::Variable { pointer, typing });

                    Ok(result)
                }

                _ => {
                    let expression = binding.value.ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                                name: target.to_string(),
                            }),
                            span,
                        )
                    })?;

                    if self.alias(target.clone(), &expression) {
                        return Ok(self.context.i64_type().const_zero().into());
                    }

                    let typing = binding.annotation.clone();
                    let global = self.builder.get_insert_block().is_none();

                    let scope = if global {
                        let void = self.context.void_type();
                        let signature = void.fn_type(&[], false);
                        let function = self.current_module().add_function("init", signature, None);
                        let block = self.context.append_basic_block(function, "entry");

                        self.builder.position_at_end(block);
                        Some(function)
                    } else {
                        None
                    };

                    let result = self.analysis(*expression)?;

                    if let Some(function) = scope {
                        self.builder.clear_insertion_position();
                        unsafe {
                            function.delete();
                        }
                    }

                    let declared = result.get_type();

                    let pointer = if global {
                        let variable = self.current_module().add_global(declared, None, &target);
                        variable.set_initializer(&result);
                        variable.set_alignment(self.align(declared));
                        variable.as_pointer_value()
                    } else {
                        let allocate =
                            self.builder
                                .build_alloca(declared, &target)
                                .map_err(|error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                })?;

                        if let Some(instruction) = allocate.as_instruction_value() {
                            instruction.set_alignment(self.align(declared)).ok();
                        }

                        let store =
                            self.builder
                                .build_store(allocate, result)
                                .map_err(|error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                })?;
                        store.set_alignment(self.align(declared)).ok();

                        allocate
                    };

                    self.insert_entity(target.clone(), Entity::Variable { pointer, typing });

                    Ok(result)
                }
            },

            AnalysisKind::Symbol(target) => match binding.kind {
                BindingKind::Static => {
                    let target = target.name;
                    let expression = binding.value.ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                                name: target.to_string(),
                            }),
                            span,
                        )
                    })?;

                    if self.alias(target.clone(), &expression) {
                        return Ok(self.context.i64_type().const_zero().into());
                    }

                    let result = self.analysis(*expression)?;
                    let declared = result.get_type();
                    let variable = self.current_module().add_global(declared, None, &target);
                    variable.set_initializer(&result);
                    variable.set_alignment(self.align(declared));
                    let pointer = variable.as_pointer_value();
                    let typing = binding.annotation.clone();
                    self.insert_entity(target.clone(), Entity::Variable { pointer, typing });
                    Ok(result)
                }
                _ => {
                    let target = target.name;
                    let expression = binding.value.ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                                name: target.to_string(),
                            }),
                            span,
                        )
                    })?;

                    if self.alias(target.clone(), &expression) {
                        return Ok(self.context.i64_type().const_zero().into());
                    }

                    let typing = binding.annotation.clone();
                    let global = self.builder.get_insert_block().is_none();

                    let scope = if global {
                        let void = self.context.void_type();
                        let signature = void.fn_type(&[], false);
                        let function = self.current_module().add_function("init", signature, None);
                        let block = self.context.append_basic_block(function, "entry");

                        self.builder.position_at_end(block);
                        Some(function)
                    } else {
                        None
                    };

                    let result = self.analysis(*expression)?;

                    if let Some(function) = scope {
                        self.builder.clear_insertion_position();
                        unsafe {
                            function.delete();
                        }
                    }

                    let declared = result.get_type();

                    let pointer = if global {
                        let variable = self.current_module().add_global(declared, None, &target);
                        variable.set_initializer(&result);
                        variable.set_alignment(self.align(declared));
                        variable.as_pointer_value()
                    } else {
                        let allocate = self.builder.build_alloca(declared, &target).map_err(|error| {
                            GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                        })?;

                        if let Some(instruction) = allocate.as_instruction_value() {
                            instruction.set_alignment(self.align(declared)).ok();
                        }

                        let store = self.builder.build_store(allocate, result).map_err(|error| {
                            GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                        })?;
                        store.set_alignment(self.align(declared)).ok();

                        allocate
                    };

                    self.insert_entity(target.clone(), Entity::Variable { pointer, typing });

                    Ok(result)
                }
            },

            _ => {
                unimplemented!("destruction isn't implemented yet!");
            }
        }
    }

    pub fn store(
        &mut self,
        target: Box<Analysis<'backend>>,
        value: Box<Analysis<'backend>>,
        span: Span,
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
