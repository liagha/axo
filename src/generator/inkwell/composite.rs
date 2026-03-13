use inkwell::values::IntValue;
use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::{Index, Str, Structure},
        generator::{
            inkwell::{Backend, Entity, GenerateError, Inkwell},
            DataStructureError, ErrorKind,
        },
        tracker::Span,
    },
    inkwell::{
        types::{BasicType, BasicTypeEnum},
        values::BasicValueEnum,
        IntPredicate,
    },
};
use crate::generator::BuilderError;

impl<'backend> Inkwell<'backend> {
    fn fields(&self, target: BasicTypeEnum<'backend>) -> Option<Vec<Str<'backend>>> {
        if let Some(Entity::Struct { fields, .. }) = self.find_entity(|entity| {
            matches!(entity, Entity::Struct { structure, .. } if structure.as_basic_type_enum() == target)
        }) {
            Some(fields.clone())
        } else {
            None
        }
    }

    fn union_fields(&self, target: BasicTypeEnum<'backend>) -> Option<Vec<(Str<'backend>, BasicTypeEnum<'backend>)>> {
        if let Some(Entity::Union { fields, .. }) = self.find_entity(|entity| {
            matches!(entity, Entity::Union { structure, .. } if structure.as_basic_type_enum() == target)
        }) {
            Some(fields.clone())
        } else {
            None
        }
    }

    fn size(&self, typ: BasicTypeEnum<'backend>) -> u64 {
        match typ {
            BasicTypeEnum::IntType(integer) => (integer.get_bit_width() as u64 + 7) / 8,
            BasicTypeEnum::FloatType(float) => (float.get_bit_width() as u64 + 7) / 8,
            BasicTypeEnum::PointerType(_) => 8,
            BasicTypeEnum::ArrayType(array) => array.len() as u64 * self.size(array.get_element_type()),
            BasicTypeEnum::StructType(structure) => {
                let mut size = 0;

                for index in 0..structure.count_fields() {
                    if let Some(field_ty) = structure.get_field_type_at_index(index) {
                        size += self.size(field_ty);
                    }
                }

                size
            }
            BasicTypeEnum::VectorType(vector) => vector.get_size() as u64 * self.size(vector.get_element_type()),
            BasicTypeEnum::ScalableVectorType(_) => {
                unimplemented!("Statically sizing scalable vectors for unions is not supported")
            }
        }
    }

    pub fn trap(
        &self,
        condition: Option<IntValue<'backend>>,
        span: Span<'backend>,
    ) -> Result<(), GenerateError<'backend>> {

        let trap_fn = self.current_module()
            .get_function("llvm.trap")
            .unwrap_or_else(|| {
                let trap_type = self.context.void_type().fn_type(&[], false);
                self.current_module().add_function("llvm.trap", trap_type, None)
            });

        match condition {
            None => {
                self.builder.build_call(trap_fn, &[], "trap")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError(e.into()), span))?;

                self.builder.build_unreachable()
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError(e.into()), span))?;
            }

            Some(condition) => {
                let block = self.builder.get_insert_block()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::BlockInsertion), span))?;

                let parent = block.get_parent()
                    .ok_or_else(|| GenerateError::new(ErrorKind::BuilderError(BuilderError::Parent), span))?;

                let failure = self.context.append_basic_block(parent, "trap");
                let success = self.context.append_basic_block(parent, "cont");

                self.builder.build_conditional_branch(condition, failure, success)
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError(e.into()), span))?;

                self.builder.position_at_end(failure);

                self.builder.build_call(trap_fn, &[], "")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError(e.into()), span))?;

                self.builder.build_unreachable()
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError(e.into()), span))?;

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
            (BasicValueEnum::IntValue(integer), target) if target.is_int_type() => self
                .builder
                .build_int_cast(integer, target.into_int_type(), "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(float), target) if target.is_float_type() => self
                .builder
                .build_float_cast(float, target.into_float_type(), "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::IntValue(integer), target) if target.is_float_type() => self
                .builder
                .build_signed_int_to_float(integer, target.into_float_type(), "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(float), target) if target.is_int_type() => self
                .builder
                .build_float_to_signed_int(float, target.into_int_type(), "cast")
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
        let string = identifier.as_str().unwrap_or("structure");

        let shape = self.context.get_struct_type(string).unwrap_or_else(|| {
            self.context.opaque_struct_type(string)
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

        self.insert_entity(
            identifier,
            Entity::Struct {
                structure: shape,
                fields,
            },
        );

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn union(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        _span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();
        let string = identifier.as_str().unwrap_or("union");

        let shape = self.context.get_struct_type(string).unwrap_or_else(|| {
            self.context.opaque_struct_type(string)
        });

        let mut fields = Vec::with_capacity(structure.members.len());
        let mut largest_type: Option<BasicTypeEnum> = None;
        let mut max_size = 0;

        for member in &structure.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();

                let typ = self.to_basic_type(&binding.annotation, member.span)?;
                fields.push((field.clone(), typ));

                let size = self.size(typ);
                if size >= max_size || largest_type.is_none() {
                    max_size = max_size.max(size);
                    largest_type = Some(typ);
                }
            }
        }

        if shape.is_opaque() {
            if let Some(largest) = largest_type {
                shape.set_body(&[largest], false);
            } else {
                shape.set_body(&[], false);
            }
        }

        self.insert_entity(
            identifier,
            Entity::Union {
                structure: shape,
                fields,
            },
        );

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn constructor(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();
        let name_str = identifier.as_str().unwrap_or("").to_string();

        let entity = self.get_entity(&identifier).cloned();

        match entity {
            Some(Entity::Struct { structure: shape, fields }) => {
                let mut value = shape.get_undef();
                let mut position = 0usize;

                for member in structure.members {
                    let (index, field_name, assigned) = match &member.kind {
                        AnalysisKind::Assign(field, assigned) => {
                            let idx = fields
                                .iter()
                                .position(|item| item == field)
                                .ok_or_else(|| {
                                    GenerateError::new(
                                        ErrorKind::DataStructure(DataStructureError::UnknownField {
                                            struct_name: name_str.clone(),
                                            field_name: field.as_str().unwrap_or("").to_string(),
                                        }),
                                        span,
                                    )
                                })?;
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
                                        struct_name: name_str,
                                    }),
                                    span,
                                ));
                            }
                            let idx = position;
                            position += 1;
                            (idx, format!("positional arg {}", idx), member)
                        }
                    };

                    let kind = shape.get_field_type_at_index(index as u32).unwrap();
                    let evaluated = self.analysis(assigned)?;

                    let casted = self.convert(evaluated, kind).ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::DataStructure(DataStructureError::ConstructorFieldTypeMismatch {
                                struct_name: name_str.clone(),
                                field_name,
                            }),
                            span,
                        )
                    })?;

                    value = self
                        .builder
                        .build_insert_value(value, casted, index as u32, "insert")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                        .into_struct_value();
                }

                Ok(value.into())
            }

            Some(Entity::Union { structure: shape, fields }) => {
                if structure.members.len() > 1 {
                    return Err(GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::TooManyInitializers {
                            struct_name: name_str,
                        }),
                        span,
                    ));
                }

                let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let pointer = self.build_entry(function, shape.into(), Str::from("union_init"));

                if let Some(member) = structure.members.into_iter().next() {
                    let (field_name, assigned) = match &member.kind {
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

                    let field_type = fields
                        .iter()
                        .find(|(name, _)| name.as_str().unwrap_or("") == field_name)
                        .map(|(_, typ)| *typ)
                        .ok_or_else(|| {
                            GenerateError::new(
                                ErrorKind::DataStructure(DataStructureError::UnknownField {
                                    struct_name: name_str.clone(),
                                    field_name: field_name.clone(),
                                }),
                                span,
                            )
                        })?;

                    let evaluated = self.analysis(assigned)?;

                    let casted = self.convert(evaluated, field_type).ok_or_else(|| {
                        GenerateError::new(
                            ErrorKind::DataStructure(DataStructureError::ConstructorFieldTypeMismatch {
                                struct_name: name_str.clone(),
                                field_name,
                            }),
                            span,
                        )
                    })?;

                    self.builder.build_store(pointer, casted).map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?;
                }

                let loaded = self.builder
                    .build_load(shape, pointer, "union_val")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?;

                Ok(loaded)
            }

            _ => Err(GenerateError::new(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    struct_name: name_str.clone(),
                    field_name: String::from("unknown_structure"),
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
            if let Some(Entity::Variable { pointer, typ }) = self.get_entity(identifier) {
                let kind = self.to_basic_type(typ, span)?;

                if kind.is_struct_type() {
                    let shape = kind.into_struct_type();

                    if let Some(fields) = self.fields(kind) {
                        if let Some(index) = fields.iter().position(|item| item == &field) {
                            let slot = self
                                .builder
                                .build_struct_gep(shape, *pointer, index as u32, "pointer")
                                .map_err(|error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                })?;
                            let resolved = shape.get_field_type_at_index(index as u32).unwrap();

                            return self.builder.build_load(resolved, slot, "value").map_err(
                                |error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                },
                            );
                        }
                    } else if let Some(fields) = self.union_fields(kind) {
                        if let Some((_, field_type)) = fields.iter().find(|(name, _)| name == &field) {
                            return self.builder.build_load(*field_type, *pointer, "value").map_err(
                                |error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                },
                            );
                        }
                    }
                }
            }
        }

        let evaluated = self.analysis(*target)?;

        if let BasicValueEnum::StructValue(structure) = evaluated {
            if let Some(fields) = self.fields(structure.get_type().as_basic_type_enum()) {
                if let Some(index) = fields.iter().position(|item| item == &field) {
                    return self
                        .builder
                        .build_extract_value(structure, index as u32, "extract")
                        .map_err(|error| {
                            GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                        })
                        .map(Into::into);
                }
            } else if let Some(fields) = self.union_fields(structure.get_type().as_basic_type_enum()) {
                if let Some((_, field_type)) = fields.iter().find(|(name, _)| name == &field) {
                    let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let pointer = self.build_entry(function, structure.get_type().into(), Str::from("union_spill"));

                    self.builder.build_store(pointer, structure).map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?;

                    return self.builder.build_load(*field_type, pointer, "value").map_err(
                        |error| {
                            GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                        },
                    );
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
        let shape = kind.array_type(values.len() as u32);
        let mut current = shape.get_undef();

        for (index, value) in values.into_iter().enumerate() {
            let casted = self.convert(value, kind).ok_or_else(|| {
                GenerateError::new(
                    ErrorKind::DataStructure(DataStructureError::ArrayLiteralTypeMismatch {
                        index,
                    }),
                    span,
                )
            })?;

            current = self
                .builder
                .build_insert_value(current, casted, index as u32, "insert")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError(error.into()), span))?
                .into_array_value();
        }

        Ok(current.into())
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
        let target = self.analysis(*base)?;
        let offset = self.analysis(index.members[0].clone())?;

        if let AnalysisKind::Usage(identifier) = &index.target.kind {
            if let Some(Entity::Variable { typ, pointer }) = self.get_entity(identifier) {
                let kind = self.to_basic_type(typ, span)?;

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
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            })?;

                        let field = shape.get_field_type_at_index(constant as u32).unwrap();
                        return self
                            .builder
                            .build_load(field, slot, "value")
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            });
                    }
                } else if kind.is_array_type() {
                    if let BasicValueEnum::IntValue(integer) = offset {
                        let shape = kind.into_array_type();

                        let length = self.context.i32_type().const_int(shape.len() as u64, false);
                        let exceeds = self
                            .builder
                            .build_int_compare(IntPredicate::UGE, integer, length, "check")
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            })?;

                        let function = self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_parent()
                            .unwrap();
                        let trap_block = self.context.append_basic_block(function, "trap");
                        let resume_block = self.context.append_basic_block(function, "resume");

                        self.builder
                            .build_conditional_branch(exceeds, trap_block, resume_block)
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            })?;

                        self.builder.position_at_end(trap_block);
                        self.trap(None, span)?;

                        self.builder.position_at_end(resume_block);
                        let zero = self.context.i32_type().const_zero();
                        let slot = unsafe {
                            self.builder
                                .build_in_bounds_gep(shape, *pointer, &[zero, integer], "index")
                                .map_err(|error| {
                                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                                })?
                        };

                        return self
                            .builder
                            .build_load(shape.get_element_type(), slot, "value")
                            .map_err(|error| {
                                GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                            });
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
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })
                    .map(Into::into);
            }
            (BasicValueEnum::ArrayValue(array), BasicValueEnum::IntValue(integer)) => {
                if let Some(constant) = integer.get_zero_extended_constant() {
                    return self
                        .builder
                        .build_extract_value(array, constant as u32, "extract")
                        .map_err(|error| {
                            GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                        })
                        .map(Into::into);
                }

                let shape = array.get_type();
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let pointer = self.build_entry(function, shape.into(), Str::from("array_spill"));

                self.builder.build_store(pointer, array).map_err(|error| {
                    GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                })?;

                let length = self.context.i32_type().const_int(shape.len() as u64, false);
                let exceeds = self
                    .builder
                    .build_int_compare(IntPredicate::UGE, integer, length, "check")
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?;

                let trap_block = self.context.append_basic_block(function, "trap");
                let resume_block = self.context.append_basic_block(function, "resume");

                self.builder
                    .build_conditional_branch(exceeds, trap_block, resume_block)
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    })?;

                self.builder.position_at_end(trap_block);
                self.trap(None, span)?;

                self.builder.position_at_end(resume_block);
                let zero = self.context.i32_type().const_zero();
                let slot = unsafe {
                    self.builder
                        .build_in_bounds_gep(shape, pointer, &[zero, integer], "index")
                        .map_err(|error| {
                            GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                        })?
                };

                return self
                    .builder
                    .build_load(shape.get_element_type(), slot, "value")
                    .map_err(|error| {
                        GenerateError::new(ErrorKind::BuilderError(error.into()), span)
                    });
            }
            _ => {}
        }

        Err(GenerateError::new(
            ErrorKind::DataStructure(DataStructureError::NotIndexable),
            span,
        ))
    }
}