use {
    super::Entity,
    crate::{
        data::Str,
        generator::Backend,
        resolver::{
            analyzer::{Analysis, Instruction},
            checker::TypeKind,
        },
        schema::{Index, Structure},
    },
    inkwell::{
        types::{BasicType, BasicTypeEnum},
        values::{BasicValueEnum, FunctionValue},
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
            (BasicValueEnum::IntValue(int), target) if target.is_int_type() => Some(
                self.builder
                    .build_int_cast(int, target.into_int_type(), "array_int_cast")
                    .ok()?
                    .into(),
            ),
            (BasicValueEnum::FloatValue(float), target) if target.is_float_type() => Some(
                self.builder
                    .build_float_cast(float, target.into_float_type(), "array_float_cast")
                    .ok()?
                    .into(),
            ),
            (BasicValueEnum::IntValue(int), target) if target.is_float_type() => Some(
                self.builder
                    .build_signed_int_to_float(int, target.into_float_type(), "array_int_to_float")
                    .ok()?
                    .into(),
            ),
            (BasicValueEnum::FloatValue(float), target) if target.is_int_type() => Some(
                self.builder
                    .build_float_to_signed_int(float, target.into_int_type(), "array_float_to_int")
                    .ok()?
                    .into(),
            ),
            _ => None,
        }
    }

    pub(crate) fn annotation_type(
        &mut self,
        analysis: &Analysis<'backend>,
    ) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.instruction {
            Instruction::Usage(name) => name
                .as_str()
                .and_then(TypeKind::from_name)
                .map(|kind| self.llvm_type_from_type_kind(&kind))
                .or_else(|| self.structs.get(name).map(|kind| (*kind).into())),
            Instruction::Array(items) => {
                if items.len() != 2 {
                    return None;
                }
                let member = self.annotation_type(&items[0])?;
                let size = match &items[1].instruction {
                    Instruction::Integer { value, .. } => *value as u32,
                    _ => return None,
                };
                Some(member.array_type(size).into())
            }
            Instruction::Tuple(items) => {
                let mut members = Vec::with_capacity(items.len());
                for item in items {
                    members.push(self.annotation_type(item)?);
                }
                let types: Vec<BasicTypeEnum> = members;
                let struct_type = self.context.struct_type(&types, false);
                Some(struct_type.into())
            }
            Instruction::Dereference(item) => self.annotation_type(item).map(|_| {
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into()
            }),
            _ => None,
        }
    }

    pub fn define_structure(
        &mut self,
        structure: Structure<Str<'backend>, Box<Analysis<'backend>>>,
    ) -> BasicValueEnum<'backend> {
        let name = structure.target.clone();
        if self.structs.get(&name).is_some() {
            return self.context.i64_type().const_zero().into();
        }

        let struct_type = self.context.opaque_struct_type(name.as_str().unwrap());

        let mut field_types = Vec::new();
        let mut field_names = Vec::new();

        for member in &structure.members {
            if let Instruction::Binding(binding) = &member.instruction {
                let field_name = binding.target.clone();
                field_names.push(field_name.clone());
                let field_type = binding
                    .annotation
                    .as_ref()
                    .map(|annotation| self.llvm_type_from_type_kind(annotation))
                    .unwrap_or_else(|| self.context.i64_type().into());
                field_types.push(field_type);
            }
        }

        struct_type.set_body(&field_types, false);
        self.structs.insert(name.clone(), struct_type);
        self.struct_fields.insert(name, field_names);

        self.context.i64_type().const_zero().into()
    }

    pub fn constructor(
        &mut self,
        structure: Structure<Str<'backend>, Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let name = structure.target.clone();
        let struct_type = match self.structs.get(&name) {
            Some(kind) => *kind,
            None => {
                return self.context.i64_type().const_zero().into();
            }
        };

        let field_names = match self.struct_fields.get(&name) {
            Some(names) => names.clone(),
            None => return self.context.i64_type().const_zero().into(),
        };

        let mut value = struct_type.get_undef();
        let mut positional_index = 0usize;
        for member in structure.members {
            match member.instruction {
                Instruction::Assign(field, assigned) => {
                    if let Some(index) = field_names.iter().position(|name| name == &field) {
                        let field_type = struct_type.get_field_type_at_index(index as u32).unwrap();
                        let field_value = self.instruction(assigned.instruction.clone(), function);
                        if let Some(casted) = self.cast_value(field_value, field_type) {
                            value = self
                                .builder
                                .build_insert_value(value, casted, index as u32, "struct_insert")
                                .unwrap()
                                .into_struct_value();
                        }
                    }
                }
                other => {
                    if positional_index >= field_names.len() {
                        continue;
                    }
                    let index = positional_index;
                    positional_index += 1;
                    let field_type = struct_type.get_field_type_at_index(index as u32).unwrap();
                    let field_value = self.instruction(other, function);
                    if let Some(casted) = self.cast_value(field_value, field_type) {
                        value = self
                            .builder
                            .build_insert_value(value, casted, index as u32, "struct_insert")
                            .unwrap()
                            .into_struct_value();
                    }
                }
            }
        }

        value.into()
    }

    pub fn access(
        &mut self,
        target: Box<Analysis<'backend>>,
        member: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if let Instruction::Usage(name) = &target.instruction {
            if self.modules.contains(name) || name.as_str() == Some("stdin") {
                match &member.instruction {
                    Instruction::Usage(name) => return self.usage(name.clone()),
                    Instruction::Invoke(invoke) => return self.invoke(invoke.clone(), function),
                    _ => {}
                }
            }
        }

        let field_name = match &member.instruction {
            Instruction::Usage(name) => name.clone(),
            _ => return self.context.i64_type().const_zero().into(),
        };

        if let Instruction::Usage(target_name) = &target.instruction {
            if let Some(Entity::Variable { pointer, kind, .. }) = self.entities.get(target_name) {
                if kind.is_struct_type() {
                    let struct_type = kind.into_struct_type();
                    let struct_name = self.structs.iter().find_map(|(name, kind)| {
                        if *kind == struct_type {
                            Some(name.clone())
                        } else {
                            None
                        }
                    });

                    if let Some(struct_name) = struct_name {
                        if let Some(fields) = self.struct_fields.get(&struct_name) {
                            if let Some(index) = fields.iter().position(|name| name == &field_name)
                            {
                                if let Ok(slot) = self.builder.build_struct_gep(
                                    struct_type,
                                    *pointer,
                                    index as u32,
                                    "field_ptr",
                                ) {
                                    let field_type =
                                        struct_type.get_field_type_at_index(index as u32).unwrap();
                                    return self
                                        .builder
                                        .build_load(field_type, slot, "field_value")
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

        let target_value = self.instruction(target.instruction, function);
        if let BasicValueEnum::StructValue(struct_value) = target_value {
            if let Some(struct_name) = self.structs.iter().find_map(|(name, kind)| {
                if kind.as_basic_type_enum() == struct_value.get_type().as_basic_type_enum() {
                    Some(name.clone())
                } else {
                    None
                }
            }) {
                if let Some(fields) = self.struct_fields.get(&struct_name) {
                    if let Some(index) = fields.iter().position(|name| name == &field_name) {
                        if let Ok(value) = self.builder.build_extract_value(
                            struct_value,
                            index as u32,
                            "field_extract",
                        ) {
                            return value;
                        }
                    }
                }
            }
        }

        self.context.i64_type().const_zero().into()
    }

    pub(crate) fn build_array(
        &mut self,
        elements: Vec<Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> (BasicValueEnum<'backend>, BasicTypeEnum<'backend>) {
        if elements.is_empty() {
            let array_type = self.context.i8_type().array_type(0);
            let ptr = self
                .builder
                .build_alloca(array_type, "array_empty")
                .unwrap();
            return (ptr.into(), self.context.i8_type().into());
        }

        let mut values = Vec::with_capacity(elements.len());
        for element in elements {
            let value = self.instruction(element.instruction, function);
            values.push(value);
        }

        let element_type = values[0].get_type();
        let array_type = element_type.array_type(values.len() as u32);
        let ptr = self.builder.build_alloca(array_type, "array").unwrap();

        let zero = self.context.i32_type().const_zero();
        for (index, value) in values.into_iter().enumerate() {
            let idx = self.context.i32_type().const_int(index as u64, false);
            let slot = unsafe {
                self.builder
                    .build_in_bounds_gep(array_type, ptr, &[zero, idx], "array_index")
                    .unwrap()
            };
            if let Some(casted) = self.cast_value(value, element_type) {
                self.builder.build_store(slot, casted);
            }
        }

        let first = unsafe {
            self.builder
                .build_in_bounds_gep(
                    array_type,
                    ptr,
                    &[
                        self.context.i32_type().const_zero(),
                        self.context.i32_type().const_zero(),
                    ],
                    "array_first",
                )
                .unwrap()
        };

        (first.into(), element_type)
    }

    pub fn array(
        &mut self,
        elements: Vec<Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let (value, _) = self.build_array(elements, function);
        value
    }

    pub fn tuple(
        &mut self,
        elements: Vec<Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let mut values = Vec::with_capacity(elements.len());
        let mut types = Vec::with_capacity(elements.len());

        for element in elements {
            let value = self.instruction(element.instruction, function);
            types.push(value.get_type());
            values.push(value);
        }

        let struct_type = self.context.struct_type(&types, false);
        let mut current = struct_type.get_undef();
        for (index, value) in values.into_iter().enumerate() {
            current = self
                .builder
                .build_insert_value(current, value, index as u32, "tuple_insert")
                .unwrap()
                .into_struct_value();
        }

        current.into()
    }

    pub fn index(
        &mut self,
        index: Index<Box<Analysis<'backend>>, Box<Analysis<'backend>>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if index.members.is_empty() {
            return self.context.i64_type().const_zero().into();
        }

        let target_instruction = index.target.instruction.clone();
        let target = self.instruction(target_instruction, function);
        let idx_value = self.instruction(index.members[0].instruction.clone(), function);

        if let Instruction::Usage(name) = &index.target.instruction {
            if let Some(element_type) = self.array_elements.get(name) {
                if let BasicValueEnum::PointerValue(pointer) = target {
                    if let BasicValueEnum::IntValue(idx) = idx_value {
                        let slot = unsafe {
                            self.builder
                                .build_in_bounds_gep(*element_type, pointer, &[idx], "array_index")
                                .unwrap()
                        };
                        return self
                            .builder
                            .build_load(*element_type, slot, "array_value")
                            .unwrap();
                    }
                }
            } else if let Some(Entity::Variable { kind, pointer, .. }) = self.entities.get(name) {
                if kind.is_struct_type() {
                    if let BasicValueEnum::IntValue(idx) = idx_value {
                        if let Some(constant) = idx.get_zero_extended_constant() {
                            let struct_type = kind.into_struct_type();
                            if let Ok(slot) = self.builder.build_struct_gep(
                                struct_type,
                                *pointer,
                                constant as u32,
                                "tuple_index",
                            ) {
                                return self
                                    .builder
                                    .build_load(
                                        struct_type
                                            .get_field_type_at_index(constant as u32)
                                            .unwrap(),
                                        slot,
                                        "tuple_value",
                                    )
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }
        if let BasicValueEnum::StructValue(struct_value) = target {
            if let BasicValueEnum::IntValue(idx) = idx_value {
                if let Some(constant) = idx.get_zero_extended_constant() {
                    if let Ok(value) = self.builder.build_extract_value(
                        struct_value,
                        constant as u32,
                        "tuple_extract",
                    ) {
                        return value;
                    }
                }
            }
        } else if let BasicValueEnum::ArrayValue(array_value) = target {
            if let BasicValueEnum::IntValue(idx) = idx_value {
                if let Some(constant) = idx.get_zero_extended_constant() {
                    if let Ok(value) = self.builder.build_extract_value(
                        array_value,
                        constant as u32,
                        "array_extract",
                    ) {
                        return value;
                    }
                }
            }
        }

        self.context.i64_type().const_zero().into()
    }
}
