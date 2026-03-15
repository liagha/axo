use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::{Index, Str, Structure},
        generator::{
            inkwell::{Backend, Entity, GenerateError, Generator},
            BuilderError, DataStructureError, ErrorKind,
        },
        resolver::TypeKind,
        tracker::Span,
    },
    inkwell::{
        types::{BasicType, BasicTypeEnum},
        values::{BasicValueEnum, IntValue},
        IntPredicate,
    },
};

impl<'backend> Generator<'backend> {
    fn fields(&self, target: BasicTypeEnum<'backend>) -> Option<Vec<Str<'backend>>> {
        if let Some(Entity::Struct { fields, .. }) = self.find_entity(|entity| {
            matches!(entity, Entity::Struct { structure, .. } if structure.as_basic_type_enum() == target)
        }) {
            Some(fields.clone())
        } else {
            None
        }
    }

    fn union_fields(
        &self,
        target: BasicTypeEnum<'backend>,
    ) -> Option<Vec<(Str<'backend>, BasicTypeEnum<'backend>)>> {
        if let Some(Entity::Union { fields, .. }) = self.find_entity(|entity| {
            matches!(entity, Entity::Union { structure, .. } if structure.as_basic_type_enum() == target)
        }) {
            Some(fields.clone())
        } else {
            None
        }
    }

    fn size(&self, typing: BasicTypeEnum<'backend>) -> u64 {
        typing
            .size_of()
            .and_then(|value| value.get_zero_extended_constant())
            .unwrap_or(0)
    }

    pub fn trap(
        &self,
        condition: Option<IntValue<'backend>>,
        span: Span<'backend>,
    ) -> Result<(), GenerateError<'backend>> {
        let module = self.current_module();

        let function = module.get_function("llvm.trap").unwrap_or_else(|| {
            let shape = self.context.void_type().fn_type(&[], false);
            module.add_function("llvm.trap", shape, None)
        });

        match condition {
            None => {
                self.builder
                    .build_call(function, &[], "trap")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder
                    .build_unreachable()
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
            }
            Some(value) => {
                let block = self
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                let parent = block
                    .get_parent()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                let failure = self.context.append_basic_block(parent, "failure");
                let success = self.context.append_basic_block(parent, "success");

                self.builder
                    .build_conditional_branch(value, failure, success)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder.position_at_end(failure);

                self.builder
                    .build_call(function, &[], "")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder
                    .build_unreachable()
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder.position_at_end(success);
            }
        }

        Ok(())
    }

    fn convert(
        &self,
        value: BasicValueEnum<'backend>,
        target: BasicTypeEnum<'backend>,
    ) -> Option<BasicValueEnum<'backend>> {
        if value.get_type() == target {
            return Some(value);
        }

        match (value, target) {
            (BasicValueEnum::IntValue(left), BasicTypeEnum::IntType(right)) => self
                .builder
                .build_int_cast(left, right, "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(left), BasicTypeEnum::FloatType(right)) => self
                .builder
                .build_float_cast(left, right, "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::IntValue(left), BasicTypeEnum::FloatType(right)) => self
                .builder
                .build_signed_int_to_float(left, right, "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(left), BasicTypeEnum::IntType(right)) => self
                .builder
                .build_float_to_signed_int(left, right, "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::PointerValue(left), BasicTypeEnum::PointerType(right)) => self
                .builder
                .build_pointer_cast(left, right, "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::PointerValue(left), BasicTypeEnum::IntType(right)) => self
                .builder
                .build_ptr_to_int(left, right, "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::IntValue(left), BasicTypeEnum::PointerType(right)) => self
                .builder
                .build_int_to_ptr(left, right, "cast")
                .ok()
                .map(Into::into),
            _ => None,
        }
    }

    pub fn structure(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();
        let name = identifier.as_str().unwrap_or("structure");

        let shape = self.context.get_struct_type(name).unwrap_or_else(|| {
            self.context.opaque_struct_type(name)
        });

        let mut types = Vec::with_capacity(structure.members.len());
        let mut fields = Vec::with_capacity(structure.members.len());

        for member in &structure.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();
                fields.push(field.clone());
                types.push(self.to_basic_type(&binding.annotation, member.span)?);
            }
        }

        if shape.is_opaque() {
            shape.set_body(&types, false);
        }

        self.insert_entity(identifier, Entity::Struct { structure: shape, fields });

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn union(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();
        let name = identifier.as_str().unwrap_or("union");

        let shape = self.context.get_struct_type(name).unwrap_or_else(|| {
            self.context.opaque_struct_type(name)
        });

        let mut fields = Vec::with_capacity(structure.members.len());
        let mut largest: Option<BasicTypeEnum> = None;
        let mut maximum = 0;

        for member in &structure.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();
                let typing = self.to_basic_type(&binding.annotation, member.span)?;

                fields.push((field.clone(), typing));

                let limit = self.size(typing);

                if limit >= maximum || largest.is_none() {
                    maximum = limit;
                    largest = Some(typing);
                }
            }
        }

        if shape.is_opaque() {
            if let Some(target) = largest {
                shape.set_body(&[target], false);
            } else {
                shape.set_body(&[], false);
            }
        }

        self.insert_entity(identifier, Entity::Union { structure: shape, fields });

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn constructor(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();
        let target = identifier.as_str().unwrap_or("").to_string();

        let entity = self.get_entity(&identifier).cloned();

        match entity {
            Some(Entity::Struct { structure: shape, fields }) => {
                let mut current = shape.get_undef();
                let mut position = 0usize;

                for member in structure.members {
                    let (index, name, assigned) = match &member.kind {
                        AnalysisKind::Assign(field, assigned) => {
                            let idx = fields
                                .iter()
                                .position(|item| item == field)
                                .ok_or_else(|| {
                                    GenerateError::new(
                                        ErrorKind::DataStructure(DataStructureError::UnknownField {
                                            struct_name: target.clone(),
                                            field_name: field.as_str().unwrap_or("").to_string(),
                                        }),
                                        span,
                                    )
                                })?;

                            position = idx + 1;

                            (
                                idx,
                                field.as_str().unwrap_or("").to_string(),
                                *assigned.clone(),
                            )
                        }
                        _ => {
                            if position >= fields.len() {
                                return Err(GenerateError::new(
                                    ErrorKind::DataStructure(DataStructureError::TooManyInitializers {
                                        struct_name: target,
                                    }),
                                    span,
                                ));
                            }

                            let idx = position;
                            position += 1;

                            (idx, format!("position {}", idx), member)
                        }
                    };

                    let kind = shape.get_field_type_at_index(index as u32).unwrap();
                    let value = self.analysis(assigned)?;

                    let cast = self.convert(value, kind).ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::DataStructure(DataStructureError::ConstructorFieldTypeMismatch {
                                struct_name: target.clone(),
                                field_name: name,
                            }),
                            span,
                        )
                    })?;

                    current = self
                        .builder
                        .build_insert_value(current, cast, index as u32, "insert")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                        .into_struct_value();
                }

                Ok(current.into())
            }

            Some(Entity::Union { structure: shape, fields }) => {
                if structure.members.len() > 1 {
                    return Err(GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::TooManyInitializers {
                            struct_name: target,
                        }),
                        span,
                    ));
                }

                let block = self
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                let parent = block
                    .get_parent()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                let pointer = self.build_entry(parent, shape.into(), Str::from("init"));

                if let Some(member) = structure.members.into_iter().next() {
                    let (name, assigned) = match &member.kind {
                        AnalysisKind::Assign(field, assigned) => {
                            (field.as_str().unwrap_or("").to_string(), *assigned.clone())
                        }
                        _ => {
                            return Err(GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression),
                                span,
                            ))
                        }
                    };

                    let typing = fields
                        .iter()
                        .find(|(item, _)| item.as_str().unwrap_or("") == name)
                        .map(|(_, typing)| *typing)
                        .ok_or_else(|| {
                            GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::UnknownField {
                                    struct_name: target.clone(),
                                    field_name: name.clone(),
                                }),
                                span,
                            )
                        })?;

                    let value = self.analysis(assigned)?;

                    let cast = self.convert(value, typing).ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::DataStructure(DataStructureError::ConstructorFieldTypeMismatch {
                                struct_name: target.clone(),
                                field_name: name,
                            }),
                            span,
                        )
                    })?;

                    let space = pointer.get_type().get_address_space();
                    let destination = self.context.ptr_type(space);

                    let slot = self
                        .builder
                        .build_pointer_cast(pointer, destination, "cast")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                    self.builder
                        .build_store(slot, cast)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
                }

                self.builder
                    .build_load(shape, pointer, "value")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
            }

            _ => Err(GenerateError::new(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    struct_name: target,
                    field_name: String::from("unknown"),
                }),
                span,
            )),
        }
    }

    pub fn access(
        &mut self,
        target: Box<Analysis<'backend>>,
        member: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let AnalysisKind::Usage(identifier) = &target.kind {
            if self.has_module(identifier) {
                return match &member.kind {
                    AnalysisKind::Usage(name) => self.usage(name.clone(), span),
                    AnalysisKind::Invoke(invoke) => self.invoke(invoke.clone(), span),
                    AnalysisKind::Constructor(constructor) => self.constructor(constructor.clone(), span),
                    _ => Err(GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::InvalidModuleAccess),
                        span,
                    )),
                };
            }
        }

        let field = match &member.kind {
            AnalysisKind::Usage(identifier) => identifier.clone(),
            _ => {
                return Err(GenerateError::new(
                    ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression),
                    span,
                ))
            }
        };

        if let AnalysisKind::Usage(identifier) = &target.kind {
            if let Some(Entity::Variable { pointer, typing }) = self.get_entity(identifier) {
                let kind = self.to_basic_type(typing, span)?;

                if kind.is_struct_type() {
                    let shape = kind.into_struct_type();

                    if let Some(fields) = self.fields(kind) {
                        if let Some(index) = fields.iter().position(|item| item == &field) {
                            let slot = self
                                .builder
                                .build_struct_gep(shape, *pointer, index as u32, "slot")
                                .map_err(|error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                })?;

                            let resolved = shape.get_field_type_at_index(index as u32).unwrap();

                            return self
                                .builder
                                .build_load(resolved, slot, "value")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                        }
                    } else if let Some(fields) = self.union_fields(kind) {
                        if let Some((_, resolved)) = fields.iter().find(|(name, _)| name == &field) {
                            let space = pointer.get_type().get_address_space();
                            let destination = self.context.ptr_type(space);

                            let slot = self
                                .builder
                                .build_pointer_cast(*pointer, destination, "cast")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            return self
                                .builder
                                .build_load(*resolved, slot, "value")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                        }
                    }
                }
            }
        }

        let value = self.analysis(*target)?;

        if let BasicValueEnum::StructValue(structure) = value {
            if let Some(fields) = self.fields(structure.get_type().as_basic_type_enum()) {
                if let Some(index) = fields.iter().position(|item| item == &field) {
                    return self
                        .builder
                        .build_extract_value(structure, index as u32, "extract")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                        .map(Into::into);
                }
            } else if let Some(fields) = self.union_fields(structure.get_type().as_basic_type_enum()) {
                if let Some((_, resolved)) = fields.iter().find(|(name, _)| name == &field) {
                    let block = self
                        .builder
                        .get_insert_block()
                        .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                    let parent = block
                        .get_parent()
                        .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                    let pointer = self.build_entry(parent, structure.get_type().into(), Str::from("spill"));

                    self.builder
                        .build_store(pointer, structure)
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                    let space = pointer.get_type().get_address_space();
                    let destination = self.context.ptr_type(space);

                    let slot = self
                        .builder
                        .build_pointer_cast(pointer, destination, "cast")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                    return self
                        .builder
                        .build_load(*resolved, slot, "value")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                }
            }
        }

        Err(GenerateError::new(
            ErrorKind::DataStructure(DataStructureError::AccessOnNonStructType {
                field_name: field.to_string(),
            }),
            span,
        ))
    }

    pub fn array(
        &mut self,
        elements: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if elements.is_empty() {
            return Err(GenerateError::new(
                ErrorKind::DataStructure(DataStructureError::EmptyArray),
                span,
            ));
        }

        let mut values = Vec::with_capacity(elements.len());

        for element in elements {
            values.push(self.analysis(element)?);
        }

        let kind = values[0].get_type();
        let limit = values.len() as u32;
        let shape = kind.array_type(limit);

        let block = self
            .builder
            .get_insert_block()
            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

        let parent = block
            .get_parent()
            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

        let pointer = self.build_entry(parent, shape.into(), Str::from("array"));

        for (index, value) in values.into_iter().enumerate() {
            let cast = self.convert(value, kind).ok_or_else(|| {
                GenerateError::new(
                    ErrorKind::DataStructure(DataStructureError::ArrayLiteralTypeMismatch { index }),
                    span,
                )
            })?;

            let offset = self.context.i32_type().const_int(index as u64, false);
            let zero = self.context.i32_type().const_zero();

            let slot = unsafe {
                self.builder
                    .build_in_bounds_gep(shape, pointer, &[zero, offset], "slot")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
            };

            self.builder
                .build_store(slot, cast)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;
        }

        self.builder
            .build_load(shape, pointer, "value")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
    }

    pub fn tuple(
        &mut self,
        elements: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut values = Vec::with_capacity(elements.len());

        for element in elements {
            values.push(self.analysis(element)?);
        }

        let types: Vec<BasicTypeEnum> = values.iter().map(|item| item.get_type()).collect();
        let shape = self.context.struct_type(&types, false);
        let mut current = shape.get_undef();

        for (index, value) in values.into_iter().enumerate() {
            current = self
                .builder
                .build_insert_value(current, value, index as u32, "insert")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                .into_struct_value();
        }

        Ok(current.into())
    }

    pub fn index(
        &mut self,
        index: Index<Box<Analysis<'backend>>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if index.members.is_empty() {
            return Err(GenerateError::new(
                ErrorKind::DataStructure(DataStructureError::IndexMissingArgument),
                span,
            ));
        }

        let base = index.target.clone();
        let target = self.analysis(*base.clone())?;
        let offset = self.analysis(index.members[0].clone())?;

        if let AnalysisKind::Usage(identifier) = &index.target.kind {
            if let Some(Entity::Variable { typing, pointer }) = self.get_entity(identifier) {
                let kind = self.to_basic_type(typing, span)?;

                if kind.is_struct_type() {
                    if let BasicValueEnum::IntValue(integer) = offset {
                        let constant = integer.get_zero_extended_constant().ok_or_else(|| {
                            GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::TupleIndexNotConstant),
                                span,
                            )
                        })?;

                        let shape = kind.into_struct_type();
                        let slot = self
                            .builder
                            .build_struct_gep(shape, *pointer, constant as u32, "index")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        let field = shape.get_field_type_at_index(constant as u32).unwrap();

                        return self
                            .builder
                            .build_load(field, slot, "value")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                    }
                } else if kind.is_array_type() {
                    if let BasicValueEnum::IntValue(integer) = offset {
                        let shape = kind.into_array_type();
                        let limit = integer.get_type().const_int(shape.len() as u64, false);

                        let exceeds = self
                            .builder
                            .build_int_compare(IntPredicate::UGE, integer, limit, "check")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        let block = self
                            .builder
                            .get_insert_block()
                            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                        let parent = block
                            .get_parent()
                            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                        let failure = self.context.append_basic_block(parent, "failure");
                        let success = self.context.append_basic_block(parent, "success");

                        self.builder
                            .build_conditional_branch(exceeds, failure, success)
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        self.builder.position_at_end(failure);
                        self.trap(None, span)?;

                        self.builder.position_at_end(success);

                        let zero = integer.get_type().const_zero();
                        let slot = unsafe {
                            self.builder
                                .build_in_bounds_gep(shape, *pointer, &[zero, integer], "index")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                        };

                        return self
                            .builder
                            .build_load(shape.get_element_type(), slot, "value")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                    }
                }
            }
        }

        match (target, offset) {
            (BasicValueEnum::StructValue(structure), BasicValueEnum::IntValue(integer)) => {
                let constant = integer.get_zero_extended_constant().ok_or_else(|| {
                    GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::TupleIndexNotConstant),
                        span,
                    )
                })?;

                return self
                    .builder
                    .build_extract_value(structure, constant as u32, "extract")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                    .map(Into::into);
            }
            (BasicValueEnum::ArrayValue(array), BasicValueEnum::IntValue(integer)) => {
                if let Some(constant) = integer.get_zero_extended_constant() {
                    return self
                        .builder
                        .build_extract_value(array, constant as u32, "extract")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                        .map(Into::into);
                }

                let shape = array.get_type();
                let block = self
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                let parent = block
                    .get_parent()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                let pointer = self.build_entry(parent, shape.into(), Str::from("spill"));

                self.builder
                    .build_store(pointer, array)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                let limit = integer.get_type().const_int(shape.len() as u64, false);

                let exceeds = self
                    .builder
                    .build_int_compare(IntPredicate::UGE, integer, limit, "check")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                let failure = self.context.append_basic_block(parent, "failure");
                let success = self.context.append_basic_block(parent, "success");

                self.builder
                    .build_conditional_branch(exceeds, failure, success)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder.position_at_end(failure);
                self.trap(None, span)?;

                self.builder.position_at_end(success);

                let zero = integer.get_type().const_zero();
                let slot = unsafe {
                    self.builder
                        .build_in_bounds_gep(shape, pointer, &[zero, integer], "index")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                };

                return self
                    .builder
                    .build_load(shape.get_element_type(), slot, "value")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
            }
            (BasicValueEnum::PointerValue(pointer), BasicValueEnum::IntValue(integer)) => {
                let pointee = if let TypeKind::Pointer { target } = &base.typing.kind {
                    self.to_basic_type(target, base.span)?
                } else {
                    unreachable!()
                };

                let slot = unsafe {
                    self.builder
                        .build_in_bounds_gep(pointee, pointer, &[integer], "index")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                };

                return self
                    .builder
                    .build_load(pointee, slot, "value")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
            }
            _ => {}
        }

        Err(GenerateError::new(
            ErrorKind::DataStructure(DataStructureError::NotIndexable),
            span,
        ))
    }
}
