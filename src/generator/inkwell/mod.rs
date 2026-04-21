mod arithmetic;
mod bitwise;
mod comparison;
mod composite;
mod error;
mod functions;
mod logical;
mod primitives;
mod variables;

pub use {
    error::*,
    inkwell::{
        context::{Context, ContextRef},
        targets::TargetMachine,
    }
};

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::Str,
        generator::{GenerateError},
        internal::hash::Map,
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        module::Module,
        types::{BasicType, BasicTypeEnum, StructType},
        values::{BasicValueEnum, FunctionValue, PointerValue},
    },
};

#[derive(Clone, Debug)]
pub enum Entity<'backend> {
    Variable {
        pointer: PointerValue<'backend>,
        typing: Type<'backend>,
    },
    Module,
    Structure {
        shape: StructType<'backend>,
        members: Vec<Str<'backend>>,
    },
    Union {
        shape: StructType<'backend>,
        members: Vec<(Str<'backend>, BasicTypeEnum<'backend>)>,
    },
    Function(FunctionValue<'backend>),
}

pub struct Generator<'backend> {
    pub context: ContextRef<'backend>,
    pub builder: Builder<'backend>,
    pub modules: Map<Str<'backend>, Module<'backend>>,
    pub current_module: Str<'backend>,

    entities: Map<Str<'backend>, Entity<'backend>>,
    pub errors: Vec<GenerateError<'backend>>,

    loop_headers: Vec<BasicBlock<'backend>>,
    loop_exits: Vec<BasicBlock<'backend>>,
    loop_results: Vec<Option<PointerValue<'backend>>>,
}

impl<'backend> Generator<'backend> {
    fn value_type(&self, typing: &Type<'backend>) -> Type<'backend> {
        match &typing.kind {
            TypeKind::Binding(binding) => binding
                .value
                .as_deref()
                .cloned()
                .or_else(|| binding.annotation.as_deref().cloned())
                .unwrap_or_else(|| Type::from(TypeKind::Unknown)),
            _ => typing.clone(),
        }
    }

    fn member_name(&self, typing: &Type<'backend>) -> Option<Str<'backend>> {
        match &typing.kind {
            TypeKind::Binding(binding) => Some(binding.target),
            TypeKind::Function(function) if !function.target.is_empty() => Some(function.target),
            TypeKind::Has(target) => self.member_name(target),
            _ => None,
        }
    }

    fn shape(&self, typing: &Type<'backend>) -> Vec<Str<'backend>> {
        match &self.value_type(typing).kind {
            TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => aggregate
                .members
                .iter()
                .filter_map(|member| self.member_name(member))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn field(&self, typing: &Type<'backend>, field: &Str<'backend>) -> Option<usize> {
        self.shape(typing).iter().position(|name| name == field)
    }

    fn member_type(
        &self,
        typing: &Type<'backend>,
        field: &Str<'backend>,
        span: Span,
    ) -> Result<Option<BasicTypeEnum<'backend>>, GenerateError<'backend>> {
        match &self.value_type(typing).kind {
            TypeKind::Pointer { target } => self.member_type(target, field, span),
            TypeKind::And(left, right) => Ok(self
                .member_type(left, field, span)?
                .or(self.member_type(right, field, span)?)),
            TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => {
                let member = aggregate
                    .members
                    .iter()
                    .find(|member| self.member_name(member).is_some_and(|name| &name == field));

                match member {
                    Some(member) => Ok(Some(self.to_basic_type(&self.value_type(member), span)?)),
                    None => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn function_target(&self, typing: &Type<'backend>) -> Option<Str<'backend>> {
        match &self.value_type(typing).kind {
            TypeKind::Function(function) if !function.target.is_empty() => Some(function.target),
            _ => None,
        }
    }

    fn callable(&self, typing: &Type<'backend>) -> Option<FunctionValue<'backend>> {
        let target = self.function_target(typing)?;

        self.get_entity(&target)
            .and_then(|entity| match entity {
                Entity::Function(function) => Some(*function),
                _ => None,
            })
            .or_else(|| self.current_module().get_function(target.as_str().unwrap_or_default()))
    }

    pub fn get_entity(&self, name: &Str<'backend>) -> Option<&Entity<'backend>> {
        self.entities.get(name)
    }

    pub fn insert_entity(&mut self, name: Str<'backend>, entity: Entity<'backend>) {
        self.entities.insert(name, entity);
    }

    pub fn clear_loops(&mut self) {
        self.loop_headers.clear();
        self.loop_exits.clear();
        self.loop_results.clear();
    }

    pub fn enter_loop(
        &mut self,
        header: BasicBlock<'backend>,
        exit: BasicBlock<'backend>,
        result: Option<PointerValue<'backend>>,
    ) {
        self.loop_headers.push(header);
        self.loop_exits.push(exit);
        self.loop_results.push(result);
    }

    pub fn exit_loop(&mut self) {
        self.loop_results.pop();
        self.loop_exits.pop();
        self.loop_headers.pop();
    }

    pub fn current_loop_header(&self) -> Option<BasicBlock<'backend>> {
        self.loop_headers.last().copied()
    }

    pub fn current_loop_exit(&self) -> Option<BasicBlock<'backend>> {
        self.loop_exits.last().copied()
    }

    pub fn current_loop_result(&self) -> Option<PointerValue<'backend>> {
        self.loop_results.last().copied().flatten()
    }

    pub fn update_entity(&mut self, name: &Str<'backend>, new_entity: Entity<'backend>) -> bool {
        if self.entities.contains_key(name) {
            self.entities.insert(name.clone(), new_entity);
            true
        } else {
            false
        }
    }

    pub fn find_entity<F>(&self, mut predicate: F) -> Option<&Entity<'backend>>
    where
        F: FnMut(&Entity<'backend>) -> bool,
    {
        for entity in self.entities.values() {
            if predicate(entity) {
                return Some(entity);
            }
        }

        None
    }

    pub fn has_module(&self, name: &Str<'backend>) -> bool {
        self.modules.contains_key(name)
    }

    pub fn raw_entity(&self, name: &Str<'backend>) -> Option<Entity<'backend>> {
        self.get_entity(name)
            .cloned()
            .or_else(|| self.has_module(name).then_some(Entity::Module))
    }

    pub fn symbol(&self, analysis: &Analysis<'backend>) -> Option<Str<'backend>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => Some(*name),
            AnalysisKind::Access(target, member) if self.namespace(target) => match &member.kind {
                AnalysisKind::Usage(name) => Some(*name),
                AnalysisKind::Access(_, _) => self.symbol(member),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn entity(&self, analysis: &Analysis<'backend>) -> Option<Entity<'backend>> {
        self.symbol(analysis)
            .and_then(|name| self.raw_entity(&name))
            .filter(|entity| !matches!(entity, Entity::Variable { .. }))
    }

    pub fn namespace(&self, analysis: &Analysis<'backend>) -> bool {
        matches!(
            self.entity(analysis),
            Some(Entity::Module | Entity::Structure { .. } | Entity::Union { .. })
        )
    }

    pub fn to_basic_type(
        &self,
        typing: &Type<'backend>,
        span: Span,
    ) -> Result<BasicTypeEnum<'backend>, GenerateError<'backend>> {
        let typing = self.value_type(typing);

        let typing = match &typing.kind {
            TypeKind::Integer { size: bits, .. } => match bits {
                1 => self.context.bool_type().into(),
                8 => self.context.i8_type().into(),
                16 => self.context.i16_type().into(),
                32 => self.context.i32_type().into(),
                64 => self.context.i64_type().into(),
                128 => self.context.i128_type().into(),
                size => self.context.custom_width_int_type(*size as u32).into(),
            },
            TypeKind::Float { size: bits } => match bits {
                16 => self.context.f16_type().into(),
                32 => self.context.f32_type().into(),
                64 => self.context.f64_type().into(),
                128 => self.context.f128_type().into(),
                _ => self.context.f64_type().into(),
            },
            TypeKind::Boolean => self.context.bool_type().into(),
            TypeKind::Character => self.context.i32_type().into(),
            TypeKind::String => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            TypeKind::Pointer { .. } => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            TypeKind::Array { member, size } => {
                let typing = self.to_basic_type(member, span.clone())?;
                typing.array_type(*size as u32).into()
            }
            TypeKind::Tuple { members } => {
                let mut typings = Vec::with_capacity(members.len());
                for member in &**members {
                    typings.push(self.to_basic_type(&member, span.clone())?);
                }
                self.context.struct_type(&typings, false).into()
            }
            TypeKind::Structure(structure) => {
                if let Some(typing) =
                    self.get_entity(&structure.target)
                        .and_then(|entity| match entity {
                            Entity::Structure {
                                shape: struct_type, ..
                            } => Some((*struct_type).into()),
                            Entity::Union {
                                shape: struct_type, ..
                            } => Some((*struct_type).into()),
                            _ => None,
                        })
                {
                    typing
                } else {
                    let name = structure.target.clone();

                    if &*name == "" {
                        let mut members = Vec::new();
                        for member in &structure.members {
                            members.push(self.to_basic_type(member, span.clone())?);
                        }
                        self.context.struct_type(&members, false).into()
                    } else {
                        let shape = self
                            .context
                            .get_struct_type(&name)
                            .unwrap_or_else(|| self.context.opaque_struct_type(&name));

                        if shape.is_opaque() {
                            let mut members = Vec::new();
                            for member in &structure.members {
                                members.push(self.to_basic_type(member, span.clone())?);
                            }
                            shape.set_body(&members, false);
                        }

                        shape.into()
                    }
                }
            }
            TypeKind::Union(union) => {
                if let Some(typing) = self
                    .get_entity(&union.target)
                    .and_then(|entity| match entity {
                        Entity::Union {
                            shape: structure, ..
                        } => Some((*structure).into()),
                        _ => None,
                    })
                {
                    typing
                } else {
                    let name = union.target.clone();

                    if &*name == "" {
                        let mut largest: Option<BasicTypeEnum> = None;
                        let mut maximum = 0;

                        for member in &union.members {
                            let typing = self.to_basic_type(member, span.clone())?;
                            let limit = self.size(typing);

                            if limit >= maximum || largest.is_none() {
                                maximum = limit;
                                largest = Some(typing);
                            }
                        }

                        if let Some(target) = largest {
                            self.context.struct_type(&[target], false).into()
                        } else {
                            self.context.struct_type(&[], false).into()
                        }
                    } else {
                        let shape = self
                            .context
                            .get_struct_type(&name)
                            .unwrap_or_else(|| self.context.opaque_struct_type(&name));

                        if shape.is_opaque() {
                            let mut largest: Option<BasicTypeEnum> = None;
                            let mut maximum = 0;

                            for member in &union.members {
                                let typing = self.to_basic_type(member, span.clone())?;
                                let limit = self.size(typing);

                                if limit >= maximum || largest.is_none() {
                                    maximum = limit;
                                    largest = Some(typing);
                                }
                            }

                            if let Some(target) = largest {
                                shape.set_body(&[target], false);
                            } else {
                                shape.set_body(&[], false);
                            }
                        }

                        shape.into()
                    }
                }
            }
            _ => {
                return Err(GenerateError::new(
                    ErrorKind::InvalidType(typing.clone()),
                    span,
                ));
            }
        };

        Ok(typing)
    }

    pub fn new(context: ContextRef<'backend>) -> Self {
        let builder = context.create_builder();

        inkwell::targets::Target::initialize_all(&inkwell::targets::InitializationConfig::default());

        Self {
            context,
            builder,
            current_module: Default::default(),
            entities: Default::default(),
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
        let temporary_builder = self.context.create_builder();

        let entry = function
            .get_first_basic_block()
            .unwrap_or_else(|| self.context.append_basic_block(function, "entry"));

        if let Some(first) = entry.get_first_instruction() {
            temporary_builder.position_before(&first);
        } else {
            temporary_builder.position_at_end(entry);
        }

        temporary_builder.build_alloca(kind, &*name).unwrap()
    }

    pub fn current_module(&self) -> &Module<'backend> {
        self.modules.get(&self.current_module).unwrap()
    }

    pub fn generate(&mut self, analyses: Vec<Analysis<'backend>>) {
        for analysis in &analyses {
            match &analysis.kind {
                AnalysisKind::Structure(structure) => {
                    if let Err(error) =
                        self.declare_structure(structure.clone(), analysis.span.clone())
                    {
                        self.errors.push(error);
                    }
                }
                AnalysisKind::Union(union) => {
                    if let Err(error) = self.declare_union(union.clone(), analysis.span.clone()) {
                        self.errors.push(error);
                    }
                }
                AnalysisKind::Function(function) => {
                    if let Err(error) =
                        self.declare_function(function.clone(), analysis.span.clone())
                    {
                        self.errors.push(error);
                    }
                }
                _ => {}
            }
        }

        let mut entry = None;

        for analysis in &analyses {
            match &analysis.kind {
                AnalysisKind::Structure(structure) => {
                    if let Err(error) =
                        self.define_structure(structure.clone(), analysis.span.clone())
                    {
                        self.errors.push(error);
                    }
                }
                AnalysisKind::Union(union) => {
                    if let Err(error) = self.define_union(union.clone(), analysis.span.clone()) {
                        self.errors.push(error);
                    }
                }
                AnalysisKind::Function(function) => {
                    if function.entry {
                        entry = Some((function, analysis.span.clone()));
                    } else {
                        self.builder.clear_insertion_position();
                        if let Err(error) =
                            self.define_function(function.clone(), analysis.span.clone())
                        {
                            self.errors.push(error);
                        }
                    }
                }
                AnalysisKind::Binding(_) => {
                    self.builder.clear_insertion_position();
                    if let Err(error) = self.analysis(analysis.clone()) {
                        self.errors.push(error);
                    }
                }
                _ => {
                    self.builder.clear_insertion_position();
                    if let Err(error) = self.analysis(analysis.clone()) {
                        self.errors.push(error);
                    }
                }
            }
        }

        if let Some((entry_func, span)) = entry {
            self.builder.clear_insertion_position();
            if let Err(error) = self.define_function(entry_func.clone(), span) {
                self.errors.push(error);
            }
        }

        if let Some(block) = self.builder.get_insert_block() {
            if block.get_terminator().is_none() {
                if self.errors.is_empty() {
                    self.errors.push(GenerateError::new(
                        ErrorKind::Function(FunctionError::MissingReturn),
                        Span::void(),
                    ));
                }
                _ = self.builder.build_unreachable();
            }
        }

        if self.errors.is_empty() {
            if let Err(error) = self.modules.get(&self.current_module).unwrap().verify() {
                self.errors.push(GenerateError::new(
                    ErrorKind::Verification(error.to_string()),
                    Span::void(),
                ))
            }
        }
    }

    pub fn analysis(
        &mut self,
        analysis: Analysis<'backend>,
    ) -> Result<BasicValueEnum<'backend>, GenerateError<'backend>> {
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
            } => Ok(self.integer(value, size, signed)),
            AnalysisKind::Float { value, size } => self.float(value, size, span),
            AnalysisKind::Boolean { value } => Ok(self.boolean(value)),
            AnalysisKind::Character { value } => Ok(self.character(value)),
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
            AnalysisKind::LessOrEqual(left, right) => {
                self.less_or_equal(left, right, span)
            }
            AnalysisKind::Greater(left, right) => self.greater(left, right, span),
            AnalysisKind::GreaterOrEqual(left, right) => {
                self.greater_or_equal(left, right, span)
            }
            AnalysisKind::Index(index) => self.index(index, span),
            AnalysisKind::Usage(identifier) => self.usage(identifier, span),
            AnalysisKind::Access(target, member) => self.access(target, member, span),
            AnalysisKind::Constructor(structure) => self.constructor(typing, structure, span),
            AnalysisKind::Assign(target, value) => self.assign(target, value, span),
            AnalysisKind::Store(target, value) => self.store(target, value, span),
            AnalysisKind::Binding(binding) => self.binding(binding, span),
            AnalysisKind::Block(analyses) => self.block(analyses, span),
            AnalysisKind::Conditional(condition, then, otherwise) => self.conditional(
                *condition,
                *then,
                otherwise.map(|value| *value),
                span,
                false,
            ),
            AnalysisKind::While(condition, body) => self.r#while(condition, body, span),
            AnalysisKind::Module(name, analyses) => self.module(name, analyses, span),
            AnalysisKind::Invoke(invoke) => self.invoke(invoke, span),
            AnalysisKind::Return(value) => self.r#return(value, span),
            AnalysisKind::Break(value) => self.r#break(value, span),
            AnalysisKind::Continue(value) => self.r#continue(value, span),
        }
    }
}
