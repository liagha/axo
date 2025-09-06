mod primitives;
mod arithmetic;
mod logical;
mod bitwise;
mod comparison;
mod variables;
mod functions;

use {
    inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::Module,
        types::{
            AnyTypeEnum, BasicTypeEnum
        },
        values::{
            BasicValueEnum,
            FunctionValue, 
            PointerValue
        },
    },

    crate::{
        data::{
            Str,
        },
        internal::{
            hash::Map,
            platform::Write,
        },
        resolver::{
            analyzer::{Analysis, Instruction}
        }
    },
    
    super::Backend,
};

pub struct Inkwell<'backend> {
    context: &'backend Context,
    builder: Builder<'backend>,
    module: Module<'backend>,
    variables: Map<Str<'backend>, PointerValue<'backend>>,
    types: Map<Str<'backend>, BasicTypeEnum<'backend>>,
    functions: Map<Str<'backend>, FunctionValue<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn new(name: Str<'backend>, context: &'backend Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(&name);
        let printf_type = context.i32_type().fn_type(
            &[context.ptr_type(AddressSpace::default()).into()],
            true,
        );
        let printf = module.add_function("printf", printf_type, Some(inkwell::module::Linkage::External));
        let mut functions = Map::new();
        functions.insert(Str::from("printf"), printf);
        Self {
            context,
            builder,
            module,
            variables: Map::new(),
            types: Map::new(),
            functions,
        }
    }

    fn create_format(&self, format: &str) -> PointerValue<'backend> {
        let string = self.context.const_string(format.as_bytes(), true);
        let global = self.module.add_global(string.get_type(), Some(AddressSpace::default()), "format");
        global.set_initializer(&string);
        global.set_constant(true);
        self.builder.build_global_string_ptr(format, "format_ptr").unwrap().as_pointer_value()
    }
}

impl<'backend> Backend<'backend> for Inkwell<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        for analysis in &analyses {
            if let Instruction::Method(_) = analysis.instruction {
                self.generate_instruction(analysis.instruction.clone(), self.module.add_function("dummy", self.context.void_type().fn_type(&[], false), None));
            }
        }

        let function_type = self.context.i32_type().fn_type(&[], false);
        let function = self.module.add_function("main", function_type, None);
        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);

        let mut value = BasicValueEnum::from(self.context.i64_type().const_zero());

        for analysis in analyses {
            if !matches!(analysis.instruction, Instruction::Method(_)) {
                value = self.generate_instruction(analysis.instruction, function);
            }
        }

        let return_value = if value.is_int_value() {
            let integer = value.into_int_value();
            if integer.get_type().get_bit_width() != 32 {
                if integer.get_type().get_bit_width() > 32 {
                    self.builder.build_int_truncate(integer, self.context.i32_type(), "truncate").unwrap()
                } else {
                    self.builder.build_int_z_extend(integer, self.context.i32_type(), "extend").unwrap()
                }
            } else {
                integer
            }
        } else if value.is_float_value() {
            self.builder.build_float_to_signed_int(
                value.into_float_value(),
                self.context.i32_type(),
                "float_to_int"
            ).unwrap()
        } else {
            self.context.i32_type().const_zero()
        };

        self.builder.build_return(Some(&return_value));
    }

    fn generate_instruction(&mut self, instruction: Instruction<'backend>, function: FunctionValue<'backend>) -> BasicValueEnum<'backend> {
        match instruction {
            Instruction::Integer { value, size, signed } => self.generate_integer(value, size, signed),
            Instruction::Float { value, size } => self.generate_float(value, size),
            Instruction::Boolean { value } => self.generate_boolean(value),
            Instruction::Add(left, right) => self.generate_add(left, right, function),
            Instruction::Subtract(left, right) => self.generate_subtract(left, right, function),
            Instruction::Multiply(left, right) => self.generate_multiply(left, right, function),
            Instruction::Divide(left, right) => self.generate_divide(left, right, function),
            Instruction::Modulus(left, right) => self.generate_modulus(left, right, function),
            Instruction::LogicalAnd(left, right) => self.generate_logical_and(left, right, function),
            Instruction::LogicalOr(left, right) => self.generate_logical_or(left, right, function),
            Instruction::LogicalNot(operand) => self.generate_logical_not(operand, function),
            Instruction::BitwiseAnd(left, right) => self.generate_bitwise_and(left, right, function),
            Instruction::BitwiseOr(left, right) => self.generate_bitwise_or(left, right, function),
            Instruction::BitwiseNot(operand) => self.generate_bitwise_not(operand, function),
            Instruction::BitwiseXOr(left, right) => self.generate_bitwise_xor(left, right, function),
            Instruction::ShiftLeft(left, right) => self.generate_shift_left(left, right, function),
            Instruction::ShiftRight(left, right) => self.generate_shift_right(left, right, function),
            Instruction::Equal(left, right) => self.generate_equal(left, right, function),
            Instruction::NotEqual(left, right) => self.generate_not_equal(left, right, function),
            Instruction::Less(left, right) => self.generate_less(left, right, function),
            Instruction::LessOrEqual(left, right) => self.generate_less_or_equal(left, right, function),
            Instruction::Greater(left, right) => self.generate_greater(left, right, function),
            Instruction::GreaterOrEqual(left, right) => self.generate_greater_or_equal(left, right, function),
            Instruction::Usage(identifier) => self.generate_usage(identifier),
            Instruction::Assign(assign) => self.generate_assign(assign, function),
            Instruction::Binding(binding) => self.generate_binding(binding, function),
            Instruction::Module(name, analyses) => self.generate_module(name, analyses, function),
            Instruction::Method(method) => self.generate_method(method),
            Instruction::Invoke(invoke) => self.generate_invoke(invoke, function),
            Instruction::Return(value) => self.generate_return(value, function),
            _ => BasicValueEnum::from(self.context.i64_type().const_zero())
        }
    }

    fn print(&self) {
        let content = self.module.print_to_string();
        println!("{}", content.to_string());
    }

    fn write_to_file(&self, filename: &str) -> std::io::Result<()> {
        let content = self.module.print_to_string();
        let mut file = std::fs::File::create(filename)?;
        file.write_all(content.to_string().as_bytes())?;
        Ok(())
    }
}