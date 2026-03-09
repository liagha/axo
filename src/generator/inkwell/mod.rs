mod composite;
mod arithmetic;
mod bitwise;
mod comparison;
mod functions;
mod logical;
mod primitives;
mod variables;

use crate::analyzer::Analysis;
use crate::checker::TypeKind;
use crate::internal::hash::Set;
use {
    super::Backend,
    crate::{data::Str, generator::GenerateError, internal::hash::Map},
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::ContextRef,
        module::Module,
        types::{BasicTypeEnum, StructType},
        values::{BasicValueEnum, FunctionValue, PointerValue},
    },
};

#[derive(Clone)]
pub enum Entity<'backend> {
    Variable {
        pointer: PointerValue<'backend>,
        kind: BasicTypeEnum<'backend>,
        pointee: Option<BasicTypeEnum<'backend>>,
        signed: Option<bool>,
    },
    Array {
        pointer: PointerValue<'backend>,
        element_type: BasicTypeEnum<'backend>,
    },
    Struct {
        struct_type: StructType<'backend>,
        fields: Vec<Str<'backend>>,
    },
    Function(FunctionValue<'backend>),
}

pub struct Inkwell<'backend> {
    context: ContextRef<'backend>,
    builder: Builder<'backend>,
    pub module: Module<'backend>,

    entities: Map<Str<'backend>, Entity<'backend>>,
    modules: Set<Str<'backend>>,
    pub errors: Vec<GenerateError<'backend>>,

    loop_headers: Vec<BasicBlock<'backend>>,
    loop_exits: Vec<BasicBlock<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn llvm_type(&self, kind: &TypeKind<'backend>) -> BasicTypeEnum<'backend> {
        match kind {
            TypeKind::Integer { bits, .. } => {
                match bits {
                    8 => self.context.i8_type().into(),
                    16 => self.context.i16_type().into(),
                    32 => self.context.i32_type().into(),
                    64 => self.context.i64_type().into(),
                    size => self.context.custom_width_int_type(*size as u32).into(),
                }
            },
            TypeKind::Float { bits } => {
                match bits {
                    32 => self.context.f32_type().into(),
                    64 => self.context.f64_type().into(),
                    _ => self.context.f64_type().into(),
                }
            },
            TypeKind::Boolean => {
                self.context.bool_type().into()
            },
            TypeKind::Character => {
                self.context.i8_type().into()
            },
            TypeKind::Pointer { .. } => {
                self
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into()
            },
            TypeKind::Structure(structure) | TypeKind::Enumeration(structure) => {
                self
                    .entities
                    .get(&structure.target)
                    .and_then(
                        |entity| {
                            if let Entity::Struct { struct_type, .. } = entity {
                                Some((*struct_type).into())
                            } else {
                                None
                            }
                        }
                    )
                    .unwrap_or_else(|| self.context.i64_type().into())
            },
            _ => {
                self.context.i64_type().into()
            },
        }
    }

    pub fn new(name: Str<'backend>, context: ContextRef<'backend>) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(&name);

        Self {
            context,
            builder,
            module,
            entities: Default::default(),
            modules: Default::default(),
            errors: Vec::new(),
            loop_headers: Vec::new(),
            loop_exits: Vec::new(),
        }
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

    pub fn build_entry(
        &mut self,
        function: FunctionValue<'backend>,
        kind: BasicTypeEnum<'backend>,
        name: Str<'backend>,
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

        let allocation = self.builder.build_alloca(kind, &*name).unwrap();

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

        let mut entry = None;

        for analysis in &analyses {
            if let Analysis::Method(method) = analysis {
                if method.entry {
                    entry = Some(method);
                } else {
                    self.analysis(analysis.clone());
                }
            }
        }

        if let Some(entry) = entry {
            self.method(entry.clone());
        }

        if self
            .builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_none()
        {
            let _ = self
                .builder
                .build_return(Some(&self.context.i32_type().const_zero()));
        }

        let _ = self.module.verify();
    }

    fn analysis(&mut self, instruction: Analysis<'backend>) -> BasicValueEnum<'backend> {
        match instruction {
            Analysis::Integer { value, size, signed, } => self.integer(value, size, signed),
            Analysis::Float { value, size } => self.float(value, size),
            Analysis::Boolean { value } => self.boolean(value),
            Analysis::Character { value } => self.character(value),
            Analysis::String { value } => self.string(value),
            Analysis::Array(values) => self.array(values).0.into(),
            Analysis::Tuple(values) => self.tuple(values),
            Analysis::Add(left, right) => self.add(left, right),
            Analysis::Subtract(left, right) => self.subtract(left, right),
            Analysis::Multiply(left, right) => self.multiply(left, right),
            Analysis::Divide(left, right) => self.divide(left, right),
            Analysis::Modulus(left, right) => self.modulus(left, right),
            Analysis::LogicalAnd(left, right) => self.logical_and(left, right),
            Analysis::LogicalOr(left, right) => self.logical_or(left, right),
            Analysis::LogicalNot(operand) => self.logical_not(operand),
            Analysis::LogicalXOr(left, right) => self.logical_xor(left, right),
            Analysis::BitwiseAnd(left, right) => self.bitwise_and(left, right),
            Analysis::BitwiseOr(left, right) => self.bitwise_or(left, right),
            Analysis::BitwiseNot(operand) => self.bitwise_not(operand),
            Analysis::BitwiseXOr(left, right) => self.bitwise_xor(left, right),
            Analysis::ShiftLeft(left, right) => self.shift_left(left, right),
            Analysis::ShiftRight(left, right) => self.shift_right(left, right),
            Analysis::AddressOf(operand) => self.address_of(operand),
            Analysis::Dereference(operand) => self.dereference(operand),
            Analysis::Equal(left, right) => self.equal(left, right),
            Analysis::NotEqual(left, right) => self.not_equal(left, right),
            Analysis::Less(left, right) => self.less(left, right),
            Analysis::LessOrEqual(left, right) => self.less_or_equal(left, right),
            Analysis::Greater(left, right) => self.greater(left, right),
            Analysis::GreaterOrEqual(left, right) => self.greater_or_equal(left, right),
            Analysis::Index(index) => self.index(index),
            Analysis::Usage(identifier) => self.usage(identifier),
            Analysis::Access(target, member) => self.access(target, member),
            Analysis::Constructor(structure) => self.constructor(structure),
            Analysis::Assign(target, value) => self.assign(target, value),
            Analysis::Store(target, value) => self.store(target, value),
            Analysis::Binding(binding) => self.binding(binding),
            Analysis::Block(analyses) => self.block(analyses),
            Analysis::Conditional(condition, then, otherwise) => self.conditional(condition, then, otherwise),
            Analysis::While(condition, body) => self.r#while(condition, body),
            Analysis::Structure(structure) => self.define_structure(structure),
            Analysis::Module(name, analyses) => self.module(name, analyses),
            Analysis::Method(method) => self.method(method),
            Analysis::Invoke(invoke) => self.invoke(invoke),
            Analysis::Return(value) => self.r#return(value),
            Analysis::Break(value) => self.r#break(value),
            Analysis::Continue(value) => self.r#continue(value),
            Analysis::Enumeration(_) => {
                unimplemented!("")
            }
        }
    }
}
