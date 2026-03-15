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
    pub fn size(&self, typing: BasicTypeEnum<'backend>) -> u64 {
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

                let fail = self.context.append_basic_block(parent, "fail");
                let pass = self.context.append_basic_block(parent, "pass");

                self.builder
                    .build_conditional_branch(value, fail, pass)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder.position_at_end(fail);

                self.builder
                    .build_call(function, &[], "")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder
                    .build_unreachable()
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder.position_at_end(pass);
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
            Some(value)
        } else {
            None
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
        let mut members = Vec::with_capacity(structure.members.len());

        for member in structure.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();
                members.push(field.clone());
                types.push(self.to_basic_type(&binding.annotation, member.span)?);
            } else {
                self.analysis(member)?;
            }
        }

        if shape.is_opaque() {
            shape.set_body(&types, false);
        }

        self.insert_entity(identifier, Entity::Structure { shape, members });

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn union(
        &mut self,
        union: Structure<Str<'backend>, Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = union.target.clone();
        let name = identifier.as_str().unwrap_or("union");

        let shape = self.context.get_struct_type(name).unwrap_or_else(|| {
            self.context.opaque_struct_type(name)
        });

        let mut members = Vec::with_capacity(union.members.len());
        let mut maximum = 0;
        let mut largest: Option<BasicTypeEnum> = None;

        for member in union.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();
                let typing = self.to_basic_type(&binding.annotation, member.span)?;

                members.push((field.clone(), typing));

                let limit = self.size(typing);

                if limit >= maximum || largest.is_none() {
                    maximum = limit;
                    largest = Some(typing);
                }
            } else {
                self.analysis(member)?;
            }
        }

        if shape.is_opaque() {
            if let Some(target) = largest {
                shape.set_body(&[target], false);
            } else {
                shape.set_body(&[], false);
            }
        }

        self.insert_entity(identifier, Entity::Union { shape, members });

        Ok(self.context.i64_type().const_zero().into())
    }


    pub fn enumeration(
        &mut self,
        enumeration: Structure<Str<'backend>, Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = enumeration.target.clone();
        let name = identifier.as_str().unwrap_or("enumeration");

        let shape = self.context.get_struct_type(name).unwrap_or_else(|| {
            self.context.opaque_struct_type(name)
        });

        let mut members = Vec::with_capacity(enumeration.members.len());
        let mut maximum = 0;
        let mut largest: Option<BasicTypeEnum> = None;
        let mut index = 0;

        for member in enumeration.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();
                members.push((field, index, None));
                index += 1;
            } else {
                let field = match &member.kind {
                    AnalysisKind::Structure(structure) => Some(structure.target.clone()),
                    AnalysisKind::Union(union) => Some(union.target.clone()),
                    AnalysisKind::Enumeration(inner) => Some(inner.target.clone()),
                    _ => None,
                };

                self.analysis(member.clone())?;

                if let Some(target) = field {
                    if let Some(entity) = self.get_entity(&target).cloned() {
                        let typing = match entity {
                            Entity::Structure { shape: structure, .. } => Some(structure.as_basic_type_enum()),
                            Entity::Union { shape: structure, .. } => Some(structure.as_basic_type_enum()),
                            Entity::Enumeration { shape: structure, .. } => Some(structure.as_basic_type_enum()),
                            _ => None,
                        };

                        if let Some(kind) = typing {
                            members.push((target, index, Some(kind)));
                            let limit = self.size(kind);

                            if limit >= maximum || largest.is_none() {
                                maximum = limit;
                                largest = Some(kind);
                            }
                        }
                    }
                    index += 1;
                }
            }
        }

        if shape.is_opaque() {
            let tag = self.context.i64_type().into();
            if let Some(target) = largest {
                shape.set_body(&[tag, target], false);
            } else {
                shape.set_body(&[tag], false);
            }
        }

        self.insert_entity(identifier, Entity::Enumeration { shape, members });

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn constructor(
        &mut self,
        constructor: Structure<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = constructor.target.clone();
        let target = identifier.as_str().unwrap_or("").to_string();

        let entity = self.get_entity(&identifier).cloned();

        match entity {
            Some(Entity::Structure { shape, members }) => {
                let mut current = shape.get_undef();
                let mut position = 0usize;

                for member in constructor.members {
                    let (index, name, assign) = match &member.kind {
                        AnalysisKind::Assign(field, assign) => {
                            let found = members
                                .iter()
                                .position(|item| item == field)
                                .ok_or_else(|| {
                                    GenerateError::new(
                                        ErrorKind::DataStructure(DataStructureError::UnknownField {
                                            target: target.clone(),
                                            member: field.as_str().unwrap_or("").to_string(),
                                        }),
                                        span,
                                    )
                                })?;

                            position = found + 1;

                            (
                                found,
                                field.as_str().unwrap_or("").to_string(),
                                *assign.clone(),
                            )
                        }
                        _ => {
                            if position >= members.len() {
                                return Err(GenerateError::new(
                                    ErrorKind::DataStructure(DataStructureError::TooManyInitializers {
                                        target,
                                    }),
                                    span,
                                ));
                            }

                            let found = position;
                            position += 1;

                            (found, format!("position {}", found), member)
                        }
                    };

                    let kind = shape.get_field_type_at_index(index as u32).unwrap();
                    let value = self.analysis(assign)?;

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

            Some(Entity::Union { shape, members }) => {
                if constructor.members.len() > 1 {
                    return Err(GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::TooManyInitializers {
                            target,
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

                let pointer = self.build_entry(parent, shape.into(), Str::from("initialize"));

                if let Some(member) = constructor.members.into_iter().next() {
                    let (name, assign) = match &member.kind {
                        AnalysisKind::Assign(field, assign) => {
                            (field.as_str().unwrap_or("").to_string(), *assign.clone())
                        }
                        _ => {
                            return Err(GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression),
                                span,
                            ))
                        }
                    };

                    let typing = members
                        .iter()
                        .find(|(item, _)| item.as_str().unwrap_or("") == name)
                        .map(|(_, typing)| *typing)
                        .ok_or_else(|| {
                            GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::UnknownField {
                                    target: target.clone(),
                                    member: name.clone(),
                                }),
                                span,
                            )
                        })?;

                    let value = self.analysis(assign)?;

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
                    target,
                    member: String::from("unknown"),
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
            let namespace = self.has_module(identifier)
                || matches!(
                    self.get_entity(identifier),
                    Some(Entity::Structure { .. } | Entity::Union { .. } | Entity::Enumeration { .. })
                );

            if namespace {
                if let Some(Entity::Enumeration { shape, members }) = self.get_entity(identifier).cloned() {
                    let field = match &member.kind {
                        AnalysisKind::Usage(name) => name.clone(),
                        AnalysisKind::Constructor(constructor) => constructor.target.clone(),
                        _ => return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression), span)),
                    };

                    let found = members.iter().find(|(name, _, _)| name == &field);

                    if let Some((_, index, typing)) = found {
                        let mut current = shape.get_undef();
                        let tag = self.context.i64_type().const_int(*index as u64, false);

                        current = self
                            .builder
                            .build_insert_value(current, tag, 0, "tag")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                            .into_struct_value();

                        return if let Some(kind) = typing {
                            let value = match &member.kind {
                                AnalysisKind::Constructor(constructor) => self.constructor(constructor.clone(), span)?,
                                _ => return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression), span)),
                            };

                            let cast = self.convert(value, *kind).ok_or_else(|| {
                                GenerateError::new(
                                    ErrorKind::DataStructure(DataStructureError::ConstructorFieldTypeMismatch {
                                        struct_name: identifier.as_str().unwrap_or("").to_string(),
                                        field_name: field.as_str().unwrap_or("").to_string(),
                                    }),
                                    span,
                                )
                            })?;

                            let block = self
                                .builder
                                .get_insert_block()
                                .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                            let parent = block
                                .get_parent()
                                .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                            let pointer = self.build_entry(parent, shape.into(), Str::from("enum"));

                            self.builder
                                .build_store(pointer, current)
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            let space = pointer.get_type().get_address_space();

                            let slot = self
                                .builder
                                .build_struct_gep(shape, pointer, 1, "payload")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            let destination = self.context.ptr_type(space);
                            let slot_cast = self
                                .builder
                                .build_pointer_cast(slot, destination, "cast")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            self.builder
                                .build_store(slot_cast, cast)
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                            self
                                .builder
                                .build_load(shape, pointer, "value")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                        } else {
                            Ok(current.into())
                        }
                    }
                }

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
                    let entity = self.find_entity(|item| match item {
                        Entity::Structure { shape: structure, .. } => structure.as_basic_type_enum() == kind,
                        Entity::Union { shape: structure, .. } => structure.as_basic_type_enum() == kind,
                        Entity::Enumeration { shape: structure, .. } => structure.as_basic_type_enum() == kind,
                        _ => false,
                    });

                    let mut position = None;
                    let mut resolution = None;
                    let mut offset = 0;

                    if let Some(Entity::Structure { members: fields, .. }) = entity {
                        position = fields.iter().position(|item| item == &field);
                    } else if let Some(Entity::Union { members: fields, .. }) = entity {
                        resolution = fields.iter().find(|(name, _)| name == &field).map(|(_, typing)| *typing);
                    } else if let Some(Entity::Enumeration { members: fields, .. }) = entity {
                        resolution = fields.iter().find(|(name, _, _)| name == &field).and_then(|(_, _, typing)| *typing);
                        offset = 1;
                    }

                    if let Some(index) = position {
                        let slot = self
                            .builder
                            .build_struct_gep(shape, *pointer, index as u32, "slot")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        let resolved = shape.get_field_type_at_index(index as u32).unwrap();

                        return self
                            .builder
                            .build_load(resolved, slot, "value")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                    } else if let Some(resolved) = resolution {
                        let slot = if offset > 0 {
                            self.builder
                                .build_struct_gep(shape, *pointer, offset, "payload")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                        } else {
                            *pointer
                        };

                        let space = pointer.get_type().get_address_space();
                        let destination = self.context.ptr_type(space);

                        let cast = self
                            .builder
                            .build_pointer_cast(slot, destination, "cast")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        return self
                            .builder
                            .build_load(resolved, cast, "value")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                    }
                }
            }
        }

        let value = self.analysis(*target)?;

        if let BasicValueEnum::StructValue(data) = value {
            let kind = data.get_type().as_basic_type_enum();
            let entity = self.find_entity(|item| match item {
                Entity::Structure { shape: structure, .. } => structure.as_basic_type_enum() == kind,
                Entity::Union { shape: structure, .. } => structure.as_basic_type_enum() == kind,
                Entity::Enumeration { shape: structure, .. } => structure.as_basic_type_enum() == kind,
                _ => false,
            });

            let mut position = None;
            let mut resolution = None;
            let mut offset = 0;

            if let Some(Entity::Structure { members: fields, .. }) = entity {
                position = fields.iter().position(|item| item == &field);
            } else if let Some(Entity::Union { members: fields, .. }) = entity {
                resolution = fields.iter().find(|(name, _)| name == &field).map(|(_, typing)| *typing);
            } else if let Some(Entity::Enumeration { members: fields, .. }) = entity {
                resolution = fields.iter().find(|(name, _, _)| name == &field).and_then(|(_, _, typing)| *typing);
                offset = 1;
            }

            if let Some(index) = position {
                return self
                    .builder
                    .build_extract_value(data, index as u32, "extract")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                    .map(Into::into);
            } else if let Some(resolved) = resolution {
                let block = self
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                let parent = block
                    .get_parent()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                let pointer = self.build_entry(parent, data.get_type().into(), Str::from("spill"));

                self.builder
                    .build_store(pointer, data)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                let slot = if offset > 0 {
                    self.builder
                        .build_struct_gep(data.get_type(), pointer, offset, "payload")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                } else {
                    pointer
                };

                let space = pointer.get_type().get_address_space();
                let destination = self.context.ptr_type(space);

                let cast = self
                    .builder
                    .build_pointer_cast(slot, destination, "cast")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                return self
                    .builder
                    .build_load(resolved, cast, "value")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
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
        items: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if items.is_empty() {
            return Err(GenerateError::new(
                ErrorKind::DataStructure(DataStructureError::EmptyArray),
                span,
            ));
        }

        let mut values = Vec::with_capacity(items.len());

        for item in items {
            values.push(self.analysis(item)?);
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
        items: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut values = Vec::with_capacity(items.len());

        for item in items {
            values.push(self.analysis(item)?);
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
        data: Index<Box<Analysis<'backend>>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if data.members.is_empty() {
            return Err(GenerateError::new(
                ErrorKind::DataStructure(DataStructureError::IndexMissingArgument),
                span,
            ));
        }

        let base = data.target.clone();
        let value = self.analysis(*base.clone())?;
        let offset = self.analysis(data.members[0].clone())?;

        if let AnalysisKind::Usage(identifier) = &data.target.kind {
            if let Some(Entity::Variable { typing, pointer }) = self.get_entity(identifier) {
                let kind = self.to_basic_type(typing, span)?;

                if kind.is_struct_type() {
                    if let BasicValueEnum::IntValue(int) = offset {
                        let index = int.get_zero_extended_constant().ok_or_else(|| {
                            GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::TupleIndexNotConstant),
                                span,
                            )
                        })?;

                        let shape = kind.into_struct_type();
                        let slot = self
                            .builder
                            .build_struct_gep(shape, *pointer, index as u32, "index")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        let field = shape.get_field_type_at_index(index as u32).unwrap();

                        return self
                            .builder
                            .build_load(field, slot, "value")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
                    }
                } else if kind.is_array_type() {
                    if let BasicValueEnum::IntValue(int) = offset {
                        let shape = kind.into_array_type();
                        let limit = int.get_type().const_int(shape.len() as u64, false);

                        let exceeds = self
                            .builder
                            .build_int_compare(IntPredicate::UGE, int, limit, "check")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        let block = self
                            .builder
                            .get_insert_block()
                            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                        let parent = block
                            .get_parent()
                            .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                        let fail = self.context.append_basic_block(parent, "fail");
                        let pass = self.context.append_basic_block(parent, "pass");

                        self.builder
                            .build_conditional_branch(exceeds, fail, pass)
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                        self.builder.position_at_end(fail);
                        self.trap(None, span)?;

                        self.builder.position_at_end(pass);

                        let zero = int.get_type().const_zero();
                        let slot = unsafe {
                            self.builder
                                .build_in_bounds_gep(shape, *pointer, &[zero, int], "index")
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

        match (value, offset) {
            (BasicValueEnum::StructValue(data), BasicValueEnum::IntValue(int)) => {
                let index = int.get_zero_extended_constant().ok_or_else(|| {
                    GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::TupleIndexNotConstant),
                        span,
                    )
                })?;

                return self
                    .builder
                    .build_extract_value(data, index as u32, "extract")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))
                    .map(Into::into);
            }
            (BasicValueEnum::ArrayValue(array), BasicValueEnum::IntValue(int)) => {
                if let Some(index) = int.get_zero_extended_constant() {
                    return self
                        .builder
                        .build_extract_value(array, index as u32, "extract")
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

                let limit = int.get_type().const_int(shape.len() as u64, false);

                let exceeds = self
                    .builder
                    .build_int_compare(IntPredicate::UGE, int, limit, "check")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                let fail = self.context.append_basic_block(parent, "fail");
                let pass = self.context.append_basic_block(parent, "pass");

                self.builder
                    .build_conditional_branch(exceeds, fail, pass)
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                self.builder.position_at_end(fail);
                self.trap(None, span)?;

                self.builder.position_at_end(pass);

                let zero = int.get_type().const_zero();
                let slot = unsafe {
                    self.builder
                        .build_in_bounds_gep(shape, pointer, &[zero, int], "index")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                };

                return self
                    .builder
                    .build_load(shape.get_element_type(), slot, "value")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span));
            }
            (BasicValueEnum::PointerValue(pointer), BasicValueEnum::IntValue(int)) => {
                let pointee = if let TypeKind::Pointer { target } = &base.typing.kind {
                    self.to_basic_type(target, base.span)?
                } else {
                    unreachable!()
                };

                let slot = unsafe {
                    self.builder
                        .build_in_bounds_gep(pointee, pointer, &[int], "index")
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