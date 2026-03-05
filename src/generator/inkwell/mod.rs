mod aggregate;
mod arithmetic;
mod bitwise;
mod comparison;
mod functions;
mod logical;
mod primitives;
mod variables;

use {
    super::Backend,
    crate::{
        data::Str,
        generator::{ErrorKind, GenerateError},
        internal::{
            hash::Map,
            platform::{File, Write},
        },
        tracker::Span,
    },
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        types::{BasicTypeEnum, StructType},
        values::{BasicValueEnum, FunctionValue, PointerValue},
    },
};
use crate::analyzer::{Analysis, Instruction};
use crate::checker::TypeKind;

#[derive(Clone)]
pub enum Entity<'backend> {
    Variable {
        pointer: PointerValue<'backend>,
        kind: BasicTypeEnum<'backend>,
        pointee: Option<BasicTypeEnum<'backend>>,
        signed: Option<bool>,
    },
    Function(FunctionValue<'backend>),
}

pub struct Inkwell<'backend> {
    context: &'backend Context,
    builder: Builder<'backend>,
    module: Module<'backend>,
    entities: Map<Str<'backend>, Entity<'backend>>,
    structs: Map<Str<'backend>, StructType<'backend>>,
    struct_fields: Map<Str<'backend>, Vec<Str<'backend>>>,
    array_elements: Map<Str<'backend>, BasicTypeEnum<'backend>>,
    modules: crate::internal::hash::Set<Str<'backend>>,
    bootstrap: bool,
    errors: Vec<GenerateError<'backend>>,
    loop_headers: Vec<BasicBlock<'backend>>,
    loop_exits: Vec<BasicBlock<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub(crate) fn llvm_type_from_type_kind(
        &self,
        kind: &TypeKind<'backend>,
    ) -> BasicTypeEnum<'backend> {
        match kind {
            TypeKind::Integer { bits, .. } => match bits {
                8 => self.context.i8_type().into(),
                16 => self.context.i16_type().into(),
                32 => self.context.i32_type().into(),
                64 => self.context.i64_type().into(),
                _ => self.context.i64_type().into(),
            },
            TypeKind::Float { bits } => match bits {
                32 => self.context.f32_type().into(),
                64 => self.context.f64_type().into(),
                _ => self.context.f64_type().into(),
            },
            TypeKind::Boolean => self.context.bool_type().into(),
            TypeKind::Character => self.context.i8_type().into(),
            TypeKind::Pointer { .. } => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            TypeKind::Structure(structure) | TypeKind::Enumeration(structure) => self
                .structs
                .get(&structure.target)
                .map(|kind| (*kind).into())
                .unwrap_or_else(|| self.context.i64_type().into()),
            _ => self.context.i64_type().into(),
        }
    }

    pub fn new(name: Str<'backend>, context: &'backend Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(&name);

        Self {
            context,
            builder,
            module,
            entities: Default::default(),
            structs: Default::default(),
            struct_fields: Default::default(),
            array_elements: Default::default(),
            modules: Default::default(),
            bootstrap: false,
            errors: Vec::new(),
            loop_headers: Vec::new(),
            loop_exits: Vec::new(),
        }
    }

    pub fn with_bootstrap(mut self, bootstrap: bool) -> Self {
        self.bootstrap = bootstrap;
        self
    }

    fn name(instruction: &Instruction<'backend>) -> &'static str {
        match instruction {
            Instruction::Integer { .. } => "Integer",
            Instruction::Float { .. } => "Float",
            Instruction::Boolean { .. } => "Boolean",
            Instruction::String { .. } => "String",
            Instruction::Character { .. } => "Character",
            Instruction::Array(_) => "Array",
            Instruction::Tuple(_) => "Tuple",
            Instruction::Add(..) => "Add",
            Instruction::Subtract(..) => "Subtract",
            Instruction::Multiply(..) => "Multiply",
            Instruction::Divide(..) => "Divide",
            Instruction::Modulus(..) => "Modulus",
            Instruction::LogicalAnd(..) => "LogicalAnd",
            Instruction::LogicalOr(..) => "LogicalOr",
            Instruction::LogicalNot(..) => "LogicalNot",
            Instruction::LogicalXOr(..) => "LogicalXOr",
            Instruction::BitwiseAnd(..) => "BitwiseAnd",
            Instruction::BitwiseOr(..) => "BitwiseOr",
            Instruction::BitwiseNot(..) => "BitwiseNot",
            Instruction::BitwiseXOr(..) => "BitwiseXOr",
            Instruction::ShiftLeft(..) => "ShiftLeft",
            Instruction::ShiftRight(..) => "ShiftRight",
            Instruction::AddressOf(..) => "AddressOf",
            Instruction::Dereference(..) => "Dereference",
            Instruction::Equal(..) => "Equal",
            Instruction::NotEqual(..) => "NotEqual",
            Instruction::Less(..) => "Less",
            Instruction::LessOrEqual(..) => "LessOrEqual",
            Instruction::Greater(..) => "Greater",
            Instruction::GreaterOrEqual(..) => "GreaterOrEqual",
            Instruction::Index(_) => "Index",
            Instruction::Invoke(_) => "Invoke",
            Instruction::Block(_) => "Block",
            Instruction::Conditional(..) => "Conditional",
            Instruction::While(..) => "While",
            Instruction::Cycle(..) => "Cycle",
            Instruction::Return(_) => "Return",
            Instruction::Break(_) => "Break",
            Instruction::Continue(_) => "Continue",
            Instruction::Usage(_) => "Usage",
            Instruction::Access(..) => "Access",
            Instruction::Constructor(_) => "Constructor",
            Instruction::Assign(..) => "Assign",
            Instruction::Store(..) => "Store",
            Instruction::Binding(_) => "Binding",
            Instruction::Structure(_) => "Structure",
            Instruction::Enumeration(_) => "Enumeration",
            Instruction::Method(_) => "Method",
            Instruction::Module(_, _) => "Module",
        }
    }

    fn unsupported(&mut self, instruction: Instruction<'backend>) -> BasicValueEnum<'backend> {
        self.errors.push(GenerateError::new(
            ErrorKind::UnsupportedInstruction {
                instruction: Self::name(&instruction),
            },
            Span::void(),
        ));
        self.context.i64_type().const_zero().into()
    }

    pub(crate) fn infer_signedness(&self, analysis: &Analysis<'backend>) -> Option<bool> {
        match &analysis.instruction {
            Instruction::Integer { signed, .. } => Some(*signed),
            Instruction::Usage(identifier) => match self.entities.get(identifier) {
                Some(Entity::Variable { signed, .. }) => *signed,
                _ => None,
            },
            Instruction::Assign(_, value) => self.infer_signedness(value),
            Instruction::Binding(binding) => binding
                .value
                .as_ref()
                .and_then(|value| self.infer_signedness(value)),
            _ => None,
        }
    }

    pub(crate) fn build_entry_alloca(
        &mut self,
        function: FunctionValue<'backend>,
        kind: BasicTypeEnum<'backend>,
        name: &str,
    ) -> PointerValue<'backend> {
        let previous = self.builder.get_insert_block();
        let entry = function
            .get_first_basic_block()
            .unwrap_or_else(|| self.context.append_basic_block(function, "entry"));

        if let Some(first) = entry.get_first_instruction() {
            self.builder.position_before(&first);
        } else {
            self.builder.position_at_end(entry);
        }

        let allocation = self.builder.build_alloca(kind, name).unwrap();

        if let Some(block) = previous {
            self.builder.position_at_end(block);
        }

        allocation
    }
}

impl<'backend> Backend<'backend> for Inkwell<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        for analysis in &analyses {
            if let Instruction::Structure(structure) = &analysis.instruction {
                self.define_structure(structure.clone());
            }
        }

        for analysis in &analyses {
            if let Instruction::Method(_) = analysis.instruction {
                self.instruction(
                    analysis.instruction.clone(),
                    self.module.add_function(
                        "dummy",
                        self.context.void_type().fn_type(&[], false),
                        None,
                    ),
                );
            }
        }

        let function_type = self.context.i32_type().fn_type(&[], false);
        let function = self.module.add_function("main", function_type, None);
        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);

        for analysis in analyses {
            if self
                .builder
                .get_insert_block()
                .and_then(|block| block.get_terminator())
                .is_some()
            {
                break;
            }
            if !matches!(analysis.instruction, Instruction::Method(_)) {
                self.instruction(analysis.instruction, function);
            }
        }

        if self
            .builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_none()
        {
            self.builder
                .build_return(Some(&self.context.i32_type().const_zero()));
        }

        if self.bootstrap {
            self.emit_bootstrap_start(function);
        }

        let _ = self.module.verify();
    }

    fn instruction(
        &mut self,
        instruction: Instruction<'backend>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        match instruction {
            Instruction::Integer {
                value,
                size,
                signed,
            } => self.integer(value, size, signed),
            Instruction::Float { value, size } => self.float(value, size),
            Instruction::Boolean { value } => self.boolean(value),
            Instruction::String { value } => self.string(value),
            Instruction::Character { value } => self.character(value),
            Instruction::Array(values) => self.array(values, function),
            Instruction::Tuple(values) => self.tuple(values, function),
            Instruction::Add(left, right) => self.add(left, right, function),
            Instruction::Subtract(left, right) => self.subtract(left, right, function),
            Instruction::Multiply(left, right) => self.multiply(left, right, function),
            Instruction::Divide(left, right) => self.divide(left, right, function),
            Instruction::Modulus(left, right) => self.modulus(left, right, function),
            Instruction::LogicalAnd(left, right) => self.logical_and(left, right, function),
            Instruction::LogicalOr(left, right) => self.logical_or(left, right, function),
            Instruction::LogicalNot(operand) => self.logical_not(operand, function),
            Instruction::LogicalXOr(left, right) => self.logical_xor(left, right, function),
            Instruction::BitwiseAnd(left, right) => self.bitwise_and(left, right, function),
            Instruction::BitwiseOr(left, right) => self.bitwise_or(left, right, function),
            Instruction::BitwiseNot(operand) => self.bitwise_not(operand, function),
            Instruction::BitwiseXOr(left, right) => self.bitwise_xor(left, right, function),
            Instruction::ShiftLeft(left, right) => self.shift_left(left, right, function),
            Instruction::ShiftRight(left, right) => self.shift_right(left, right, function),
            Instruction::AddressOf(operand) => self.address_of(operand, function),
            Instruction::Dereference(operand) => self.dereference(operand, function),
            Instruction::Equal(left, right) => self.equal(left, right, function),
            Instruction::NotEqual(left, right) => self.not_equal(left, right, function),
            Instruction::Less(left, right) => self.less(left, right, function),
            Instruction::LessOrEqual(left, right) => self.less_or_equal(left, right, function),
            Instruction::Greater(left, right) => self.greater(left, right, function),
            Instruction::GreaterOrEqual(left, right) => {
                self.greater_or_equal(left, right, function)
            }
            Instruction::Index(index) => self.index(index, function),
            Instruction::Usage(identifier) => self.usage(identifier),
            Instruction::Access(target, member) => self.access(target, member, function),
            Instruction::Constructor(structure) => self.constructor(structure, function),
            Instruction::Assign(target, value) => self.assign(target, value, function),
            Instruction::Store(target, value) => self.store(target, value, function),
            Instruction::Binding(binding) => self.binding(binding, function),
            Instruction::Block(analyses) => self.block(analyses, function),
            Instruction::Conditional(condition, then, otherwise) => {
                self.conditional(condition, then, otherwise, function)
            }
            Instruction::While(condition, body) => self.r#while(condition, body, function),
            Instruction::Structure(structure) => self.define_structure(structure),
            Instruction::Module(name, analyses) => self.module(name, analyses, function),
            Instruction::Method(method) => self.method(method),
            Instruction::Invoke(invoke) => self.invoke(invoke, function),
            Instruction::Return(value) => self.r#return(value, function),
            Instruction::Break(value) => self.r#break(value, function),
            Instruction::Continue(value) => self.r#continue(value, function),
            Instruction::Cycle(condition, body) => self.cycle(condition, body, function),
            other => self.unsupported(other),
        }
    }

    fn print(&self) {
        let content = self.module.print_to_string();
        println!("{}", content.to_string());
    }

    fn write(&self, filename: &str) -> std::io::Result<()> {
        let content = self.module.print_to_string();
        let mut file = File::create(filename)?;
        file.write_all(content.to_string().as_bytes())?;
        Ok(())
    }

    fn take_errors(&mut self) -> Vec<GenerateError<'backend>> {
        core::mem::take(&mut self.errors)
    }
}
