mod composite;
mod arithmetic;
mod bitwise;
mod comparison;
mod functions;
mod logical;
mod primitives;
mod variables;

use crate::analyzer::{Analysis, AnalysisKind};
use crate::checker::{Type, TypeKind};
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
        element_count: usize, 
    },
    Struct {
        struct_type: StructType<'backend>,
        fields: Vec<Str<'backend>>,
    },
    Function(FunctionValue<'backend>),
}

pub struct Inkwell<'backend> {
    pub context: ContextRef<'backend>,
    pub builder: Builder<'backend>,
    pub modules: Map<Str<'backend>, Module<'backend>>,
    pub current_module: Str<'backend>,

    entities: Map<Str<'backend>, Entity<'backend>>,
    pub errors: Vec<GenerateError<'backend>>,

    loop_headers: Vec<BasicBlock<'backend>>,
    loop_exits: Vec<BasicBlock<'backend>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn llvm_type(&self, ty: &Type<'backend>) -> BasicTypeEnum<'backend> {
        match &ty.kind {
            TypeKind::Integer { size: bits, .. } => {
                match bits {
                    8 => self.context.i8_type().into(),
                    16 => self.context.i16_type().into(),
                    32 => self.context.i32_type().into(),
                    64 => self.context.i64_type().into(),
                    size => self.context.custom_width_int_type(*size as u32).into(),
                }
            },
            TypeKind::Float { size: bits } => {
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
            TypeKind::Structure(structure) => {
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

    pub fn new(context: ContextRef<'backend>) -> Self {
        let builder = context.create_builder();

        Self {
            context,
            builder,
            current_module: Default::default(),
            entities: Default::default(),
            modules: Default::default(),
            errors: Vec::new(),
            loop_headers: Vec::new(),
            loop_exits: Vec::new(),
        }
    }

    pub fn infer_signedness(&self, analysis: &Analysis<'backend>) -> Option<bool> {
        match &analysis.kind {
            AnalysisKind::Integer { signed, .. } => Some(*signed),
            AnalysisKind::Usage(identifier) => match self.entities.get(identifier) {
                Some(Entity::Variable { signed, .. }) => *signed,
                _ => None,
            },
            AnalysisKind::Assign(_, value) => self.infer_signedness(value),
            AnalysisKind::Binding(binding) => binding
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

    pub fn current_module(&self) -> &Module<'backend> {
        self
            .modules
            .get(&self.current_module)
            .unwrap()
    }
}

impl<'backend> Backend<'backend> for Inkwell<'backend> {
    fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        for analysis in &analyses {
            if let AnalysisKind::Structure(structure) = &analysis.kind {
                self.structure(structure.clone());
            }
        }

        let mut entry = None;

        for analysis in &analyses {
            if let AnalysisKind::Function(function) = &analysis.kind {
                if function.entry {
                    entry = Some(function);
                } else {
                    self.analysis(analysis.clone());
                }
            }
        }

        if let Some(entry) = entry {
            self.function(entry.clone());
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

        let _ = self.modules.get(&self.current_module).unwrap().verify();
    }

    fn analysis(&mut self, instruction: Analysis<'backend>) -> BasicValueEnum<'backend> {
        match instruction.kind {
            AnalysisKind::Integer { value, size, signed, } => self.integer(value, size, signed),
            AnalysisKind::Float { value, size } => self.float(value, size),
            AnalysisKind::Boolean { value } => self.boolean(value),
            AnalysisKind::Character { value } => self.character(value),
            AnalysisKind::String { value } => self.string(value),
            AnalysisKind::Array(values) => self.array(values).0.into(),
            AnalysisKind::Tuple(values) => self.tuple(values),
            AnalysisKind::Add(left, right) => self.add(left, right),
            AnalysisKind::Subtract(left, right) => self.subtract(left, right),
            AnalysisKind::Multiply(left, right) => self.multiply(left, right),
            AnalysisKind::Divide(left, right) => self.divide(left, right),
            AnalysisKind::Modulus(left, right) => self.modulus(left, right),
            AnalysisKind::LogicalAnd(left, right) => self.logical_and(left, right),
            AnalysisKind::LogicalOr(left, right) => self.logical_or(left, right),
            AnalysisKind::LogicalNot(operand) => self.logical_not(operand),
            AnalysisKind::LogicalXOr(left, right) => self.logical_xor(left, right),
            AnalysisKind::BitwiseAnd(left, right) => self.bitwise_and(left, right),
            AnalysisKind::BitwiseOr(left, right) => self.bitwise_or(left, right),
            AnalysisKind::BitwiseNot(operand) => self.bitwise_not(operand),
            AnalysisKind::BitwiseXOr(left, right) => self.bitwise_xor(left, right),
            AnalysisKind::ShiftLeft(left, right) => self.shift_left(left, right),
            AnalysisKind::ShiftRight(left, right) => self.shift_right(left, right),
            AnalysisKind::AddressOf(operand) => self.address_of(operand),
            AnalysisKind::Dereference(operand) => self.dereference(operand),
            AnalysisKind::Equal(left, right) => self.equal(left, right),
            AnalysisKind::NotEqual(left, right) => self.not_equal(left, right),
            AnalysisKind::Less(left, right) => self.less(left, right),
            AnalysisKind::LessOrEqual(left, right) => self.less_or_equal(left, right),
            AnalysisKind::Greater(left, right) => self.greater(left, right),
            AnalysisKind::GreaterOrEqual(left, right) => self.greater_or_equal(left, right),
            AnalysisKind::Index(index) => self.index(index),
            AnalysisKind::Usage(identifier) => self.usage(identifier),
            AnalysisKind::Access(target, member) => self.access(target, member),
            AnalysisKind::Constructor(structure) => self.constructor(structure),
            AnalysisKind::Assign(target, value) => self.assign(target, value),
            AnalysisKind::Store(target, value) => self.store(target, value),
            AnalysisKind::Binding(binding) => self.binding(binding),
            AnalysisKind::Block(analyses) => self.block(analyses),
            AnalysisKind::Conditional(condition, then, otherwise) => self.conditional(condition, then, otherwise),
            AnalysisKind::While(condition, body) => self.r#while(condition, body),
            AnalysisKind::Structure(structure) => self.structure(structure),
            AnalysisKind::Module(name, analyses) => self.module(name, analyses),
            AnalysisKind::Function(function) => self.function(function),
            AnalysisKind::Invoke(invoke) => self.invoke(invoke),
            AnalysisKind::Return(value) => self.r#return(value),
            AnalysisKind::Break(value) => self.r#break(value),
            AnalysisKind::Continue(value) => self.r#continue(value),
        }
    }
}
