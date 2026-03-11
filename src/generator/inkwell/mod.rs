mod composite;
mod arithmetic;
mod bitwise;
mod comparison;
mod functions;
mod logical;
mod primitives;
mod variables;
pub mod error;

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
use crate::generator::ErrorKind;
use crate::tracker::Span;

#[derive(Clone)]
pub enum Entity<'backend> {
    Variable {
        pointer: PointerValue<'backend>,
        kind: BasicTypeEnum<'backend>,
        pointee: Option<BasicTypeEnum<'backend>>,
        signed: Option<bool>,
    },
    Struct {
        structure: StructType<'backend>,
        fields: Vec<Str<'backend>>,
    },
    Function(FunctionValue<'backend>),
}

pub struct Inkwell<'backend> {
    pub context: ContextRef<'backend>,
    pub builder: Builder<'backend>,
    pub modules: Map<Str<'backend>, Module<'backend>>,
    pub current_module: Str<'backend>,

    entities: Vec<Map<Str<'backend>, Entity<'backend>>>,
    pub errors: Vec<GenerateError<'backend>>,

    loop_headers: Vec<BasicBlock<'backend>>,
    loop_exits: Vec<BasicBlock<'backend>>,
    loop_results: Vec<Option<PointerValue<'backend>>>,
}

impl<'backend> Inkwell<'backend> {
    pub fn get_entity(&self, name: &Str<'backend>) -> Option<&Entity<'backend>> {
        for scope in self.entities.iter().rev() {
            if let Some(entity) = scope.get(name) {
                return Some(entity);
            }
        }
        None
    }

    pub fn insert_entity(&mut self, name: Str<'backend>, entity: Entity<'backend>) {
        if let Some(scope) = self.entities.last_mut() {
            scope.insert(name, entity);
        }
    }

    pub fn to_basic_type(&self, ty: &Type<'backend>, span: Span<'backend>) -> Result<BasicTypeEnum<'backend>, GenerateError<'backend>> {
        let ty = match &ty.kind {
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
                self.context.i32_type().into()
            },
            TypeKind::Pointer { .. } => {
                self
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into()
            },
            TypeKind::Structure(structure) => {
                if let Some(ty) = self
                    .get_entity(&structure.target)
                    .and_then(
                        |entity| {
                            if let Entity::Struct { structure: struct_type, .. } = entity {
                                Some((*struct_type).into())
                            } else {
                                None
                            }
                        }
                    ) {
                    ty
                } else {
                    return Err(
                        GenerateError::new(
                            ErrorKind::InvalidType(ty.clone()),
                            span
                        )
                    )
                }
            },

            TypeKind::String => {
                self.context.ptr_type(inkwell::AddressSpace::default()).into()
            }

            _ => {
                return Err(
                    GenerateError::new(
                        ErrorKind::InvalidType(ty.clone()),
                        span
                    )
                );
            },
        };

        Ok(ty)
    }

    pub fn new(context: ContextRef<'backend>) -> Self {
        let builder = context.create_builder();

        inkwell::targets::Target::initialize_all(&inkwell::targets::InitializationConfig::default());

        Self {
            context,
            builder,
            current_module: Default::default(),
            entities: vec![Default::default()],
            modules: Default::default(),
            errors: Vec::new(),
            loop_headers: Vec::new(),
            loop_exits: Vec::new(),
            loop_results: Vec::new(),
        }
    }

    pub fn infer_signedness(&self, analysis: &Analysis<'backend>) -> Option<bool> {
        match &analysis.kind {
            AnalysisKind::Integer { signed, .. } => Some(*signed),
            AnalysisKind::Usage(identifier) => match self.get_entity(identifier) {
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
                if let Err(error) = self.structure(structure.clone(), analysis.span) {
                    self.errors.push(error);
                }
            }
        }

        for analysis in &analyses {
            if let AnalysisKind::Binding(_) = &analysis.kind {
                self.builder.clear_insertion_position();
                if let Err(error) = self.analysis(analysis.clone()) {
                    self.errors.push(error);
                }
            }
        }

        let mut entry = None;

        for analysis in &analyses {
            if let AnalysisKind::Function(function) = &analysis.kind {
                if function.entry {
                    entry = Some((function, analysis.span));
                } else {
                    self.builder.clear_insertion_position();
                    if let Err(error) = self.analysis(analysis.clone()) {
                        self.errors.push(error);
                    }
                }
            }
        }

        if let Some((entry_func, span)) = entry {
            self.builder.clear_insertion_position();
            if let Err(error) = self.function(entry_func.clone(), span) {
                self.errors.push(error);
            }
        }

        if let Some(block) = self.builder.get_insert_block() {
            if block.get_terminator().is_none() {
                if self.errors.is_empty() {
                    self.errors.push(
                        GenerateError::new(
                            ErrorKind::Function(error::FunctionError::MissingReturn),
                            Span::void()
                        )
                    );
                }
                let _ = self.builder.build_unreachable();
            }
        }

        if self.errors.is_empty() {
            if let Err(error) = self.modules.get(&self.current_module).unwrap().verify() {
                self.errors.push(
                    GenerateError::new(
                        ErrorKind::Verification(error.to_string()),
                        Span::void()
                    )
                )
            }
        }
    }

    fn analysis(&mut self, instruction: Analysis<'backend>) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
        match instruction.kind {
            AnalysisKind::Integer { value, size, signed, } => Ok(self.integer(value, size, signed)),
            AnalysisKind::Float { value, size } => self.float(value, size, instruction.span),
            AnalysisKind::Boolean { value } => Ok(self.boolean(value)),
            AnalysisKind::Character { value } => Ok(self.character(value)),
            AnalysisKind::String { value } => self.string(value, instruction.span),
            AnalysisKind::Array(values) => self.array(values, instruction.span),
            AnalysisKind::Tuple(values) => self.tuple(values, instruction.span),
            AnalysisKind::Cast(value, ty) => self.explicit_cast(value, ty, instruction.span),
            AnalysisKind::Negate(value) => self.negate(value, instruction.span),
            AnalysisKind::SizeOf(ty) => self.size_of(ty, instruction.span),
            AnalysisKind::Add(left, right) => self.add(left, right, instruction.span),
            AnalysisKind::Subtract(left, right) => self.subtract(left, right, instruction.span),
            AnalysisKind::Multiply(left, right) => self.multiply(left, right, instruction.span),
            AnalysisKind::Divide(left, right) => self.divide(left, right, instruction.span),
            AnalysisKind::Modulus(left, right) => self.modulus(left, right, instruction.span),
            AnalysisKind::LogicalAnd(left, right) => self.logical_and(left, right, instruction.span),
            AnalysisKind::LogicalOr(left, right) => self.logical_or(left, right, instruction.span),
            AnalysisKind::LogicalNot(operand) => self.logical_not(operand, instruction.span),
            AnalysisKind::LogicalXOr(left, right) => self.logical_xor(left, right, instruction.span),
            AnalysisKind::BitwiseAnd(left, right) => self.bitwise_and(left, right, instruction.span),
            AnalysisKind::BitwiseOr(left, right) => self.bitwise_or(left, right, instruction.span),
            AnalysisKind::BitwiseNot(operand) => self.bitwise_not(operand, instruction.span),
            AnalysisKind::BitwiseXOr(left, right) => self.bitwise_xor(left, right, instruction.span),
            AnalysisKind::ShiftLeft(left, right) => self.shift_left(left, right, instruction.span),
            AnalysisKind::ShiftRight(left, right) => self.shift_right(left, right, instruction.span),
            AnalysisKind::AddressOf(operand) => self.address_of(operand, instruction.span),
            AnalysisKind::Dereference(operand) => self.dereference(operand, instruction.span),
            AnalysisKind::Equal(left, right) => self.equal(left, right, instruction.span),
            AnalysisKind::NotEqual(left, right) => self.not_equal(left, right, instruction.span),
            AnalysisKind::Less(left, right) => self.less(left, right, instruction.span),
            AnalysisKind::LessOrEqual(left, right) => self.less_or_equal(left, right, instruction.span),
            AnalysisKind::Greater(left, right) => self.greater(left, right, instruction.span),
            AnalysisKind::GreaterOrEqual(left, right) => self.greater_or_equal(left, right, instruction.span),
            AnalysisKind::Index(index) => self.index(index, instruction.span),
            AnalysisKind::Usage(identifier) => self.usage(identifier, instruction.span),
            AnalysisKind::Access(target, member) => self.access(target, member, instruction.span),
            AnalysisKind::Constructor(structure) => self.constructor(structure, instruction.span),
            AnalysisKind::Assign(target, value) => self.assign(target, value, instruction.span),
            AnalysisKind::Store(target, value) => self.store(target, value, instruction.span),
            AnalysisKind::Binding(binding) => self.binding(binding, instruction.span),
            AnalysisKind::Block(analyses) => self.block(analyses, instruction.span),
            AnalysisKind::Conditional(condition, then, otherwise) => self.conditional(condition, then, otherwise, instruction.span),
            AnalysisKind::While(condition, body) => self.r#while(condition, body, instruction.span),
            AnalysisKind::Structure(structure) => self.structure(structure, instruction.span),
            AnalysisKind::Module(name, analyses) => self.module(name, analyses, instruction.span),
            AnalysisKind::Function(function) => self.function(function, instruction.span),
            AnalysisKind::Invoke(invoke) => self.invoke(invoke, instruction.span),
            AnalysisKind::Return(value) => self.r#return(value, instruction.span),
            AnalysisKind::Break(value) => self.r#break(value, instruction.span),
            AnalysisKind::Continue(value) => self.r#continue(value, instruction.span),
        }
    }
}
