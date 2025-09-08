
use {
    inkwell::{
        values::{
            BasicValueEnum, FunctionValue,
        }
    },
    crate::{
        data::{
            Str,
        },
        schema::*,
        resolver::{
            analyzer::{
                Analysis
            }
        }
    },
    super::Backend,
};

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_usage(&self, identifier: Str<'backend>) -> BasicValueEnum<'backend> {
        if let Some(function) = self.functions.get(&identifier) {
            BasicValueEnum::from(function.as_global_value().as_pointer_value())
        } else if let Some(pointer) = self.variables.get(&identifier) {
            if let Some(kind) = self.types.get(&identifier) {
                self.builder.build_load(*kind, *pointer, &identifier).unwrap()
            } else {
                self.context.i64_type().const_zero().into()
            }
        } else {
            self.context.i64_type().const_zero().into()
        }
    }

    pub fn generate_assign(&mut self, target: Str<'backend>, value: Box<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let result = self.generate_instruction(value.instruction.clone(), function);
        if let Some(pointer) = self.variables.get(&target) {
            self.builder.build_store(*pointer, result);
            self.types.insert(target.clone(), result.get_type());
        } else {
            let pointer = if result.is_int_value() {
                self.builder.build_alloca(result.get_type(), &target)
            } else if result.is_float_value() {
                self.builder.build_alloca(result.get_type(), &target)
            } else {
                self.builder.build_alloca(result.get_type(), &target)
            }.unwrap();
            self.builder.build_store(pointer, result);
            self.variables.insert(target.clone(), pointer);
            self.types.insert(target, result.get_type());
        }
        result
    }

    pub fn generate_binding(&mut self, binding: Binding<Str<'backend>, Box<Analysis<'backend>>, Box<Analysis<'backend>>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let value = self.generate_instruction(binding.value.unwrap().instruction.clone(), function);
        let pointer = self.builder.build_alloca(value.get_type(), &binding.target).unwrap();
        self.builder.build_store(pointer, value);
        self.variables.insert(binding.target.clone(), pointer);
        self.types.insert(binding.target, value.get_type());
        value
    }
}