use inkwell::AddressSpace;
use inkwell::types::{AnyTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, IntValue};
use {
    super::GenerateError,
    crate::{
        data::Str,
        internal::hash::Map,
        resolver::analyzer::{Analysis, Instruction},
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        types::{AnyType, BasicType},
        values::{AnyValue, BasicValueEnum, FunctionValue, PointerValue},
        FloatPredicate, IntPredicate,
    },
    std::fs::File,
    std::io::Write,
};

pub trait Backend<'backend> {
    fn generate(&mut self, analysis: Vec<Analysis<'backend>>);

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend>;

    fn print(&self);

    fn write_to_file(&self, filename: &str) -> std::io::Result<()>;
}

pub struct Generator<'generator, B: Backend<'generator>> {
    pub backend: B,
    pub errors: Vec<GenerateError<'generator>>,
}

impl<'generator, B: Backend<'generator>> Generator<'generator, B> {
    pub fn new(backend: B) -> Self {
        Self { backend, errors: Vec::new() }
    }
}

pub struct Inkwell<'backend> {
    context: &'backend Context,
    builder: Builder<'backend>,
    module: Module<'backend>,
    variables: Map<Str<'backend>, PointerValue<'backend>>,
    variable_types: Map<Str<'backend>, BasicTypeEnum<'backend>>,
    functions: Map<Str<'backend>, FunctionValue<'backend>>,
    print: FunctionValue<'backend>,
}

impl<'backend> Inkwell<'backend> {
    pub fn new(module: Str<'backend>, context: &'backend Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(&module);

        // Fix: Use printf instead of custom print function
        let printf_type = context.i32_type().fn_type(
            &[context.ptr_type(AddressSpace::default()).into()],
            true,
        );
        let printf = module.add_function("printf", printf_type, Some(inkwell::module::Linkage::External));

        let mut functions = Map::new();
        functions.insert(Str::from("print"), printf);

        Self {
            context,
            builder,
            module,
            variables: Map::new(),
            variable_types: Map::new(),
            functions,
            print: printf,
        }
    }
}

impl<'backend> Backend<'backend> for Inkwell<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        // First pass: Define all methods/functions
        for analysis in &analyses {
            if let Instruction::Method(_) = analysis.instruction {
                self.generate_instruction(analysis.instruction.clone(),
                                          self.module.add_function("dummy", self.context.void_type().fn_type(&[], false), None));
            }
        }

        let function_type = self.context.i32_type().fn_type(&[], false);
        let function = self.module.add_function("main", function_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let mut last_value = BasicValueEnum::from(self.context.i64_type().const_zero());

        // Second pass: Generate other instructions
        for analysis in analyses {
            if !matches!(analysis.instruction, Instruction::Method(_)) {
                last_value = self.generate_instruction(analysis.instruction, function);
            }
        }

        // Convert the last value to i32 for return
        let return_value = if last_value.is_int_value() {
            let int_val = last_value.into_int_value();
            // If it's not i32, truncate or extend as needed
            if int_val.get_type().get_bit_width() != 32 {
                if int_val.get_type().get_bit_width() > 32 {
                    self.builder.build_int_truncate(int_val, self.context.i32_type(), "trunc").unwrap()
                } else {
                    self.builder.build_int_z_extend(int_val, self.context.i32_type(), "ext").unwrap()
                }
            } else {
                int_val
            }
        } else if last_value.is_float_value() {
            // Convert float to i32
            self.builder.build_float_to_signed_int(
                last_value.into_float_value(),
                self.context.i32_type(),
                "fptosi"
            ).unwrap()
        } else {
            // Default to 0
            self.context.i32_type().const_zero()
        };

        self.builder.build_return(Some(&return_value));
    }

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        match instruction {
            Instruction::Integer(int, scale) => {
                let int_type = match scale {
                    8 => self.context.i8_type(),
                    16 => self.context.i16_type(),
                    32 => self.context.i32_type(),
                    64 => self.context.i64_type(),
                    _ => {
                        self.context.i64_type()
                    }
                };
                let unsigned: u64 = int as u64;
                BasicValueEnum::from(int_type.const_int(unsigned, false))
            }
            Instruction::Float(float, scale) => {
                let float_type = match scale {
                    32 => self.context.f32_type(),
                    64 => self.context.f64_type(),
                    _ => {
                        self.context.f64_type()
                    }
                };
                BasicValueEnum::from(float_type.const_float(float.0))
            }
            Instruction::Boolean(boolean) => {
                BasicValueEnum::from(self.context.bool_type().const_int(boolean as u64, false))
            }
            Instruction::Add(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_add(
                        left.into_int_value(),
                        right.into_int_value(),
                        "add",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_add(
                        left.into_float_value(),
                        right.into_float_value(),
                        "add",
                    ).unwrap())
                }
            }
            Instruction::Subtract(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_sub(
                        left.into_int_value(),
                        right.into_int_value(),
                        "sub",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_sub(
                        left.into_float_value(),
                        right.into_float_value(),
                        "sub",
                    ).unwrap())
                }
            }
            Instruction::Multiply(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_mul(
                        left.into_int_value(),
                        right.into_int_value(),
                        "mul",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_mul(
                        left.into_float_value(),
                        right.into_float_value(),
                        "mul",
                    ).unwrap())
                }
            }
            Instruction::Divide(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_signed_div(
                        left.into_int_value(),
                        right.into_int_value(),
                        "div",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_div(
                        left.into_float_value(),
                        right.into_float_value(),
                        "div",
                    ).unwrap())
                }
            }
            Instruction::Modulus(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_int_signed_rem(
                    left.into_int_value(),
                    right.into_int_value(),
                    "mod",
                ).unwrap())
            }
            Instruction::LogicalAnd(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_and(
                    left.into_int_value(),
                    right.into_int_value(),
                    "and",
                ).unwrap())
            }
            Instruction::LogicalOr(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_or(
                    left.into_int_value(),
                    right.into_int_value(),
                    "or",
                ).unwrap())
            }
            Instruction::LogicalNot(operand) => {
                let operand_val = self.generate_instruction(operand.instruction, function);

                BasicValueEnum::from(self.builder.build_not(
                    operand_val.into_int_value(),
                    "not",
                ).unwrap())
            }
            Instruction::BitwiseAnd(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_and(
                    left.into_int_value(),
                    right.into_int_value(),
                    "bitand",
                ).unwrap())
            }
            Instruction::BitwiseOr(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_or(
                    left.into_int_value(),
                    right.into_int_value(),
                    "bitor",
                ).unwrap())
            }
            Instruction::BitwiseNot(operand) => {
                let operand_val = self.generate_instruction(operand.instruction, function);

                BasicValueEnum::from(self.builder.build_not(
                    operand_val.into_int_value(),
                    "bitnot",
                ).unwrap())
            }
            Instruction::BitwiseXOr(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_xor(
                    left.into_int_value(),
                    right.into_int_value(),
                    "bitxor",
                ).unwrap())
            }
            Instruction::ShiftLeft(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_left_shift(
                    left.into_int_value(),
                    right.into_int_value(),
                    "shl",
                ).unwrap())
            }
            Instruction::ShiftRight(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                BasicValueEnum::from(self.builder.build_right_shift(
                    left.into_int_value(),
                    right.into_int_value(),
                    true,
                    "shr",
                ).unwrap())
            }
            Instruction::Equal(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_compare(
                        IntPredicate::EQ,
                        left.into_int_value(),
                        right.into_int_value(),
                        "eq",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_compare(
                        FloatPredicate::OEQ,
                        left.into_float_value(),
                        right.into_float_value(),
                        "eq",
                    ).unwrap())
                }
            }
            Instruction::NotEqual(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_compare(
                        IntPredicate::NE,
                        left.into_int_value(),
                        right.into_int_value(),
                        "ne",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_compare(
                        FloatPredicate::ONE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "ne",
                    ).unwrap())
                }
            }
            Instruction::Less(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_compare(
                        IntPredicate::SLT,
                        left.into_int_value(),
                        right.into_int_value(),
                        "lt",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_compare(
                        FloatPredicate::OLT,
                        left.into_float_value(),
                        right.into_float_value(),
                        "lt",
                    ).unwrap())
                }
            }
            Instruction::LessOrEqual(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_compare(
                        IntPredicate::SLE,
                        left.into_int_value(),
                        right.into_int_value(),
                        "le",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_compare(
                        FloatPredicate::OLE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "le",
                    ).unwrap())
                }
            }
            Instruction::Greater(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_compare(
                        IntPredicate::SGT,
                        left.into_int_value(),
                        right.into_int_value(),
                        "gt",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_compare(
                        FloatPredicate::OGT,
                        left.into_float_value(),
                        right.into_float_value(),
                        "gt",
                    ).unwrap())
                }
            }
            Instruction::GreaterOrEqual(left, right) => {
                let left = self.generate_instruction(left.instruction, function);
                let right = self.generate_instruction(right.instruction, function);

                if left.is_int_value() && right.is_int_value() {
                    BasicValueEnum::from(self.builder.build_int_compare(
                        IntPredicate::SGE,
                        left.into_int_value(),
                        right.into_int_value(),
                        "ge",
                    ).unwrap())
                } else {
                    BasicValueEnum::from(self.builder.build_float_compare(
                        FloatPredicate::OGE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "ge",
                    ).unwrap())
                }
            }
            Instruction::Usage(identifier) => {
                if let Some(func) = self.functions.get(&identifier) {
                    BasicValueEnum::from(func.as_global_value().as_pointer_value())
                } else if let Some(ptr) = self.variables.get(&identifier) {
                    if let Some(element_type) = self.variable_types.get(&identifier) {
                        self.builder.build_load(*element_type, *ptr, &identifier).unwrap()
                    } else {
                        self.context.i64_type().const_zero().into()
                    }
                } else {
                    self.context.i64_type().const_zero().into()
                }
            }
            Instruction::Assign(assign) => {
                let value_result = self.generate_instruction(assign.value.instruction.clone(), function);

                if let Some(ptr) = self.variables.get(&assign.target) {
                    self.builder.build_store(*ptr, value_result);
                    // Update the type information
                    self.variable_types.insert(assign.target.clone(), value_result.get_type());
                } else {
                    let ptr = if value_result.is_int_value() {
                        self.builder.build_alloca(value_result.get_type(), &assign.target)
                    } else if value_result.is_float_value() {
                        self.builder.build_alloca(value_result.get_type(), &assign.target)
                    } else {
                        self.builder.build_alloca(value_result.get_type(), &assign.target)
                    }.unwrap();

                    self.builder.build_store(ptr, value_result);
                    self.variables.insert(assign.target.clone(), ptr);
                    // Store the type information
                    self.variable_types.insert(assign.target, value_result.get_type());
                }

                value_result
            }
            Instruction::Binding(binding) => {
                let value = self.generate_instruction(binding.value.unwrap().instruction.clone(), function);

                let ptr = self.builder.build_alloca(value.get_type(), &binding.target).unwrap();

                self.builder.build_store(ptr, value);
                self.variables.insert(binding.target.clone(), ptr);

                self.variable_types.insert(binding.target, value.get_type());

                value
            }
            Instruction::Module(name, analyses) => {
                let function_type = self.context.void_type().fn_type(&[], false);
                let function = self.module.add_function(&name, function_type, None);
                let basic_block = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(basic_block);

                for analysis in analyses {
                    self.generate_instruction(analysis.instruction, function);
                }

                self.builder.build_return(None);
                BasicValueEnum::from(self.context.i64_type().const_zero())
            }
            Instruction::Method(method) => {
                // Build parameter types
                let mut param_types = vec![];
                for param in &method.members {
                    if let Instruction::Binding(bind) = &param.instruction {
                        if let Some(ann) = &bind.annotation {
                            if let Instruction::Usage(ty_name) = &ann.instruction {
                                let ty = match ty_name.as_str().unwrap() {
                                    "Integer" => self.context.i64_type().into(),
                                    "Float" => self.context.f64_type().into(),
                                    "Boolean" => self.context.bool_type().into(),
                                    _ => self.context.i64_type().into(),
                                };
                                param_types.push(ty);
                            }
                        } else {
                            // Default to i64 if no annotation
                            param_types.push(self.context.i64_type().into());
                        }
                    }
                }

                // Determine return type
                let ret_ty = method.output.as_ref().map_or(
                    self.context.void_type().as_any_type_enum(),
                    |o| {
                        if let Instruction::Usage(ty_name) = &o.instruction {
                            match ty_name.as_str().unwrap() {
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

                // Create function type
                let fn_type = if ret_ty.is_void_type() {
                    self.context.void_type().fn_type(&param_types, false)
                } else {
                    match ret_ty {
                        AnyTypeEnum::IntType(int_type) => int_type.fn_type(&param_types, false),
                        AnyTypeEnum::FloatType(float_type) => float_type.fn_type(&param_types, false),
                        AnyTypeEnum::VoidType(void_type) => void_type.fn_type(&param_types, false),
                        _ => self.context.void_type().fn_type(&param_types, false)
                    }
                };

                let func_name = method.target.as_str().unwrap();
                let func = self.module.add_function(func_name, fn_type, None);

                // Store the function in our functions map IMMEDIATELY
                self.functions.insert(method.target.clone(), func);

                // Handle special case for print function
                if func_name == "print" {
                    let basic_block = self.context.append_basic_block(func, "entry");

                    // Save current builder position
                    let current_bb = self.builder.get_insert_block();

                    self.builder.position_at_end(basic_block);

                    if !param_types.is_empty() {
                        let value_param = func.get_nth_param(0).unwrap();

                        // Determine format string based on parameter type
                        let format_str = if param_types[0].is_int_type() {
                            "%lld\n"
                        } else if param_types[0].is_float_type() {
                            "%f\n"
                        } else {
                            "%d\n"
                        };

                        let format_global = self.builder.build_global_string_ptr(format_str, "fmt")
                            .expect("Failed to build global string");
                        let args: Vec<BasicMetadataValueEnum> = vec![
                            format_global.as_pointer_value().into(),
                            value_param.into()
                        ];

                        self.builder.build_call(self.print, &args, "print_call");
                    }

                    self.builder.build_return(None);

                    // Restore builder position if we had one
                    if let Some(bb) = current_bb {
                        self.builder.position_at_end(bb);
                    }
                } else {
                    // Handle other methods normally
                    if let Instruction::Block(block) = &method.body.instruction {
                        let basic_block = self.context.append_basic_block(func, "entry");

                        // Save current builder position
                        let current_bb = self.builder.get_insert_block();

                        self.builder.position_at_end(basic_block);

                        // Set up parameters
                        let mut i = 0;
                        for param in &method.members {
                            if let Instruction::Binding(bind) = &param.instruction {
                                if let Some(param_val) = func.get_nth_param(i) {
                                    let alloca_ty = param_val.get_type();
                                    let ptr = self.builder.build_alloca(alloca_ty, &bind.target).unwrap();
                                    self.builder.build_store(ptr, param_val);
                                    self.variables.insert(bind.target.clone(), ptr);
                                    i += 1;
                                }
                            }
                        }

                        // Generate method body
                        let mut last_value = self.context.i64_type().const_zero().into();
                        for item in &block.items {
                            last_value = self.generate_instruction(item.instruction.clone(), func);
                        }

                        // Return
                        if ret_ty.is_void_type() {
                            self.builder.build_return(None);
                        } else {
                            self.builder.build_return(Some(&last_value));
                        }

                        // Restore builder position
                        if let Some(bb) = current_bb {
                            self.builder.position_at_end(bb);
                        }
                    }
                }

                self.context.i64_type().const_zero().into()
            }

            Instruction::Invoke(invoke) => {
                if let Instruction::Usage(func_name) = &invoke.target.instruction {
                    let func_opt = self.functions.get(func_name).cloned();

                    if let Some(func) = func_opt {
                        let mut args = vec![];
                        for arg in &invoke.arguments {
                            let arg_val = self.generate_instruction(arg.instruction.clone(), function);
                            args.push(arg_val.into());
                        }

                        let result = self.builder.build_call(func, &args, "call").unwrap();
                        result.try_as_basic_value().left().unwrap_or(self.context.i64_type().const_zero().into())
                    } else {
                        self.context.i64_type().const_zero().into()
                    }
                } else {
                    self.context.i64_type().const_zero().into()
                }
            }

            Instruction::Return(value) => {
                match value {
                    Some(v) => {
                        let val = self.generate_instruction(v.instruction, function);
                        self.builder.build_return(Some(&val));
                        val
                    }
                    None => {
                        self.builder.build_return(None);
                        self.context.i64_type().const_zero().into()
                    }
                }
            }
            _ => BasicValueEnum::from(self.context.i64_type().const_zero())
        }
    }

    fn print(&self) {
        let ir = self.module.print_to_string();
        println!("{}", ir.to_string());
    }

    fn write_to_file(&self, filename: &str) -> std::io::Result<()> {
        let ir = self.module.print_to_string();
        let mut file = File::create(filename)?;
        file.write_all(ir.to_string().as_bytes())?;
        Ok(())
    }
}