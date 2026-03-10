use inkwell::IntPredicate;
use inkwell::values::PointerValue;
use {
    super::{Entity, Backend, GenerateError, super::ErrorKind},
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
    fn cast_value(
        &self,
        value: BasicValueEnum<'backend>,
        target: BasicTypeEnum<'backend>,
    ) -> Option<BasicValueEnum<'backend>> {
        if value.get_type() == target {
            return Some(value);
        }

        match (value, target) {
            (BasicValueEnum::IntValue(int), target) if target.is_int_type() => self.builder
                .build_int_cast(int, target.into_int_type(), "array_int_cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(float), target) if target.is_float_type() => self.builder
                .build_float_cast(float, target.into_float_type(), "array_float_cast")
                .ok()
                .map(Into::into),
            (BasicValueEnum::IntValue(int), target) if target.is_float_type() => self.builder
                .build_signed_int_to_float(int, target.into_float_type(), "array_int_to_float")
                .ok()
                .map(Into::into),
            (BasicValueEnum::FloatValue(float), target) if target.is_int_type() => self.builder
                .build_float_to_signed_int(float, target.into_int_type(), "array_float_to_int")
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
        let name = structure.target.clone();
        let name_str = name.as_str().unwrap_or("anonymous_struct");

        let struct_type = self.context.opaque_struct_type(name_str);

        let mut field_types = Vec::new();
        let mut fields = Vec::new();

        for member in &structure.members {
            if let AnalysisKind::Binding(binding) = &member.kind {
                let field_name = binding.target.clone();
                fields.push(field_name.clone());

                let field_type = if let Some(annotation) = binding.annotation.as_ref() {
                    self.llvm_type(annotation)?
                } else {
                    return Err(GenerateError::new(
                        ErrorKind::SemanticError {
                            message: format!("Struct field '{}' in '{}' is missing a type annotation.", field_name, name)
                        },
                        span
                    ))
                };

                field_types.push(field_type);
            }
        }

        struct_type.set_body(&field_types, false);

        self.entities.insert(
            name,
            Entity::Struct {
                struct_type,
                fields,
            }
        );

        // Note: Defining a struct is a compile-time concept. Returning a dummy i64 zero
        // to maintain compatibility with your current expression-oriented loop.
        Ok(self.context.i64_type().const_zero().into())
    }

    pub fn constructor(
        &mut self,
        structure: Structure<Str<'backend>, Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        let name = structure.target.clone();

        let (struct_type, fields) = if let Some(entity) = self.entities.get(&name) {
            if let Entity::Struct { struct_type, fields } = entity {
                (*struct_type, fields.clone())
            } else {
                return Err(GenerateError::new(ErrorKind::SemanticError { message: format!("'{}' is not a struct type.", name) }, span));
            }
        } else {
            return Err(GenerateError::new(ErrorKind::SemanticError { message: format!("Unknown struct type '{}'.", name) }, span));
        };

        let mut value = struct_type.get_undef();
        let mut positional_index = 0usize;

        for member in structure.members {
            match member.kind {
                AnalysisKind::Assign(field, assigned) => {
                    if let Some(index) = fields.iter().position(|name| name == &field) {
                        let field_type = struct_type.get_field_type_at_index(index as u32).unwrap();
                        let field_value = self.analysis(*assigned.clone())?;

                        let casted = self.cast_value(field_value, field_type).ok_or_else(|| {
                            GenerateError::new(ErrorKind::SemanticError { message: format!("Type mismatch for field '{}' in constructor for '{}'.", field, name) }, span)
                        })?;

                        value = self.builder
                            .build_insert_value(value, casted, index as u32, "struct_insert")
                            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                            .into_struct_value();
                    } else {
                        return Err(GenerateError::new(ErrorKind::SemanticError { message: format!("Struct '{}' has no field named '{}'.", name, field) }, span));
                    }
                }
                _ => {
                    if positional_index >= fields.len() {
                        return Err(GenerateError::new(ErrorKind::SemanticError { message: format!("Too many positional initializers for struct '{}'.", name) }, span));
                    }

                    let index = positional_index;
                    positional_index += 1;

                    let field_type = struct_type.get_field_type_at_index(index as u32).unwrap();
                    let field_value = self.analysis(member)?;

                    let casted = self.cast_value(field_value, field_type).ok_or_else(|| {
                        GenerateError::new(ErrorKind::SemanticError { message: format!("Type mismatch for positional argument {} in constructor for '{}'.", index, name) }, span)
                    })?;

                    value = self.builder
                        .build_insert_value(value, casted, index as u32, "struct_insert")
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
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
        // Module access
        if let AnalysisKind::Usage(name) = &target.kind {
            if self.modules.contains_key(name) {
                match &member.kind {
                    AnalysisKind::Usage(name) => return self.usage(name.clone(), span),
                    AnalysisKind::Invoke(invoke) => return self.invoke(invoke.clone(), span),
                    _ => return Err(GenerateError::new(ErrorKind::SemanticError { message: "Invalid module access.".to_string() }, span)),
                }
            }
        }

        let field_name = if let AnalysisKind::Usage(name) = &member.kind {
            name.clone()
        } else {
            return Err(GenerateError::new(ErrorKind::SemanticError { message: "Struct member access must use a simple name.".to_string() }, span));
        };

        // Pointer / Variable Field Access
        if let AnalysisKind::Usage(target_name) = &target.kind {
            if let Some(Entity::Variable { pointer, kind, .. }) = self.entities.get(target_name) {
                if kind.is_struct_type() {
                    let struct_type = kind.into_struct_type();

                    // Optimized Struct Field Lookup
                    let mut found_fields = None;
                    for entity in self.entities.values() {
                        if let Entity::Struct { struct_type: ent_struct, fields } = entity {
                            if *ent_struct == struct_type {
                                found_fields = Some(fields);
                                break;
                            }
                        }
                    }

                    if let Some(fields) = found_fields {
                        if let Some(index) = fields.iter().position(|name| name == &field_name) {
                            let slot = self.builder.build_struct_gep(
                                struct_type,
                                *pointer,
                                index as u32,
                                "field_ptr",
                            ).map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

                            let field_type = struct_type.get_field_type_at_index(index as u32).unwrap();
                            return self.builder.build_load(field_type, slot, "field_value")
                                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span));
                        }
                    }
                }
            }
        }

        // Direct Value Extract Access
        let target_value = self.analysis(*target)?;
        if let BasicValueEnum::StructValue(struct_value) = target_value {
            let mut found_fields = None;
            for entity in self.entities.values() {
                if let Entity::Struct { struct_type, fields } = entity {
                    if struct_type.as_basic_type_enum() == struct_value.get_type().as_basic_type_enum() {
                        found_fields = Some(fields);
                        break;
                    }
                }
            }

            if let Some(fields) = found_fields {
                if let Some(index) = fields.iter().position(|name| name == &field_name) {
                    return self.builder.build_extract_value(struct_value, index as u32, "field_extract")
                        .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))
                        .map(|val| val.into());
                }
            }
        }

        Err(GenerateError::new(ErrorKind::SemanticError { message: format!("Attempted to access field '{}' on a non-struct type or value.", field_name) }, span))
    }

    pub fn array(
        &mut self,
        elements: Vec<Analysis<'backend>>,
        span: Span<'backend>,
    ) -> Result<(PointerValue<'backend>, BasicTypeEnum<'backend>), GenerateError<'backend>> {
        if elements.is_empty() {
            return Err(GenerateError::new(ErrorKind::SemanticError { message: "Cannot create an empty array without a type annotation.".to_string() }, span));
        }

        let mut values = Vec::with_capacity(elements.len());

        for element in elements {
            let value = self.analysis(element)?;
            values.push(value);
        }

        let element_type = values[0].get_type();
        let array_type = element_type.array_type(values.len() as u32);

        let ptr = self.builder.build_alloca(array_type, "array")
            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

        let zero = self.context.i32_type().const_zero();

        for (index, value) in values.into_iter().enumerate() {
            let idx = self.context.i32_type().const_int(index as u64, false);

            let slot = unsafe {
                self.builder
                    .build_in_bounds_gep(array_type, ptr, &[zero, idx], "array_index")
                    .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
            };

            let casted = self.cast_value(value, element_type).ok_or_else(|| {
                GenerateError::new(ErrorKind::SemanticError { message: format!("Type mismatch in array literal. Element {} has an incompatible type.", index) }, span)
            })?;

            self.builder.build_store(slot, casted)
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
        }

        Ok((ptr, element_type))
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

        let types: Vec<BasicTypeEnum> = values.iter().map(|v| v.get_type()).collect();

        let struct_type = self.context.struct_type(&types, false);
        let mut current = struct_type.get_undef();

        for (index, value) in values.into_iter().enumerate() {
            current = self.builder
                .build_insert_value(current, value, index as u32, "tuple_insert")
                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
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
            return Err(GenerateError::new(ErrorKind::SemanticError { message: "Index operation requires at least one index argument.".to_string() }, span));
        }

        let target_instruction = index.target.clone();
        let target = self.analysis(*target_instruction)?;
        let idx_value = self.analysis(index.members[0].clone())?;

        if let AnalysisKind::Usage(name) = &index.target.kind {
            // Pointer Indexing (Arrays)
            if let Some(Entity::Array { element_type, element_count, .. }) = self.entities.get(name) {
                if let BasicValueEnum::PointerValue(pointer) = target {
                    if let BasicValueEnum::IntValue(idx) = idx_value {
                        let length_val = self.context.i32_type().const_int(*element_count as u64, false);

                        let is_idx_out_of_bounds = self.builder.build_int_compare(
                            IntPredicate::UGE,
                            idx,
                            length_val,
                            "array_bounds_check"
                        ).map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

                        let current_block = self.builder.get_insert_block().unwrap();
                        let function = current_block.get_parent().unwrap();

                        let trap_block = self.context.append_basic_block(function, "trap_oob");
                        let continue_block = self.context.append_basic_block(function, "continue_oob");

                        self.builder.build_conditional_branch(is_idx_out_of_bounds, trap_block, continue_block)
                            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

                        self.builder.position_at_end(trap_block);
                        if let Some(trap_fn) = self.current_module().get_function("llvm.trap") {
                            self.builder.build_call(trap_fn, &[], "trap_call")
                                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;
                        }

                        self.builder.build_unreachable()
                            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

                        self.builder.position_at_end(continue_block);

                        let slot = unsafe {
                            self.builder
                                .build_in_bounds_gep(*element_type, pointer, &[idx], "array_index")
                                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?
                        };

                        return self.builder.build_load(*element_type, slot, "array_value")
                            .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span));
                    }
                }
            }
            // Pointer Indexing (Struct/Tuples)
            else if let Some(Entity::Variable { kind, pointer, .. }) = self.entities.get(name) {
                if kind.is_struct_type() {
                    if let BasicValueEnum::IntValue(idx) = idx_value {
                        if let Some(constant) = idx.get_zero_extended_constant() {
                            let struct_type = kind.into_struct_type();
                            let slot = self.builder.build_struct_gep(
                                struct_type,
                                *pointer,
                                constant as u32,
                                "tuple_index",
                            ).map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))?;

                            let field_type = struct_type.get_field_type_at_index(constant as u32).unwrap();
                            return self.builder.build_load(field_type, slot, "tuple_value")
                                .map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span));
                        } else {
                            return Err(GenerateError::new(ErrorKind::SemanticError { message: "Tuple index must be a compile-time constant.".to_string() }, span));
                        }
                    }
                }
            }
        }

        // Direct Value Extraction (Structs/Tuples)
        if let BasicValueEnum::StructValue(struct_value) = target {
            if let BasicValueEnum::IntValue(idx) = idx_value {
                if let Some(constant) = idx.get_zero_extended_constant() {
                    return self.builder.build_extract_value(
                        struct_value,
                        constant as u32,
                        "tuple_extract",
                    ).map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))
                        .map(Into::into);
                } else {
                    return Err(GenerateError::new(ErrorKind::SemanticError { message: "Tuple index must be a compile-time constant.".to_string() }, span));
                }
            }
        }
        // Direct Value Extraction (Arrays)
        else if let BasicValueEnum::ArrayValue(array_value) = target {
            if let BasicValueEnum::IntValue(idx) = idx_value {
                if let Some(constant) = idx.get_zero_extended_constant() {
                    return self.builder.build_extract_value(
                        array_value,
                        constant as u32,
                        "array_extract",
                    ).map_err(|e| GenerateError::new(ErrorKind::BuilderError { reason: e.to_string() }, span))
                        .map(Into::into);
                } else {
                    return Err(GenerateError::new(ErrorKind::SemanticError { message: "Array value index must be a compile-time constant.".to_string() }, span));
                }
            }
        }

        Err(GenerateError::new(ErrorKind::SemanticError { message: "Type cannot be indexed or invalid index provided.".to_string() }, span))
    }
}
