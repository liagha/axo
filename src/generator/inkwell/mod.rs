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
use crate::analyzer::Analysis;
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
    errors: Vec<GenerateError<'backend>>,
    loop_headers: Vec<BasicBlock<'backend>>,
    loop_exits: Vec<BasicBlock<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn llvm_type(
        &self,
        kind: &TypeKind<'backend>,
    ) -> BasicTypeEnum<'backend> {
        match kind {
            TypeKind::Integer { bits, .. } => match bits {
                8 => self.context.i8_type().into(),
                16 => self.context.i16_type().into(),
                32 => self.context.i32_type().into(),
                64 => self.context.i64_type().into(),
                size => self.context.custom_width_int_type(*size as u32).into(),
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
            errors: Vec::new(),
            loop_headers: Vec::new(),
            loop_exits: Vec::new(),
        }
    }

    fn name(instruction: &Analysis<'backend>) -> &'static str {
        match instruction {
            Analysis::Integer { .. } => "Integer",
            Analysis::Float { .. } => "Float",
            Analysis::Boolean { .. } => "Boolean",
            Analysis::String { .. } => "String",
            Analysis::Character { .. } => "Character",
            Analysis::Array(_) => "Array",
            Analysis::Tuple(_) => "Tuple",
            Analysis::Add(..) => "Add",
            Analysis::Subtract(..) => "Subtract",
            Analysis::Multiply(..) => "Multiply",
            Analysis::Divide(..) => "Divide",
            Analysis::Modulus(..) => "Modulus",
            Analysis::LogicalAnd(..) => "LogicalAnd",
            Analysis::LogicalOr(..) => "LogicalOr",
            Analysis::LogicalNot(..) => "LogicalNot",
            Analysis::LogicalXOr(..) => "LogicalXOr",
            Analysis::BitwiseAnd(..) => "BitwiseAnd",
            Analysis::BitwiseOr(..) => "BitwiseOr",
            Analysis::BitwiseNot(..) => "BitwiseNot",
            Analysis::BitwiseXOr(..) => "BitwiseXOr",
            Analysis::ShiftLeft(..) => "ShiftLeft",
            Analysis::ShiftRight(..) => "ShiftRight",
            Analysis::AddressOf(..) => "AddressOf",
            Analysis::Dereference(..) => "Dereference",
            Analysis::Equal(..) => "Equal",
            Analysis::NotEqual(..) => "NotEqual",
            Analysis::Less(..) => "Less",
            Analysis::LessOrEqual(..) => "LessOrEqual",
            Analysis::Greater(..) => "Greater",
            Analysis::GreaterOrEqual(..) => "GreaterOrEqual",
            Analysis::Index(_) => "Index",
            Analysis::Invoke(_) => "Invoke",
            Analysis::Block(_) => "Block",
            Analysis::Conditional(..) => "Conditional",
            Analysis::While(..) => "While",
            Analysis::Cycle(..) => "Cycle",
            Analysis::Return(_) => "Return",
            Analysis::Break(_) => "Break",
            Analysis::Continue(_) => "Continue",
            Analysis::Usage(_) => "Usage",
            Analysis::Access(..) => "Access",
            Analysis::Constructor(_) => "Constructor",
            Analysis::Assign(..) => "Assign",
            Analysis::Store(..) => "Store",
            Analysis::Binding(_) => "Binding",
            Analysis::Structure(_) => "Structure",
            Analysis::Enumeration(_) => "Enumeration",
            Analysis::Method(_) => "Method",
            Analysis::Module(_, _) => "Module",
        }
    }

    fn unsupported(&mut self, instruction: Analysis<'backend>) -> BasicValueEnum<'backend> {
        self.errors.push(GenerateError::new(
            ErrorKind::UnsupportedAnalysis {
                instruction: Self::name(&instruction),
            },
            Span::void(),
        ));
        self.context.i64_type().const_zero().into()
    }

    pub fn infer_signedness(&self, analysis: &Analysis<'backend>) -> Option<bool> {
        match &analysis {
            Analysis::Integer { signed, .. } => Some(*signed),
            Analysis::Usage(identifier) => match self.entities.get(identifier) {
                Some(Entity::Variable { signed, .. }) => *signed,
                _ => None,
            },
            Analysis::Assign(_, value) => self.infer_signedness(value),
            Analysis::Binding(binding) => binding
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
            if let Analysis::Structure(structure) = &analysis {
                self.define_structure(structure.clone());
            }
        }

        for analysis in &analyses {
            if let Analysis::Method(_) = analysis {
                self.analysis(
                    analysis.clone(),
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
            if !matches!(analysis, Analysis::Method(_)) {
                self.analysis(analysis, function);
            }
        }

        if self
            .builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_none()
        {
            let _ = self.builder
                .build_return(Some(&self.context.i32_type().const_zero()));
        }

        let _ = self.module.verify();
    }

    fn analysis(
        &mut self,
        instruction: Analysis<'backend>,
        function: FunctionValue<'backend>,
    ) -> BasicValueEnum<'backend> {
        match instruction {
            Analysis::Integer {
                value,
                size,
                signed,
            } => self.integer(value, size, signed),
            Analysis::Float { value, size } => self.float(value, size),
            Analysis::Boolean { value } => self.boolean(value),
            Analysis::String { value } => self.string(value),
            Analysis::Character { value } => self.character(value),
            Analysis::Array(values) => self.array(values, function),
            Analysis::Tuple(values) => self.tuple(values, function),
            Analysis::Add(left, right) => self.add(left, right, function),
            Analysis::Subtract(left, right) => self.subtract(left, right, function),
            Analysis::Multiply(left, right) => self.multiply(left, right, function),
            Analysis::Divide(left, right) => self.divide(left, right, function),
            Analysis::Modulus(left, right) => self.modulus(left, right, function),
            Analysis::LogicalAnd(left, right) => self.logical_and(left, right, function),
            Analysis::LogicalOr(left, right) => self.logical_or(left, right, function),
            Analysis::LogicalNot(operand) => self.logical_not(operand, function),
            Analysis::LogicalXOr(left, right) => self.logical_xor(left, right, function),
            Analysis::BitwiseAnd(left, right) => self.bitwise_and(left, right, function),
            Analysis::BitwiseOr(left, right) => self.bitwise_or(left, right, function),
            Analysis::BitwiseNot(operand) => self.bitwise_not(operand, function),
            Analysis::BitwiseXOr(left, right) => self.bitwise_xor(left, right, function),
            Analysis::ShiftLeft(left, right) => self.shift_left(left, right, function),
            Analysis::ShiftRight(left, right) => self.shift_right(left, right, function),
            Analysis::AddressOf(operand) => self.address_of(operand, function),
            Analysis::Dereference(operand) => self.dereference(operand, function),
            Analysis::Equal(left, right) => self.equal(left, right, function),
            Analysis::NotEqual(left, right) => self.not_equal(left, right, function),
            Analysis::Less(left, right) => self.less(left, right, function),
            Analysis::LessOrEqual(left, right) => self.less_or_equal(left, right, function),
            Analysis::Greater(left, right) => self.greater(left, right, function),
            Analysis::GreaterOrEqual(left, right) => {
                self.greater_or_equal(left, right, function)
            }
            Analysis::Index(index) => self.index(index, function),
            Analysis::Usage(identifier) => self.usage(identifier),
            Analysis::Access(target, member) => self.access(target, member, function),
            Analysis::Constructor(structure) => self.constructor(structure, function),
            Analysis::Assign(target, value) => self.assign(target, value, function),
            Analysis::Store(target, value) => self.store(target, value, function),
            Analysis::Binding(binding) => self.binding(binding, function),
            Analysis::Block(analyses) => self.block(analyses, function),
            Analysis::Conditional(condition, then, otherwise) => {
                self.conditional(condition, then, otherwise, function)
            }
            Analysis::While(condition, body) => self.r#while(condition, body, function),
            Analysis::Structure(structure) => self.define_structure(structure),
            Analysis::Module(name, analyses) => self.module(name, analyses, function),
            Analysis::Method(method) => self.method(method),
            Analysis::Invoke(invoke) => self.invoke(invoke, function),
            Analysis::Return(value) => self.r#return(value, function),
            Analysis::Break(value) => self.r#break(value, function),
            Analysis::Continue(value) => self.r#continue(value, function),
            Analysis::Cycle(condition, body) => self.cycle(condition, body, function),
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
