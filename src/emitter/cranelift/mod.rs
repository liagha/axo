// src/generator/cranelift/mod.rs
mod arithmetic;
pub mod evaluate;
mod bitwise;
mod comparison;
mod composite;
mod error;
mod functions;
mod logical;
mod primitives;
mod variables;

pub use {error::*, evaluate::{Engine, Value as EvalValue}};

use {
    crate::{
        analyzer::{Analysis, AnalysisKind, Target},
        data::Str,
        generator::GenerateError,
        internal::hash::Map,
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    cranelift_codegen::ir::{Block, Type as IrType, Value},
    cranelift_frontend::{FunctionBuilder, Variable},
};

#[derive(Clone, Debug)]
pub enum Entity<'backend> {
    Variable {
        pointer: Variable,
        typing: Type<'backend>,
    },
    Module,
    Structure {
        members: Vec<Str<'backend>>,
    },
    Union {
        members: Vec<(Str<'backend>, IrType)>,
    },
    Function,
}

pub struct CraneliftGenerator<'backend> {
    pub builder: FunctionBuilder<'backend>,
    pub entities: Map<Str<'backend>, Entity<'backend>>,
    pub errors: Vec<GenerateError<'backend>>,

    loop_headers: Vec<Block>,
    loop_exits: Vec<Block>,
    loop_results: Vec<Option<Variable>>,
}

impl<'backend> CraneliftGenerator<'backend> {
    pub fn new() -> Self {
        unimplemented!("Cranelift generator initialization requires builder context.")
    }

    pub fn finish(&mut self) -> Vec<u8> {
        Vec::new()
    }

    pub fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        for analysis in analyses {
            if let Err(error) = self.analysis(analysis) {
                self.errors.push(error);
            }
        }
    }

    pub fn analysis(
        &mut self,
        analysis: Analysis<'backend>,
    ) -> Result<Value, GenerateError<'backend>> {
        let span = analysis.span;
        let typing = analysis.typing.clone();

        match analysis.kind {
            AnalysisKind::Structure(structure) => self.define_structure(structure, span),
            AnalysisKind::Union(structure) => self.define_union(structure, span),
            AnalysisKind::Function(function) => self.define_function(function, span),

            AnalysisKind::Integer {
                value,
                size,
                signed,
            } => self.integer(value, size, signed),
            AnalysisKind::Float { value, size } => self.float(value, size, span),
            AnalysisKind::Boolean { value } => self.boolean(value),
            AnalysisKind::Character { value } => self.character(value),
            AnalysisKind::String { value } => self.string(value, span),
            AnalysisKind::Array(values) => self.array(values, span),
            AnalysisKind::Tuple(values) => self.tuple(values, span),
            AnalysisKind::Negate(value) => self.negate(value, span),
            AnalysisKind::SizeOf(typing) => self.size_of(typing, span),
            AnalysisKind::Add(left, right) => self.add(left, right, span),
            AnalysisKind::Subtract(left, right) => self.subtract(left, right, span),
            AnalysisKind::Multiply(left, right) => self.multiply(left, right, span),
            AnalysisKind::Divide(left, right) => self.divide(left, right, span),
            AnalysisKind::Modulus(left, right) => self.modulus(left, right, span),
            AnalysisKind::LogicalAnd(left, right) => self.logical_and(left, right, span),
            AnalysisKind::LogicalOr(left, right) => self.logical_or(left, right, span),
            AnalysisKind::LogicalNot(operand) => self.logical_not(operand, span),
            AnalysisKind::LogicalXOr(left, right) => self.logical_xor(left, right, span),
            AnalysisKind::BitwiseAnd(left, right) => self.bitwise_and(left, right, span),
            AnalysisKind::BitwiseOr(left, right) => self.bitwise_or(left, right, span),
            AnalysisKind::BitwiseNot(operand) => self.bitwise_not(operand, span),
            AnalysisKind::BitwiseXOr(left, right) => self.bitwise_xor(left, right, span),
            AnalysisKind::ShiftLeft(left, right) => self.shift_left(left, right, span),
            AnalysisKind::ShiftRight(left, right) => self.shift_right(left, right, span),
            AnalysisKind::AddressOf(operand) => self.address_of(operand, span),
            AnalysisKind::Dereference(operand) => self.dereference(operand, span),
            AnalysisKind::Equal(left, right) => self.equal(left, right, span),
            AnalysisKind::NotEqual(left, right) => self.not_equal(left, right, span),
            AnalysisKind::Less(left, right) => self.less(left, right, span),
            AnalysisKind::LessOrEqual(left, right) => self.less_or_equal(left, right, span),
            AnalysisKind::Greater(left, right) => self.greater(left, right, span),
            AnalysisKind::GreaterOrEqual(left, right) => self.greater_or_equal(left, right, span),
            AnalysisKind::Index(index) => self.index(index, span),
            AnalysisKind::Usage(identifier) => self.usage(identifier, span),
            AnalysisKind::Symbol(target) => self.symbol_value(target, span),
            AnalysisKind::Access(target, member) => self.access(target, member, span),
            AnalysisKind::Slot(target, slot) => self.slot(target, slot, span),
            AnalysisKind::Constructor(structure) => self.constructor(typing, structure, span),
            AnalysisKind::Pack(target, values) => self.pack(typing, target, values, span),
            AnalysisKind::Assign(target, value) => self.assign(target, value, span),
            AnalysisKind::Write(target, value) => self.write(target, value, span),
            AnalysisKind::Store(target, value) => self.store(target, value, span),
            AnalysisKind::Binding(binding) => self.binding(binding, span),
            AnalysisKind::Block(analyses) => self.block(analyses, span),
            AnalysisKind::Conditional(condition, then, otherwise) => {
                self.conditional(*condition, *then, otherwise.map(|val| *val), span, false)
            }
            AnalysisKind::While(condition, body) => self.r#while(condition, body, span),
            AnalysisKind::Module(name, analyses) => self.module(name, analyses, span),
            AnalysisKind::Invoke(invoke) => self.invoke(invoke, span),
            AnalysisKind::Call(target, values) => self.call(target, values, span),
            AnalysisKind::Return(value) => self.r#return(value, span),
            AnalysisKind::Break(value) => self.r#break(value, span),
            AnalysisKind::Continue(value) => self.r#continue(value, span),
            AnalysisKind::Composite(composite) => self.composite(composite, span),
        }
    }

    pub fn infer_signedness(&self, analysis: &Analysis<'backend>) -> Option<bool> {
        match &analysis.kind {
            AnalysisKind::Integer { signed, .. } => Some(*signed),
            AnalysisKind::Usage(identifier) => match self.entities.get(identifier) {
                Some(Entity::Variable { typing, .. }) => {
                    if let TypeKind::Integer { signed, .. } = &typing.kind {
                        Some(*signed)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            AnalysisKind::Assign(_, value) => self.infer_signedness(value),
            AnalysisKind::Write(_, value) => self.infer_signedness(value),
            AnalysisKind::Binding(binding) => binding
                .value
                .as_ref()
                .and_then(|value| self.infer_signedness(value)),
            _ => None,
        }
    }
}
