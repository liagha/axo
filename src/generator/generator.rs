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
};

pub trait Backend<'backend> {
    fn generate(&mut self, analysis: Vec<Analysis<'backend>>);

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend>;

    fn print(&self);
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
}

impl<'backend> Inkwell<'backend> {
    pub fn new(module: Str<'backend>, context: &'backend Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(&module);

        Self {
            context,
            builder,
            module,
            variables: Map::new(),
        }
    }
}

impl<'backend> Backend<'backend> for Inkwell<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        let function_type = self.context.i64_type().fn_type(&[], false);
        let function = self.module.add_function("main", function_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let mut last_value = BasicValueEnum::from(self.context.i64_type().const_zero());
        
        for analysis in analyses {
            last_value = BasicValueEnum::from(self.generate_instruction(analysis.instruction, function));
        }

        self.builder.build_return(Some(&last_value));
    }

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        match instruction {
            Instruction::Integer(int) => {
                let unsigned: u64 = int as u64;
                BasicValueEnum::from(self.context.i64_type().const_int(unsigned, true))
            }
            Instruction::Float(float) => {
                BasicValueEnum::from(self.context.f64_type().const_float(float.0))
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
                let ptr = self.variables.get(&identifier).unwrap();
                self.builder.build_load(ptr.get_type(), *ptr, &identifier).unwrap()
            }
            Instruction::Assign(assign) => {
                let value_result = self.generate_instruction(assign.get_value().instruction.clone(), function);

                if let Some(ptr) = self.variables.get(assign.get_target()) {
                    self.builder.build_store(*ptr, value_result);
                } else {
                    let ptr = if value_result.is_int_value() {
                        self.builder.build_alloca(self.context.i64_type(), &assign.get_target())
                    } else if value_result.is_float_value() {
                        self.builder.build_alloca(self.context.f64_type(), assign.get_target())
                    } else {
                        self.builder.build_alloca(self.context.bool_type(), assign.get_target())
                    }.unwrap();

                    self.builder.build_store(ptr, value_result);
                    self.variables.insert(*assign.get_target(), ptr);
                }

                value_result
            }
            Instruction::Binding(binding) => {
                let value = self.generate_instruction(binding.get_value().unwrap().instruction.clone(), function);

                let ptr = if value.is_int_value() {
                    self.builder.build_alloca(self.context.i64_type(), &binding.get_target())
                } else if value.is_float_value() {
                    self.builder.build_alloca(self.context.f64_type(), &binding.get_target())
                } else {
                    self.builder.build_alloca(self.context.bool_type(), &binding.get_target())
                }.unwrap();

                self.builder.build_store(ptr, value);
                self.variables.insert(*binding.get_target(), ptr);
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
            _ => BasicValueEnum::from(self.context.i64_type().const_zero())
        }
    }

    fn print(&self) {
        let ir = self.module.print_to_string();
        println!("{}", ir.to_string());
    }
}