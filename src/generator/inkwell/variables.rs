use {
    super::{Backend, Entity},
    crate::{
        data::Str,
        generator::{ErrorKind, GenerateError},
        tracker::Span,
    },
    inkwell::{
        types::BasicTypeEnum,
        values::{BasicValueEnum, FunctionValue, PointerValue},
    },
};
use crate::analyzer::{Analysis, Instruction};
use crate::checker::TypeKind;
use crate::data::*;

impl<'backend> super::Inkwell<'backend> {
    fn lvalue_type(&self, analysis: &Analysis<'backend>) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.instruction {
            Instruction::Usage(name) => match self.entities.get(name) {
                Some(Entity::Variable { kind, .. }) => Some(*kind),
                _ => None,
            },
            Instruction::Dereference(operand) => self.pointer_pointee_type(operand),
            _ => None,
        }
    }

    fn pointer_pointee_type(
        &self,
        analysis: &Analysis<'backend>,
    ) -> Option<BasicTypeEnum<'backend>> {
        match &analysis.instruction {
            Instruction::Usage(name) => match self.entities.get(name) {
                Some(Entity::Variable { pointee, .. }) => *pointee,
                _ => None,
            },
            Instruction::AddressOf(operand) => self.lvalue_type(operand),
            Instruction::Dereference(operand) => {
                self.pointer_pointee_type(operand).and_then(|kind| {
                    if kind.is_pointer_type() {
                        None
                    } else {
                        Some(kind)
                    }
                })
            }
            _ => None,
        }
    }

    fn lvalue_pointer(
        &mut self,
        analysis: &Analysis<'backend>,
        function: FunctionValue<'backend>,
    ) -> Option<(PointerValue<'backend>, BasicTypeEnum<'backend>)> {
        match &analysis.instruction {
            Instruction::Usage(name) => match self.entities.get(name) {
                Some(Entity::Variable { pointer, kind, .. }) => Some((*pointer, *kind)),
                _ => None,
            },
            Instruction::Dereference(operand) => {
                let pointee = self.pointer_pointee_type(operand)?;
                let value = self.instruction(operand.instruction.clone(), function);
                match value {
                    BasicValueEnum::PointerValue(pointer) => Some((pointer, pointee)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn address_of(
        &mut self,
        operand: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        if let Some((pointer, _)) = self.lvalue_pointer(&operand, function) {
            pointer.into()
        } else {
            self.context.i64_type().const_zero().into()
        }
    }

    pub fn dereference(
        &mut self,
        operand: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let pointee = self.pointer_pointee_type(&operand);
        let value = self.instruction(operand.instruction.clone(), function);
        match (value, pointee) {
            (BasicValueEnum::PointerValue(pointer), Some(kind)) => self
                .builder
                .build_load(kind, pointer, "deref_value")
                .unwrap_or_else(|_| self.context.i64_type().const_zero().into()),
            _ => self.context.i64_type().const_zero().into(),
        }
    }

    pub fn usage(&self, identifier: Str<'backend>) -> BasicValueEnum<'backend> {
        if let Some(entity) = self.entities.get(&identifier) {
            match entity {
                Entity::Function(function) => {
                    BasicValueEnum::from(function.as_global_value().as_pointer_value())
                }
                Entity::Variable { pointer, kind, .. } => self
                    .builder
                    .build_load(*kind, *pointer, &identifier)
                    .unwrap(),
            }
        } else {
            self.context.i64_type().const_zero().into()
        }
    }

    pub fn assign(
        &mut self,
        target: Str<'backend>,
        value: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let pointee = self.pointer_pointee_type(&value);
        let result = match &value.instruction {
            crate::analyzer::Instruction::Array(elements) => {
                let (value, element_type) = self.build_array(elements.clone(), function);
                self.array_elements.insert(target.clone(), element_type);
                value
            }
            _ => self.instruction(value.instruction.clone(), function),
        };
        let signed = self.infer_signedness(&value);

        if let Some(Entity::Variable { pointer, .. }) = self.entities.get(&target) {
            let slot = *pointer;
            self.builder.build_store(slot, result);
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
            let pointer = self.build_entry_alloca(function, result.get_type(), &target);
            self.builder.build_store(pointer, result);
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
        result
    }

    pub fn store(
        &mut self,
        target: Box<Analysis<'backend>>,
        value: Box<Analysis<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let result = self.instruction(value.instruction.clone(), function);
        if let Some((pointer, kind)) = self.lvalue_pointer(&target, function) {
            if result.get_type() == kind {
                self.builder.build_store(pointer, result);
            } else if result.is_int_value() && kind.is_int_type() {
                let casted = self
                    .builder
                    .build_int_cast(result.into_int_value(), kind.into_int_type(), "store_cast")
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);
                self.builder.build_store(pointer, casted);
            } else if result.is_float_value() && kind.is_float_type() {
                let casted = self
                    .builder
                    .build_float_cast(
                        result.into_float_value(),
                        kind.into_float_type(),
                        "store_cast",
                    )
                    .ok()
                    .map(Into::into)
                    .unwrap_or(result);
                self.builder.build_store(pointer, casted);
            } else {
                self.builder.build_store(pointer, result);
            }
        }
        result
    }

    pub fn binding(
        &mut self,
        binding: Binding<Str<'backend>, Box<Analysis<'backend>>, TypeKind<'backend>>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        let value_analysis = match binding.value {
            Some(value) => value,
            None => {
                self.errors.push(GenerateError::new(
                    ErrorKind::InvalidModule {
                        reason: format!("binding `{}` has no initializer", binding.target),
                    },
                    Span::void(),
                ));

                return self.context.i64_type().const_zero().into();
            }
        };

        let signed = self.infer_signedness(&value_analysis);
        let pointee = self.pointer_pointee_type(&value_analysis);
        let value = match &value_analysis.instruction {
            crate::analyzer::Instruction::Array(elements) => {
                let (value, element_type) = self.build_array(elements.clone(), function);
                self.array_elements
                    .insert(binding.target.clone(), element_type);
                value
            }
            _ => self.instruction(value_analysis.instruction.clone(), function),
        };
        let declared_kind = binding
            .annotation
            .as_ref()
            .map(|annotation| self.llvm_type_from_type_kind(annotation))
            .unwrap_or_else(|| value.get_type());
        let casted = if value.get_type() == declared_kind {
            value
        } else if value.is_int_value() && declared_kind.is_int_type() {
            self.builder
                .build_int_cast(
                    value.into_int_value(),
                    declared_kind.into_int_type(),
                    "bind_cast",
                )
                .ok()
                .map(Into::into)
                .unwrap_or(value)
        } else if value.is_float_value() && declared_kind.is_float_type() {
            self.builder
                .build_float_cast(
                    value.into_float_value(),
                    declared_kind.into_float_type(),
                    "bind_cast",
                )
                .ok()
                .map(Into::into)
                .unwrap_or(value)
        } else {
            value
        };
        let pointer = self.build_entry_alloca(function, declared_kind, &binding.target);
        self.builder.build_store(pointer, casted);
        let signed = binding
            .annotation
            .as_ref()
            .and_then(|annotation| match annotation {
                TypeKind::Integer { signed, .. } => Some(*signed),
                _ => signed,
            });
        self.entities.insert(
            binding.target.clone(),
            Entity::Variable {
                pointer,
                kind: declared_kind,
                pointee,
                signed,
            },
        );
        casted
    }
}
