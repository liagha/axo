use inkwell::IntPredicate;
use {
    super::{Entity, Backend, GenerateError, super::{ErrorKind, DataStructureError}},
    crate::{
        data::{Str, Index, Structure},
        analyzer::{Analysis, AnalysisKind},
        tracker::Span,
    },
    inkwell::{
        types::{BasicType, BasicTypeEnum},
        values::BasicValueEnum,
    },
};

impl<'backend> super::Inkwell<'backend> {
    fn convert(
        &self,
        value: BasicValueEnum<'backend>,
        target: BasicTypeEnum<'backend>,
    ) -> Option<BasicValueEnum<'backend>> {
        if value.get_type() == target {
            return Some(value);
        }

        match (value, target) {
            (BasicValueEnum::IntValue(integer), target) if target.is_int_type() => self.builder
                .build_int_cast(integer, target.into_int_type(), "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(float), target) if target.is_float_type() => self.builder
                .build_float_cast(float, target.into_float_type(), "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::IntValue(integer), target) if target.is_float_type() => self.builder
                .build_signed_int_to_float(integer, target.into_float_type(), "cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(float), target) if target.is_int_type() => self.builder
                .build_float_to_signed_int(float, target.into_int_type(), "cast")
                .ok()
                .map(Into::into),
            _ => None,
        }
    }

    pub fn structure(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();
        let string = identifier.as_str().unwrap_or("structure");

        let shape = self.context.opaque_struct_type(string);

        let mut types = Vec::new();
        let mut fields = Vec::new();

        for member in &structure.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field = binding.target.clone();
                fields.push(field.clone());

                let kind = if let Some(annotation) = binding.annotation.as_ref() {
                    self.llvm_type(annotation, member.span)?
                } else {
                    return Err(GenerateError::new(
                        ErrorKind::DataStructure(DataStructureError::FieldMissingAnnotation {
                            struct_name: identifier.to_string(),
                            field_name: field.to_string(),
                        }),
                        span,
                    ));
                };

                types.push(kind);
            }
        }

        shape.set_body(&types, false);

        self.entities.insert(
            identifier,
            Entity::Struct {
                struct_type: shape,
                fields,
            }
        );

        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn constructor(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let identifier = structure.target.clone();

        let (shape, fields) = if let Some(entity) = self.entities.get(&identifier) {
            if let Entity::Struct { struct_type: defined, fields } = entity {
                (*defined, fields.clone())
            } else {
                return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::NotAStructType { name: identifier.to_string() }), span));
            }
        } else {
            return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::UnknownStructType { name: identifier.to_string() }), span));
        };

        let mut value = shape.get_undef();
        let mut position = 0usize;

        for member in structure.members {
            match member.kind {
                AnalysisKind::Assign(field, assigned) => {
                    if let Some(index) = fields.iter().position(|item| item == &field) {
                        let kind = shape.get_field_type_at_index(index as u32).unwrap();
                        let evaluated = self.analysis(*assigned.clone())?;

                        let casted = self.convert(evaluated, kind).ok_or_else(|| {
                            GenerateError::new(ErrorKind::DataStructure(DataStructureError::ConstructorFieldTypeMismatch { struct_name: identifier.to_string(), field_name: field.to_string() }), span)
                        })?;

                        value = self.builder
                            .build_insert_value(value, casted, index as u32, "insert")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
                            .into_struct_value();
                    } else {
                        return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::UnknownField { struct_name: identifier.to_string(), field_name: field.to_string() }), span));
                    }
                }
                _ => {
                    if position >= fields.len() {
                        return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::TooManyInitializers { struct_name: identifier.to_string() }), span));
                    }

                    let index = position;
                    position += 1;

                    let kind = shape.get_field_type_at_index(index as u32).unwrap();
                    let evaluated = self.analysis(member)?;

                    let casted = self.convert(evaluated, kind).ok_or_else(|| {
                        GenerateError::new(ErrorKind::DataStructure(DataStructureError::ConstructorPositionalArgTypeMismatch { struct_name: identifier.to_string(), index }), span)
                    })?;

                    value = self.builder
                        .build_insert_value(value, casted, index as u32, "insert")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
                        .into_struct_value();
                }
            }
        }

        Ok(value.into())
    }

    pub fn access(
        &mut self,
        target: Box<Analysis<'backend>>,
        member: Box<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if let AnalysisKind::Usage(identifier) = &target.kind {
            if self.modules.contains_key(identifier) {
                match &member.kind {
                    AnalysisKind::Usage(name) => return self.usage(name.clone(), span),
                    AnalysisKind::Invoke(invoke) => return self.invoke(invoke.clone(), span),
                    _ => return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::InvalidModuleAccess), span)),
                }
            }
        }

        let field = if let AnalysisKind::Usage(identifier) = &member.kind {
            identifier.clone()
        } else {
            return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression), span));
        };

        if let AnalysisKind::Usage(identifier) = &target.kind {
            if let Some(Entity::Variable { pointer, kind, .. }) = self.entities.get(identifier) {
                if kind.is_struct_type() {
                    let shape = kind.into_struct_type();

                    let mut found = None;
                    for entity in self.entities.values() {
                        if let Entity::Struct { struct_type: defined, fields } = entity {
                            if *defined == shape {
                                found = Some(fields);
                                break;
                            }
                        }
                    }

                    if let Some(fields) = found {
                        if let Some(index) = fields.iter().position(|item| item == &field) {
                            let slot = self.builder.build_struct_gep(
                                shape,
                                *pointer,
                                index as u32,
                                "pointer",
                            ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;

                            let resolved = shape.get_field_type_at_index(index as u32).unwrap();
                            return self.builder.build_load(resolved, slot, "value")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span));
                        }
                    }
                }
            }
        }

        let evaluated = self.analysis(*target)?;
        if let BasicValueEnum::StructValue(structure) = evaluated {
            let mut found = None;
            for entity in self.entities.values() {
                if let Entity::Struct { struct_type: defined, fields } = entity {
                    if defined.as_basic_type_enum() == structure.get_type().as_basic_type_enum() {
                        found = Some(fields);
                        break;
                    }
                }
            }

            if let Some(fields) = found {
                if let Some(index) = fields.iter().position(|item| item == &field) {
                    return self.builder.build_extract_value(structure, index as u32, "extract")
                        .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))
                        .map(|value| value.into());
                }
            }
        }

        Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::AccessOnNonStructType { field_name: field.to_string() }), span))
    }

    pub fn array(
        &mut self,
        elements: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        if elements.is_empty() {
            return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::EmptyArray), span));
        }

        let mut values = Vec::with_capacity(elements.len());

        for element in elements {
            let value = self.analysis(element)?;
            values.push(value);
        }

        let kind = values[0].get_type();
        let shape = kind.array_type(values.len() as u32);

        let pointer = self.builder.build_alloca(shape, "array")
            .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;

        let zero = self.context.i32_type().const_zero();

        for (index, value) in values.into_iter().enumerate() {
            let offset = self.context.i32_type().const_int(index as u64, false);

            let slot = unsafe {
                self.builder
                    .build_in_bounds_gep(shape, pointer, &[zero, offset], "index")
                    .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
            };

            let casted = self.convert(value, kind).ok_or_else(|| {
                GenerateError::new(ErrorKind::DataStructure(DataStructureError::ArrayLiteralTypeMismatch { index }), span)
            })?;

            self.builder.build_store(slot, casted)
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;
        }

        Ok(pointer.into())
    }

    pub fn tuple(
        &mut self,
        elements: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let mut values = Vec::new();

        for element in elements {
            let value = self.analysis(element)?;
            values.push(value);
        }

        let types: Vec<BasicTypeEnum> = values.iter().map(|item| item.get_type()).collect();

        let shape = self.context.struct_type(&types, false);
        let mut current = shape.get_undef();

        for (index, value) in values.into_iter().enumerate() {
            current = self.builder
                .build_insert_value(current, value, index as u32, "insert")
                .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
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
            return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::IndexMissingArgument), span));
        }

        let base = index.target.clone();
        let target = self.analysis(*base)?;
        let offset = self.analysis(index.members[0].clone())?;

        if let AnalysisKind::Usage(identifier) = &index.target.kind {
            if let Some(Entity::Array { element_type: element, element_count: count, .. }) = self.entities.get(identifier) {
                if let BasicValueEnum::PointerValue(pointer) = target {
                    if let BasicValueEnum::IntValue(integer) = offset {
                        let length = self.context.i32_type().const_int(*count as u64, false);

                        let exceeds = self.builder.build_int_compare(
                            IntPredicate::UGE,
                            integer,
                            length,
                            "check"
                        ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;

                        let block = self.builder.get_insert_block().unwrap();
                        let function = block.get_parent().unwrap();

                        let trap = self.context.append_basic_block(function, "trap");
                        let resume = self.context.append_basic_block(function, "resume");

                        self.builder.build_conditional_branch(exceeds, trap, resume)
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;

                        self.builder.position_at_end(trap);
                        if let Some(callable) = self.current_module().get_function("llvm.trap") {
                            self.builder.build_call(callable, &[], "trap")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;
                        }

                        self.builder.build_unreachable()
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;

                        self.builder.position_at_end(resume);

                        let slot = unsafe {
                            self.builder
                                .build_in_bounds_gep(*element, pointer, &[integer], "index")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?
                        };

                        return self.builder.build_load(*element, slot, "value")
                            .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span));
                    }
                }
            } else if let Some(Entity::Variable { kind, pointer, .. }) = self.entities.get(identifier) {
                if kind.is_struct_type() {
                    if let BasicValueEnum::IntValue(integer) = offset {
                        if let Some(constant) = integer.get_zero_extended_constant() {
                            let shape = kind.into_struct_type();
                            let slot = self.builder.build_struct_gep(
                                shape,
                                *pointer,
                                constant as u32,
                                "index",
                            ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))?;

                            let field = shape.get_field_type_at_index(constant as u32).unwrap();
                            return self.builder.build_load(field, slot, "value")
                                .map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span));
                        } else {
                            return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::TupleIndexNotConstant), span));
                        }
                    }
                }
            }
        }

        if let BasicValueEnum::StructValue(structure) = target {
            if let BasicValueEnum::IntValue(integer) = offset {
                if let Some(constant) = integer.get_zero_extended_constant() {
                    return self.builder.build_extract_value(
                        structure,
                        constant as u32,
                        "extract",
                    ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))
                        .map(Into::into);
                } else {
                    return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::TupleIndexNotConstant), span));
                }
            }
        } else if let BasicValueEnum::ArrayValue(array) = target {
            if let BasicValueEnum::IntValue(integer) = offset {
                if let Some(constant) = integer.get_zero_extended_constant() {
                    return self.builder.build_extract_value(
                        array,
                        constant as u32,
                        "extract",
                    ).map_err(|error| GenerateError::new(ErrorKind::BuilderError { reason: error.to_string() }, span))
                        .map(Into::into);
                } else {
                    return Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::ArrayIndexNotConstant), span));
                }
            }
        }

        Err(GenerateError::new(ErrorKind::DataStructure(DataStructureError::NotIndexable), span))
    }
}