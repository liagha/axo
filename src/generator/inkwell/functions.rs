use inkwell::types::AnyTypeEnum;
use {
    inkwell::{
        types::{AnyType, BasicType}, 
        values::{
            BasicMetadataValueEnum, 
            BasicValueEnum,
            FunctionValue,
            AnyValueEnum,
        },
    },
    crate::{
        data::{
            Str,
        },
        schema::{
            Method, Invoke, 
        },
        generator::{
            Backend,
        },
        resolver::{
            analyzer::{Analysis, Instruction},
        },
    }
};

impl<'backend> super::Inkwell<'backend> {
    pub fn generate_module(&mut self, name: Str<'backend>, analyses: Vec<Analysis<'backend>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        let function_type = self.context.void_type().fn_type(&[], false);
        let function = self.module.add_function(&name, function_type, None);
        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);
        for analysis in analyses {
            self.generate_instruction(analysis.instruction, function);
        }
        self.builder.build_return(None);
        BasicValueEnum::from(self.context.i64_type().const_zero())
    }

    pub fn generate_method(&mut self, method: Method<Str<'backend>, Box<Analysis<'backend>>, Box<Analysis<'backend>>, Option<Box<Analysis<'backend>>>>) -> BasicValueEnum<'backend> {
        let mut parameters = vec![];
        for member in &method.members {
            if let Instruction::Binding(bind) = &member.instruction {
                if let Some(annotation) = &bind.annotation {
                    if let Instruction::Usage(name) = &annotation.instruction {
                        let kind = match name.as_str().unwrap() {
                            "Integer" => self.context.i64_type().into(),
                            "Float" => self.context.f64_type().into(),
                            "Boolean" => self.context.bool_type().into(),
                            _ => self.context.i64_type().into(),
                        };
                        parameters.push(kind);
                    }
                } else {
                    parameters.push(self.context.i64_type().into());
                }
            }
        }
        let return_kind = method.output.as_ref().map_or(
            self.context.void_type().as_any_type_enum(),
            |output| {
                if let Instruction::Usage(name) = &output.instruction {
                    match name.as_str().unwrap() {
                        "Integer" => self.context.i64_type().into(),
                        "Float" => self.context.f64_type().into(),
                        "Boolean" => self.context.bool_type().into(),
                        _ => self.context.void_type().into(),
                    }
                } else {
                    self.context.void_type().into()
                }
            }
        );
        let function_type = if return_kind.is_void_type() {
            self.context.void_type().fn_type(&parameters, false)
        } else {
            match return_kind {
                AnyTypeEnum::IntType(integer) => integer.fn_type(&parameters, false),
                AnyTypeEnum::FloatType(float) => float.fn_type(&parameters, false),
                AnyTypeEnum::VoidType(void) => void.fn_type(&parameters, false),
                _ => self.context.void_type().fn_type(&parameters, false)
            }
        };
        let name = method.target.as_str().unwrap();
        let function = self.module.add_function(name, function_type, None);
        self.functions.insert(method.target.clone(), function);
        self.context.i64_type().const_zero().into()
    }

    pub fn generate_invoke(&mut self, invoke: Invoke<Box<Analysis<'backend>>, Box<Analysis<'backend>>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        if let Instruction::Usage(name) = &invoke.target.instruction {
            let option = self.functions.get(name).cloned();
            if let Some(value) = option {
                if name.as_str().unwrap() == "printf" {
                    let mut arguments = vec![];
                    if !invoke.arguments.is_empty() {
                        let first = self.generate_instruction(invoke.arguments[0].instruction.clone(), function);
                        let format = if first.is_int_value() {
                            "%d\n"
                        } else if first.is_float_value() {
                            "%f\n"
                        } else {
                            "%s\n"
                        };
                        let pointer = self.builder.build_global_string_ptr(format, "format").unwrap().as_pointer_value();
                        arguments.push(BasicMetadataValueEnum::from(pointer));
                        arguments.push(BasicMetadataValueEnum::from(first));
                        for argument in invoke.arguments.iter().skip(1) {
                            let value = self.generate_instruction(argument.instruction.clone(), function);
                            arguments.push(BasicMetadataValueEnum::from(value));
                        }
                    } else {
                        let pointer = self.builder.build_global_string_ptr("\n", "format").unwrap().as_pointer_value();
                        arguments.push(BasicMetadataValueEnum::from(pointer));
                    }
                    let result = self.builder.build_call(value, &arguments, "printf_call").unwrap();
                    result.try_as_basic_value().left().unwrap_or(self.context.i32_type().const_zero().into())
                } else {
                    let mut arguments = vec![];
                    for argument in &invoke.arguments {
                        let value = self.generate_instruction(argument.instruction.clone(), function);
                        arguments.push(value.into());
                    }
                    let result = self.builder.build_call(value, &arguments, "call").unwrap();
                    result.try_as_basic_value().left().unwrap_or(self.context.i64_type().const_zero().into())
                }
            } else {
                self.context.i64_type().const_zero().into()
            }
        } else {
            self.context.i64_type().const_zero().into()
        }
    }

    pub fn generate_return(&mut self, value: Option<Box<Analysis<'backend>>>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        match value {
            Some(item) => {
                let result = self.generate_instruction(item.instruction, function);
                self.builder.build_return(Some(&result));
                result
            }
            None => {
                self.builder.build_return(None);
                self.context.i64_type().const_zero().into()
            }
        }
    }
}