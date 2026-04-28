mod error;
pub mod evaluate;

pub use {
    error::*,
    evaluate::{Engine, Value as EvalValue},
};

use {
    crate::{
        analyzer::{Analysis, AnalysisKind, Target},
        data::{Aggregate, Binding, BindingKind, Function, Index, Interface, Scale, Str},
        generator::{
            ControlFlowError, DataStructureError, ErrorKind, FunctionError, GenerateError,
            VariableError,
        },
        internal::hash::Map,
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    cranelift_codegen::{
        ir::{
            condcodes::{FloatCC, IntCC},
            types, AbiParam, Block, Function as IrFunction, InstBuilder, MemFlags, Signature,
            StackSlot, StackSlotData, StackSlotKind, Type as IrType, UserFuncName, Value,
        },
        isa, settings,
    },
    cranelift_frontend::{FunctionBuilder, FunctionBuilderContext},
    cranelift_module::{default_libcall_names, FuncId, Linkage, Module},
    cranelift_object::{ObjectBuilder, ObjectModule},
    std::str::FromStr,
    target_lexicon::Triple,
};

#[derive(Clone, Debug)]
pub enum Entity<'a> {
    Variable { slot: StackSlot, typing: Type<'a> },
    Structure { members: Vec<Str<'a>> },
    Union { members: Vec<Str<'a>> },
    Function(FuncData<'a>),
    Module,
}

#[derive(Clone, Debug)]
pub struct FuncData<'a> {
    pub id: FuncId,
    pub sig: Signature,
    pub output: Option<Type<'a>>,
    pub indirect: bool,
    pub variadic: bool,
    pub entry: bool,
    pub interface: Interface,
}

#[derive(Clone, Copy, Debug)]
struct Layout {
    size: u32,
    align: u8,
}

#[derive(Clone, Copy)]
struct Loop {
    head: Block,
    exit: Block,
    slot: Option<StackSlot>,
}

struct Lower<'a, 'b, M: Module> {
    module: &'a mut M,
    builder: FunctionBuilder<'a>,
    pointer: IrType,
    entities: Map<Str<'b>, Entity<'b>>,
    func: FuncData<'b>,
    loops: Vec<Loop>,
    ret: Option<Value>,
}

pub(crate) fn lower<'a, M: Module>(
    module: &mut M,
    analyses: Vec<Analysis<'a>>,
) -> Result<Map<Str<'a>, Entity<'a>>, Vec<GenerateError<'a>>> {
    let pointer = module.target_config().pointer_type();
    let mut entities = Map::new();
    let mut errors = Vec::new();

    for analysis in &analyses {
        match &analysis.kind {
            AnalysisKind::Structure(value) => {
                entities.insert(
                    value.target,
                    Entity::Structure {
                        members: member_names(value),
                    },
                );
            }
            AnalysisKind::Union(value) => {
                entities.insert(
                    value.target,
                    Entity::Union {
                        members: member_names(value),
                    },
                );
            }
            AnalysisKind::Function(value) => match declare_function(module, pointer, value) {
                Ok(data) => {
                    entities.insert(value.target, Entity::Function(data));
                }
                Err(error) => errors.push(GenerateError::new(
                    ErrorKind::Verification(error),
                    analysis.span,
                )),
            },
            AnalysisKind::Module(name, _) => {
                entities.insert(*name, Entity::Module);
            }
            _ => {}
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut entry = None;

    for analysis in analyses {
        match analysis.kind {
            AnalysisKind::Function(value) => {
                if value.entry {
                    entry = Some((value, analysis.span));
                } else if let Err(error) = define_function(module, pointer, &entities, value, analysis.span) {
                    errors.push(error);
                }
            }
            AnalysisKind::Binding(value) => {
                if value.kind == BindingKind::Static {
                    if let Err(error) = define_static(module, pointer, &entities, value, analysis.span) {
                        errors.push(error);
                    }
                }
            }
            AnalysisKind::Structure(_) | AnalysisKind::Union(_) | AnalysisKind::Module(_, _) => {}
            _ => {
                errors.push(GenerateError::new(
                    ErrorKind::Verification(
                        "top-level Cranelift expressions are not supported".to_string(),
                    ),
                    analysis.span,
                ));
            }
        }
    }

    if let Some((value, span)) = entry {
        if let Err(error) = define_function(module, pointer, &entities, value, span) {
            errors.push(error);
        }
    }

    if errors.is_empty() {
        Ok(entities)
    } else {
        Err(errors)
    }
}

pub fn compile<'a>(
    analyses: Vec<Analysis<'a>>,
    stem: &str,
    target: Option<&str>,
) -> Result<Vec<u8>, Vec<GenerateError<'a>>> {
    let isa = match build_isa(target) {
        Ok(isa) => isa,
        Err(error) => {
            return Err(vec![GenerateError::new(
                ErrorKind::Verification(error),
                Span::void(),
            )])
        }
    };

    let builder = match ObjectBuilder::new(isa, stem, default_libcall_names()) {
        Ok(builder) => builder,
        Err(error) => {
            return Err(vec![GenerateError::new(
                ErrorKind::Verification(error.to_string()),
                Span::void(),
            )])
        }
    };

    let mut module = ObjectModule::new(builder);
    lower(&mut module, analyses)?;

    match module.finish().emit() {
        Ok(bytes) => Ok(bytes),
        Err(error) => Err(vec![GenerateError::new(
            ErrorKind::Verification(error.to_string()),
            Span::void(),
        )]),
    }
}

fn build_isa(target: Option<&str>) -> Result<std::sync::Arc<dyn isa::TargetIsa>, String> {
    let triple = match target {
        Some(target) => Triple::from_str(target).map_err(|error| error.to_string())?,
        None => Triple::host(),
    };
    let flags = settings::Flags::new(settings::builder());
    let builder = isa::lookup(triple).map_err(|error| error.to_string())?;
    builder.finish(flags).map_err(|error| error.to_string())
}

fn declare_function<'a, M: Module>(
    module: &mut M,
    pointer: IrType,
    value: &Function<Str<'a>, Analysis<'a>, Option<Box<Analysis<'a>>>, Option<Type<'a>>>,
) -> Result<FuncData<'a>, String> {
    let mut sig = module.make_signature();
    let output = value.output.clone();
    let use_memory = output.as_ref().is_some_and(indirect);

    if use_memory {
        sig.params.push(AbiParam::new(pointer));
    }

    for member in &value.members {
        if let AnalysisKind::Binding(binding) = &member.kind {
            let typing = binding.annotation.clone();
            if indirect(&typing) {
                sig.params.push(AbiParam::new(pointer));
            } else if let Some(kind) = scalar_type(&typing, pointer) {
                sig.params.push(AbiParam::new(kind));
            }
        }
    }

    if let Some(output) = &output {
        if !use_memory {
            if let Some(kind) = scalar_type(output, pointer) {
                sig.returns.push(AbiParam::new(kind));
            }
        }
    }

    let linkage = if value.body.is_some() || value.entry {
        Linkage::Export
    } else {
        Linkage::Import
    };

    let id = module
        .declare_function(value.target.as_str().unwrap_or_default(), linkage, &sig)
        .map_err(|error| error.to_string())?;

    Ok(FuncData {
        id,
        sig,
        output,
        indirect: use_memory,
        variadic: value.variadic,
        entry: value.entry,
        interface: value.interface,
    })
}

fn define_static<'a, M: Module>(
    _module: &mut M,
    _pointer: IrType,
    _entities: &Map<Str<'a>, Entity<'a>>,
    _binding: Binding<Box<Analysis<'a>>, Box<Analysis<'a>>, Type<'a>>,
    span: Span,
) -> Result<(), GenerateError<'a>> {
    Err(GenerateError::new(
        ErrorKind::Verification("Cranelift static bindings are not supported yet".to_string()),
        span,
    ))
}

fn define_function<'a, M: Module>(
    module: &mut M,
    pointer: IrType,
    entities: &Map<Str<'a>, Entity<'a>>,
    value: Function<Str<'a>, Analysis<'a>, Option<Box<Analysis<'a>>>, Option<Type<'a>>>,
    span: Span,
) -> Result<(), GenerateError<'a>> {
    if matches!(value.interface, Interface::C) && value.body.is_none() {
        return Ok(());
    }

    let Entity::Function(func) = entities.get(&value.target).cloned().ok_or_else(|| {
        GenerateError::new(
            ErrorKind::Function(FunctionError::Undefined {
                name: value.target.as_str().unwrap_or_default().to_string(),
            }),
            span,
        )
    })?
    else {
        return Err(GenerateError::new(
            ErrorKind::Function(FunctionError::Undefined {
                name: value.target.as_str().unwrap_or_default().to_string(),
            }),
            span,
        ));
    };

    let mut ctx = module.make_context();
    ctx.func =
        IrFunction::with_name_signature(UserFuncName::user(0, func.id.as_u32()), func.sig.clone());
    let mut funcs = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut funcs);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    builder.append_block_params_for_function_params(entry);
    builder.seal_block(entry);

    let mut lower = Lower {
        module,
        builder,
        pointer,
        entities: entities.clone(),
        func,
        loops: Vec::new(),
        ret: None,
    };

    let params = lower.builder.block_params(entry).to_vec();
    let mut next = 0usize;

    if lower.func.indirect {
        lower.ret = Some(params[next]);
        next += 1;
    }

    for member in &value.members {
        if let AnalysisKind::Binding(binding) = &member.kind {
            let AnalysisKind::Symbol(target) = &binding.target.kind else {
                continue;
            };
            let slot = lower.stack(&binding.annotation);
            let addr = lower.addr(slot);
            let param = params[next];
            next += 1;

            if indirect(&binding.annotation) {
                lower.copy(&binding.annotation, param, addr);
            } else {
                lower.store(addr, &binding.annotation, param);
            }

            lower.entities.insert(
                target.name,
                Entity::Variable {
                    slot,
                    typing: binding.annotation.clone(),
                },
            );
        }
    }

    if let Some(body) = value.body {
        let result = lower.expr(*body)?;
        if !lower.done() {
            if let Some(output) = lower.func.output.clone() {
                if lower.func.indirect {
                    if let Some(ret) = lower.ret {
                        lower.write(ret, &output, result);
                    }
                    lower.builder.ins().return_(&[]);
                } else {
                    lower.builder.ins().return_(&[result]);
                }
            } else {
                lower.builder.ins().return_(&[]);
            }
        }
    } else if !lower.done() {
        lower.builder.ins().return_(&[]);
    }

    let id = lower.func.id;
    lower.builder.finalize();

    module
        .define_function(id, &mut ctx)
        .map_err(|error| GenerateError::new(ErrorKind::Verification(error.to_string()), span))?;

    Ok(())
}

fn member_names<'a>(value: &Aggregate<Str<'a>, Analysis<'a>>) -> Vec<Str<'a>> {
    let mut members = Vec::new();
    for member in &value.members {
        if let AnalysisKind::Binding(binding) = &member.kind {
            if let AnalysisKind::Symbol(target) = &binding.target.kind {
                members.push(target.name);
            }
        }
    }
    members
}

fn resolved<'a>(typing: &Type<'a>) -> Type<'a> {
    match &typing.kind {
        TypeKind::Binding(value) => value
            .value
            .as_deref()
            .cloned()
            .or_else(|| value.annotation.as_deref().cloned())
            .unwrap_or_else(|| Type::from(TypeKind::Unknown)),
        TypeKind::Has(value) => resolved(value),
        _ => typing.clone(),
    }
}

fn indirect<'a>(typing: &Type<'a>) -> bool {
    matches!(
        resolved(typing).kind,
        TypeKind::Array { .. }
            | TypeKind::Tuple { .. }
            | TypeKind::Structure(_)
            | TypeKind::Union(_)
    )
}

fn scalar_type<'a>(typing: &Type<'a>, pointer: IrType) -> Option<IrType> {
    match resolved(typing).kind {
        TypeKind::Integer { size, .. } => Some(int_type(size)),
        TypeKind::Float { size } => match size {
            32 => Some(types::F32),
            64 => Some(types::F64),
            _ => None,
        },
        TypeKind::Boolean => Some(types::I8),
        TypeKind::Character => Some(types::I32),
        TypeKind::String | TypeKind::Pointer { .. } | TypeKind::Function(_) => Some(pointer),
        TypeKind::Void => None,
        _ => None,
    }
}

fn int_type(size: Scale) -> IrType {
    match size {
        1 => types::I8,
        8 => types::I8,
        16 => types::I16,
        32 => types::I32,
        64 => types::I64,
        128 => types::I128,
        _ => types::I64,
    }
}

fn align_shift(align: u8) -> u8 {
    align.trailing_zeros() as u8
}

fn layout<'a>(typing: &Type<'a>) -> Layout {
    match &resolved(typing).kind {
        TypeKind::Integer { size, .. } => {
            let bytes = ((*size).div_ceil(8)).max(1) as u32;
            Layout {
                size: bytes,
                align: bytes.min(8) as u8,
            }
        }
        TypeKind::Float { size } => {
            let bytes = ((*size).div_ceil(8)).max(1) as u32;
            Layout {
                size: bytes,
                align: bytes.min(8) as u8,
            }
        }
        TypeKind::Boolean => Layout { size: 1, align: 1 },
        TypeKind::Character => Layout { size: 4, align: 4 },
        TypeKind::String | TypeKind::Pointer { .. } | TypeKind::Function(_) => {
            Layout { size: 8, align: 8 }
        }
        TypeKind::Array { member, size } => {
            let item = layout(member);
            Layout {
                size: pad(item.size, item.align) * *size as u32,
                align: item.align,
            }
        }
        TypeKind::Tuple { members } => aggregate_layout(members),
        TypeKind::Structure(value) => aggregate_layout(&value.members),
        TypeKind::Union(value) => {
            let mut size = 0u32;
            let mut align = 1u8;
            for member in &value.members {
                let item = layout(member);
                size = size.max(item.size);
                align = align.max(item.align);
            }
            Layout {
                size: pad(size, align),
                align,
            }
        }
        _ => Layout { size: 8, align: 8 },
    }
}

fn aggregate_layout<'a>(members: &[Type<'a>]) -> Layout {
    let mut size = 0u32;
    let mut align = 1u8;
    for member in members {
        let item = layout(member);
        size = pad(size, item.align);
        size += item.size;
        align = align.max(item.align);
    }
    Layout {
        size: pad(size, align),
        align,
    }
}

fn pad(size: u32, align: u8) -> u32 {
    let align = align.max(1) as u32;
    let rem = size % align;
    if rem == 0 {
        size
    } else {
        size + align - rem
    }
}

impl<'a, 'b, M: Module> Lower<'a, 'b, M> {
    fn error(&self, kind: ErrorKind<'b>, span: Span) -> GenerateError<'b> {
        GenerateError::new(kind, span)
    }

    fn done(&self) -> bool {
        self.builder.is_unreachable()
    }

    fn stack(&mut self, typing: &Type<'b>) -> StackSlot {
        let data = layout(typing);
        self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            data.size,
            align_shift(data.align),
        ))
    }

    fn addr(&mut self, slot: StackSlot) -> Value {
        self.builder.ins().stack_addr(self.pointer, slot, 0)
    }

    fn zero(&mut self, typing: &Type<'b>) -> Value {
        match scalar_type(typing, self.pointer).unwrap_or(self.pointer) {
            types::F32 => self.builder.ins().f32const(0.0),
            types::F64 => self.builder.ins().f64const(0.0),
            kind => self.builder.ins().iconst(kind, 0),
        }
    }

    fn truth(&mut self, value: Value) -> Value {
        let kind = self.builder.func.dfg.value_type(value);
        if kind == types::F32 {
            let zero = self.builder.ins().f32const(0.0);
            self.builder.ins().fcmp(FloatCC::NotEqual, value, zero)
        } else if kind == types::F64 {
            let zero = self.builder.ins().f64const(0.0);
            self.builder.ins().fcmp(FloatCC::NotEqual, value, zero)
        } else {
            let zero = self.builder.ins().iconst(kind, 0);
            self.builder.ins().icmp(IntCC::NotEqual, value, zero)
        }
    }

    fn cast_bool(&mut self, value: Value) -> Value {
        let kind = self.builder.func.dfg.value_type(value);
        if kind == types::I8 {
            value
        } else {
            self.builder.ins().uextend(types::I8, value)
        }
    }

    fn place(&mut self, analysis: &Analysis<'b>) -> Result<(Value, Type<'b>), GenerateError<'b>> {
        match &analysis.kind {
            AnalysisKind::Symbol(target) => match self.entities.get(&target.name).cloned() {
                Some(Entity::Variable { slot, typing }) => Ok((self.addr(slot), typing)),
                _ => Err(self.error(
                    ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                    analysis.span,
                )),
            },
            AnalysisKind::Usage(name) => match self.entities.get(name).cloned() {
                Some(Entity::Variable { slot, typing }) => Ok((self.addr(slot), typing)),
                _ => Err(self.error(
                    ErrorKind::Variable(VariableError::Undefined {
                        name: name.as_str().unwrap_or_default().to_string(),
                    }),
                    analysis.span,
                )),
            },
            AnalysisKind::Dereference(value) => {
                let addr = self.expr(*value.clone())?;
                Ok((addr, analysis.typing.clone()))
            }
            AnalysisKind::Slot(target, index) => self.slot_place(target, *index, analysis.span),
            AnalysisKind::Access(target, member) => {
                let Some(name) = symbol(member) else {
                    return Err(self.error(
                        ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression),
                        analysis.span,
                    ));
                };
                let base = member_index(&resolved(&target.typing), name).ok_or_else(|| {
                    self.error(
                        ErrorKind::DataStructure(DataStructureError::UnknownField {
                            target: String::new(),
                            member: name.as_str().unwrap_or_default().to_string(),
                        }),
                        analysis.span,
                    )
                })?;
                self.slot_place(target, base as Scale, analysis.span)
            }
            AnalysisKind::Index(value) => self.index_place(value, analysis.span),
            _ => Err(self.error(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                analysis.span,
            )),
        }
    }

    fn slot_place(
        &mut self,
        target: &Analysis<'b>,
        index: Scale,
        span: Span,
    ) -> Result<(Value, Type<'b>), GenerateError<'b>> {
        let (base, typing) = self.base_place(target)?;
        let target = field_type(&typing, index as usize).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    target: String::new(),
                    member: index.to_string(),
                }),
                span,
            )
        })?;
        let offset = field_offset(&typing, index as usize).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    target: String::new(),
                    member: index.to_string(),
                }),
                span,
            )
        })?;
        let addr = if offset == 0 {
            base
        } else {
            self.builder.ins().iadd_imm(base, offset as i64)
        };
        Ok((addr, target))
    }

    fn index_place(
        &mut self,
        value: &Index<Box<Analysis<'b>>, Analysis<'b>>,
        span: Span,
    ) -> Result<(Value, Type<'b>), GenerateError<'b>> {
        let member = value.members.first().ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::IndexMissingArgument),
                span,
            )
        })?;
        let (base, typing) = self.base_place(&value.target)?;
        let item = element_type(&typing).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::NotIndexable),
                span,
            )
        })?;
        let scale = self.expr(member.clone())?;
        let scale = self.extend(scale, self.pointer, false);
        let step = layout(&item).size as i64;
        let offs = if step == 1 {
            scale
        } else {
            self.builder.ins().imul_imm(scale, step)
        };
        let addr = self.builder.ins().iadd(base, offs);
        Ok((addr, item))
    }

    fn base_place(
        &mut self,
        target: &Analysis<'b>,
    ) -> Result<(Value, Type<'b>), GenerateError<'b>> {
        match &resolved(&target.typing).kind {
            TypeKind::Pointer { target: inner } => {
                Ok((self.expr(target.clone())?, (*inner.clone()).clone()))
            }
            _ => self.place(target),
        }
    }

    fn expr(&mut self, analysis: Analysis<'b>) -> Result<Value, GenerateError<'b>> {
        let span = analysis.span;
        let typing = analysis.typing.clone();
        match analysis.kind {
            AnalysisKind::Integer { value, size, .. } => {
                Ok(self.builder.ins().iconst(int_type(size), value as i64))
            }
            AnalysisKind::Float { value, size } => match size {
                32 => Ok(self.builder.ins().f32const(value.0 as f32)),
                64 => Ok(self.builder.ins().f64const(value.0)),
                width => Err(self.error(ErrorKind::UnsupportedFloatWidth(width), span)),
            },
            AnalysisKind::Boolean { value } => {
                Ok(self.builder.ins().iconst(types::I8, i64::from(value)))
            }
            AnalysisKind::Character { value } => {
                Ok(self.builder.ins().iconst(types::I32, value as i64))
            }
            AnalysisKind::String { value } => self.string(value, span),
            AnalysisKind::Array(values) => self.array(&typing, values),
            AnalysisKind::Tuple(values) => self.tuple(&typing, values),
            AnalysisKind::Negate(value) => self.negate(*value, span),
            AnalysisKind::SizeOf(value) => Ok(self
                .builder
                .ins()
                .iconst(types::I64, layout(&value).size as i64)),
            AnalysisKind::Add(left, right) => self.add(*left, *right, span),
            AnalysisKind::Subtract(left, right) => self.subtract(*left, *right, span),
            AnalysisKind::Multiply(left, right) => self.multiply(*left, *right, span),
            AnalysisKind::Divide(left, right) => self.divide(*left, *right, span),
            AnalysisKind::Modulus(left, right) => self.modulus(*left, *right, span),
            AnalysisKind::LogicalAnd(left, right) => self.logical_and(*left, *right),
            AnalysisKind::LogicalOr(left, right) => self.logical_or(*left, *right),
            AnalysisKind::LogicalNot(value) => {
                let value = self.expr(*value)?;
                let value = self.truth(value);
                let value = self.builder.ins().bnot(value);
                Ok(self.cast_bool(value))
            }
            AnalysisKind::LogicalXOr(left, right) => {
                let left = self.expr(*left)?;
                let left = self.truth(left);
                let right = self.expr(*right)?;
                let right = self.truth(right);
                let value = self.builder.ins().bxor(left, right);
                Ok(self.cast_bool(value))
            }
            AnalysisKind::BitwiseAnd(left, right) => {
                self.bitwise(*left, *right, span, |this, left, right| {
                    this.builder.ins().band(left, right)
                })
            }
            AnalysisKind::BitwiseOr(left, right) => {
                self.bitwise(*left, *right, span, |this, left, right| {
                    this.builder.ins().bor(left, right)
                })
            }
            AnalysisKind::BitwiseNot(value) => {
                let value = self.expr(*value)?;
                let kind = self.builder.func.dfg.value_type(value);
                if kind.is_float() {
                    return Err(self.error(ErrorKind::Normalize, span));
                }
                Ok(self.builder.ins().bnot(value))
            }
            AnalysisKind::BitwiseXOr(left, right) => {
                self.bitwise(*left, *right, span, |this, left, right| {
                    this.builder.ins().bxor(left, right)
                })
            }
            AnalysisKind::ShiftLeft(left, right) => {
                self.bitwise(*left, *right, span, |this, left, right| {
                    this.builder.ins().ishl(left, right)
                })
            }
            AnalysisKind::ShiftRight(left, right) => {
                let signed = signed(&left.typing) && signed(&right.typing);
                self.bitwise(*left, *right, span, move |this, left, right| {
                    if signed {
                        this.builder.ins().sshr(left, right)
                    } else {
                        this.builder.ins().ushr(left, right)
                    }
                })
            }
            AnalysisKind::AddressOf(value) => Ok(self.place(&value)?.0),
            AnalysisKind::Dereference(value) => {
                let addr = self.expr(*value)?;
                if indirect(&typing) {
                    Ok(addr)
                } else {
                    self.load(addr, &typing)
                }
            }
            AnalysisKind::Equal(left, right) => self.compare(
                *left,
                *right,
                span,
                FloatCC::Equal,
                IntCC::Equal,
                IntCC::Equal,
            ),
            AnalysisKind::NotEqual(left, right) => self.compare(
                *left,
                *right,
                span,
                FloatCC::NotEqual,
                IntCC::NotEqual,
                IntCC::NotEqual,
            ),
            AnalysisKind::Less(left, right) => self.ordered(
                *left,
                *right,
                span,
                FloatCC::LessThan,
                IntCC::SignedLessThan,
                IntCC::UnsignedLessThan,
            ),
            AnalysisKind::LessOrEqual(left, right) => self.ordered(
                *left,
                *right,
                span,
                FloatCC::LessThanOrEqual,
                IntCC::SignedLessThanOrEqual,
                IntCC::UnsignedLessThanOrEqual,
            ),
            AnalysisKind::Greater(left, right) => self.ordered(
                *left,
                *right,
                span,
                FloatCC::GreaterThan,
                IntCC::SignedGreaterThan,
                IntCC::UnsignedGreaterThan,
            ),
            AnalysisKind::GreaterOrEqual(left, right) => self.ordered(
                *left,
                *right,
                span,
                FloatCC::GreaterThanOrEqual,
                IntCC::SignedGreaterThanOrEqual,
                IntCC::UnsignedGreaterThanOrEqual,
            ),
            AnalysisKind::Index(value) => {
                let (addr, item) = self.index_place(&value, span)?;
                if indirect(&item) {
                    Ok(addr)
                } else {
                    self.load(addr, &item)
                }
            }
            AnalysisKind::Usage(name) => self.read(name, span),
            AnalysisKind::Symbol(target) => self.read(target.name, span),
            AnalysisKind::Access(target, member) => {
                let (addr, item) = self.place(&Analysis::new(
                    AnalysisKind::Access(target, member),
                    span,
                    typing.clone(),
                ))?;
                if indirect(&item) {
                    Ok(addr)
                } else {
                    self.load(addr, &item)
                }
            }
            AnalysisKind::Slot(target, index) => {
                let (addr, item) = self.slot_place(&target, index, span)?;
                if indirect(&item) {
                    Ok(addr)
                } else {
                    self.load(addr, &item)
                }
            }
            AnalysisKind::Constructor(value) => self.constructor(&typing, value, span),
            AnalysisKind::Pack(target, values) => self.pack(&typing, target, values, span),
            AnalysisKind::Assign(name, value) => self.assign(name, *value, span),
            AnalysisKind::Write(target, value) => self.write_target(target, *value, span),
            AnalysisKind::Store(target, value) => self.store_target(*target, *value, span),
            AnalysisKind::Binding(value) => self.bind(value, span),
            AnalysisKind::Structure(_)
            | AnalysisKind::Union(_)
            | AnalysisKind::Composite(_)
            | AnalysisKind::Module(_, _) => Ok(self.builder.ins().iconst(types::I64, 0)),
            AnalysisKind::Function(_) => Ok(self.builder.ins().iconst(types::I64, 0)),
            AnalysisKind::Block(values) => self.block(values),
            AnalysisKind::Conditional(condition, truth, fall) => {
                self.conditional(typing, *condition, *truth, fall.map(|item| *item), span)
            }
            AnalysisKind::While(condition, body) => self.loop_expr(typing, *condition, *body, span),
            AnalysisKind::Invoke(_) => Err(self.error(
                ErrorKind::Verification("unexpected invoke".to_string()),
                span,
            )),
            AnalysisKind::Call(target, values) => self.call(target, values, &typing, span),
            AnalysisKind::Return(value) => self.return_value(value.map(|item| *item), span),
            AnalysisKind::Break(value) => self.break_value(value.map(|item| *item), span),
            AnalysisKind::Continue(_) => self.continue_value(span),
        }
    }

    fn string(&mut self, value: Str<'b>, span: Span) -> Result<Value, GenerateError<'b>> {
        let text = value.as_str().unwrap_or_default();
        let name = format!(".str.{}", self.builder.func.dfg.num_values());
        let id = self
            .module
            .declare_data(&name, Linkage::Local, false, false)
            .map_err(|error| self.error(ErrorKind::Verification(error.to_string()), span))?;
        let mut data = cranelift_module::DataDescription::new();
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        data.define(bytes.into_boxed_slice());
        self.module
            .define_data(id, &data)
            .map_err(|error| self.error(ErrorKind::Verification(error.to_string()), span))?;
        let gv = self.module.declare_data_in_func(id, &mut self.builder.func);
        Ok(self.builder.ins().global_value(self.pointer, gv))
    }

    fn array(
        &mut self,
        typing: &Type<'b>,
        values: Vec<Analysis<'b>>,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        if let TypeKind::Array { member, .. } = &resolved(typing).kind {
            let step = layout(member).size;
            for (index, value) in values.into_iter().enumerate() {
                let item = if step == 0 {
                    addr
                } else {
                    self.builder
                        .ins()
                        .iadd_imm(addr, (index as u32 * step) as i64)
                };
                let value = self.expr(value)?;
                self.write(item, member, value);
            }
        }
        Ok(addr)
    }

    fn tuple(
        &mut self,
        typing: &Type<'b>,
        values: Vec<Analysis<'b>>,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        if let TypeKind::Tuple { members } = &resolved(typing).kind {
            for (index, value) in values.into_iter().enumerate() {
                if let Some(item) = members.get(index) {
                    let offs = field_offset(typing, index).unwrap_or(0);
                    let place = if offs == 0 {
                        addr
                    } else {
                        self.builder.ins().iadd_imm(addr, offs as i64)
                    };
                    let value = self.expr(value)?;
                    self.write(place, item, value);
                }
            }
        }
        Ok(addr)
    }

    fn constructor(
        &mut self,
        typing: &Type<'b>,
        value: Aggregate<Str<'b>, Analysis<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        let names = member_names_of(typing);
        for (index, item) in value.members.into_iter().enumerate() {
            match item.kind {
                AnalysisKind::Assign(name, value) => {
                    if let Some(slot) = names.iter().position(|item| *item == name) {
                        let place = self.field_addr(addr, typing, slot, span)?;
                        let item_type = field_type(typing, slot).unwrap();
                        let value = self.expr(*value)?;
                        self.write(place, &item_type, value);
                    }
                }
                _ => {
                    let place = self.field_addr(addr, typing, index, span)?;
                    let item_type = field_type(typing, index).unwrap();
                    let value = self.expr(item)?;
                    self.write(place, &item_type, value);
                }
            }
        }
        Ok(addr)
    }

    fn pack(
        &mut self,
        typing: &Type<'b>,
        _target: Target<'b>,
        values: Vec<(Scale, Analysis<'b>)>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let slot = self.stack(typing);
        let addr = self.addr(slot);
        for (index, value) in values {
            let slot = index as usize;
            let place = self.field_addr(addr, typing, slot, span)?;
            let item = field_type(typing, slot).unwrap();
            let value = self.expr(value)?;
            self.write(place, &item, value);
        }
        Ok(addr)
    }

    fn field_addr(
        &mut self,
        addr: Value,
        typing: &Type<'b>,
        index: usize,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let offs = field_offset(typing, index).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    target: String::new(),
                    member: index.to_string(),
                }),
                span,
            )
        })?;
        Ok(if offs == 0 {
            addr
        } else {
            self.builder.ins().iadd_imm(addr, offs as i64)
        })
    }

    fn bind(
        &mut self,
        value: Binding<Box<Analysis<'b>>, Box<Analysis<'b>>, Type<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let AnalysisKind::Symbol(target) = &value.target.kind else {
            return Err(self.error(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                span,
            ));
        };
        let slot = self.stack(&value.annotation);
        let addr = self.addr(slot);
        if let Some(init) = value.value {
            let current = self.expr(*init)?;
            self.write(addr, &value.annotation, current);
        } else {
            if matches!(value.kind, BindingKind::Let) {
                return Err(self.error(
                    ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                        name: target.name.as_str().unwrap_or_default().to_string(),
                    }),
                    span,
                ));
            }
        }
        self.entities.insert(
            target.name,
            Entity::Variable {
                slot,
                typing: value.annotation.clone(),
            },
        );
        if indirect(&value.annotation) {
            Ok(addr)
        } else {
            self.load(addr, &value.annotation)
        }
    }

    fn read(&mut self, name: Str<'b>, span: Span) -> Result<Value, GenerateError<'b>> {
        match self.entities.get(&name).cloned() {
            Some(Entity::Variable { slot, typing }) => {
                let addr = self.addr(slot);
                if indirect(&typing) {
                    Ok(addr)
                } else {
                    self.load(addr, &typing)
                }
            }
            Some(Entity::Function(_)) => Err(self.error(
                ErrorKind::Function(FunctionError::Undefined {
                    name: name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            )),
            _ => Err(self.error(
                ErrorKind::Variable(VariableError::Undefined {
                    name: name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            )),
        }
    }

    fn assign(
        &mut self,
        name: Str<'b>,
        value: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let target = Analysis::new(AnalysisKind::Usage(name), span, value.typing.clone());
        self.store_target(target, value, span)
    }

    fn write_target(
        &mut self,
        target: Target<'b>,
        value: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.store_target(
            Analysis::new(AnalysisKind::Symbol(target), span, value.typing.clone()),
            value,
            span,
        )
    }

    fn store_target(
        &mut self,
        target: Analysis<'b>,
        value: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let (addr, typing) = self.place(&target)?;
        let value = self.expr(value)?;
        self.write(addr, &typing, value);
        if indirect(&typing) {
            Ok(addr)
        } else {
            self.load(addr, &typing)
        }
    }

    fn block(&mut self, values: Vec<Analysis<'b>>) -> Result<Value, GenerateError<'b>> {
        let mut last = self.builder.ins().iconst(types::I64, 0);
        for value in values {
            if self.done() {
                break;
            }
            last = self.expr(value)?;
        }
        Ok(last)
    }

    fn conditional(
        &mut self,
        typing: Type<'b>,
        condition: Analysis<'b>,
        truth: Analysis<'b>,
        fall: Option<Analysis<'b>>,
        _span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let check = self.expr(condition)?;
        let check = self.truth(check);
        let pass = self.builder.create_block();
        let fail = self.builder.create_block();
        let join = self.builder.create_block();
        self.builder.ins().brif(check, pass, &[], fail, &[]);

        let mut slot = None;
        let mut var = None;

        if indirect(&typing) {
            slot = Some(self.stack(&typing));
        } else if let Some(kind) = scalar_type(&typing, self.pointer) {
            let temp = self.builder.declare_var(kind);
            var = Some(temp);
        }

        self.builder.switch_to_block(pass);
        let left = self.expr(truth)?;
        if let Some(slot) = slot {
            let addr = self.addr(slot);
            self.write(addr, &typing, left);
        }
        if let Some(var) = var {
            self.builder.def_var(var, left);
        }
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(pass);

        self.builder.switch_to_block(fail);
        let right = if let Some(fall) = fall {
            self.expr(fall)?
        } else if indirect(&typing) {
            let slot = self.stack(&typing);
            self.addr(slot)
        } else {
            self.zero(&typing)
        };
        if let Some(slot) = slot {
            let addr = self.addr(slot);
            self.write(addr, &typing, right);
        }
        if let Some(var) = var {
            self.builder.def_var(var, right);
        }
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(fail);

        self.builder.switch_to_block(join);
        self.builder.seal_block(join);

        if let Some(slot) = slot {
            Ok(self.addr(slot))
        } else if let Some(var) = var {
            Ok(self.builder.use_var(var))
        } else {
            Ok(self.zero(&typing))
        }
    }

    fn loop_expr(
        &mut self,
        typing: Type<'b>,
        condition: Analysis<'b>,
        body: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let head = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit = self.builder.create_block();
        let slot = if matches!(resolved(&typing).kind, TypeKind::Void | TypeKind::Unknown) {
            None
        } else {
            Some(self.stack(&typing))
        };
        self.builder.ins().jump(head, &[]);
        self.builder.switch_to_block(head);
        let check = self.expr(condition)?;
        let check = self.truth(check);
        self.builder.ins().brif(check, body_block, &[], exit, &[]);
        self.loops.push(Loop { head, exit, slot });

        self.builder.switch_to_block(body_block);
        let _ = self.expr(body)?;
        if !self.done() {
            self.builder.ins().jump(head, &[]);
        }
        self.builder.seal_block(body_block);
        self.loops.pop();
        self.builder.seal_block(head);
        self.builder.switch_to_block(exit);
        self.builder.seal_block(exit);
        if let Some(slot) = slot {
            Ok(self.addr(slot))
        } else {
            Ok(self.builder.ins().iconst(types::I64, 0))
        }
    }

    fn call(
        &mut self,
        target: Target<'b>,
        values: Vec<Analysis<'b>>,
        typing: &Type<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let Some(Entity::Function(func)) = self.entities.get(&target.name).cloned() else {
            return Err(self.error(
                ErrorKind::Function(FunctionError::Undefined {
                    name: target.name.as_str().unwrap_or_default().to_string(),
                }),
                span,
            ));
        };
        let callee = self
            .module
            .declare_func_in_func(func.id, &mut self.builder.func);
        let mut args = Vec::new();
        let mut slot = None;
        if func.indirect {
            let temp = self.stack(typing);
            let addr = self.addr(temp);
            slot = Some(temp);
            args.push(addr);
        }
        for value in values {
            let current = self.expr(value.clone())?;
            if indirect(&value.typing) {
                args.push(current);
            } else {
                args.push(current);
            }
        }
        let call = self.builder.ins().call(callee, &args);
        if let Some(slot) = slot {
            Ok(self.addr(slot))
        } else if let Some(result) = self.builder.inst_results(call).first().copied() {
            Ok(result)
        } else {
            Ok(self.builder.ins().iconst(types::I64, 0))
        }
    }

    fn return_value(
        &mut self,
        value: Option<Analysis<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        if self.done() {
            return Ok(self.builder.ins().iconst(types::I64, 0));
        }
        match (self.func.output.clone(), value) {
            (Some(output), Some(value)) => {
                let value = self.expr(value)?;
                if self.func.indirect {
                    let Some(ret) = self.ret else {
                        return Err(self.error(
                            ErrorKind::Function(FunctionError::IncompatibleReturnType),
                            span,
                        ));
                    };
                    self.write(ret, &output, value);
                    self.builder.ins().return_(&[]);
                } else {
                    self.builder.ins().return_(&[value]);
                }
                Ok(value)
            }
            (None, _) => {
                self.builder.ins().return_(&[]);
                Ok(self.builder.ins().iconst(types::I64, 0))
            }
            _ => Err(self.error(
                ErrorKind::Function(FunctionError::IncompatibleReturnType),
                span,
            )),
        }
    }

    fn break_value(
        &mut self,
        value: Option<Analysis<'b>>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let looped = self.loops.last().copied().ok_or_else(|| {
            self.error(
                ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                span,
            )
        })?;
        if let (Some(value), Some(loop_slot)) = (value, looped.slot) {
            let value = self.expr(value)?;
            let addr = self.addr(loop_slot);
            self.builder.ins().store(MemFlags::new(), value, addr, 0);
        }
        if !self.done() {
            self.builder.ins().jump(looped.exit, &[]);
        }
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    fn continue_value(&mut self, span: Span) -> Result<Value, GenerateError<'b>> {
        let looped = self.loops.last().copied().ok_or_else(|| {
            self.error(
                ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                span,
            )
        })?;
        if !self.done() {
            self.builder.ins().jump(looped.head, &[]);
        }
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    fn negate(&mut self, value: Analysis<'b>, span: Span) -> Result<Value, GenerateError<'b>> {
        let value = self.expr(value)?;
        let kind = self.builder.func.dfg.value_type(value);
        if kind.is_int() {
            Ok(self.builder.ins().ineg(value))
        } else if kind.is_float() {
            Ok(self.builder.ins().fneg(value))
        } else {
            Err(self.error(ErrorKind::Negate, span))
        }
    }

    fn add(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.numeric(left, right, span, |this, left, right, float| {
            if float {
                this.builder.ins().fadd(left, right)
            } else {
                this.builder.ins().iadd(left, right)
            }
        })
    }

    fn subtract(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.numeric(left, right, span, |this, left, right, float| {
            if float {
                this.builder.ins().fsub(left, right)
            } else {
                this.builder.ins().isub(left, right)
            }
        })
    }

    fn multiply(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        self.numeric(left, right, span, |this, left, right, float| {
            if float {
                this.builder.ins().fmul(left, right)
            } else {
                this.builder.ins().imul(left, right)
            }
        })
    }

    fn divide(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        self.numeric(left, right, span, move |this, left, right, float| {
            if float {
                this.builder.ins().fdiv(left, right)
            } else if sign {
                this.builder.ins().sdiv(left, right)
            } else {
                this.builder.ins().udiv(left, right)
            }
        })
    }

    fn modulus(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        if kind.is_float() {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        Ok(if sign {
            self.builder.ins().srem(left, right)
        } else {
            self.builder.ins().urem(left, right)
        })
    }

    fn numeric<F>(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        apply: F,
    ) -> Result<Value, GenerateError<'b>>
    where
        F: Fn(&mut Self, Value, Value, bool) -> Value,
    {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let left_kind = self.builder.func.dfg.value_type(left);
        let right_kind = self.builder.func.dfg.value_type(right);
        if left_kind != right_kind {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        Ok(apply(self, left, right, left_kind.is_float()))
    }

    fn bitwise<F>(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        apply: F,
    ) -> Result<Value, GenerateError<'b>>
    where
        F: Fn(&mut Self, Value, Value) -> Value,
    {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) || kind.is_float() {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        Ok(apply(self, left, right))
    }

    fn compare(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        float: FloatCC,
        ints: IntCC,
        _uints: IntCC,
    ) -> Result<Value, GenerateError<'b>> {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        let value = if kind.is_float() {
            self.builder.ins().fcmp(float, left, right)
        } else {
            self.builder.ins().icmp(ints, left, right)
        };
        Ok(self.cast_bool(value))
    }

    fn ordered(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
        span: Span,
        float: FloatCC,
        ints: IntCC,
        uints: IntCC,
    ) -> Result<Value, GenerateError<'b>> {
        let sign = signed(&left.typing) && signed(&right.typing);
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        let kind = self.builder.func.dfg.value_type(left);
        if kind != self.builder.func.dfg.value_type(right) {
            return Err(self.error(ErrorKind::Normalize, span));
        }
        let value = if kind.is_float() {
            self.builder.ins().fcmp(float, left, right)
        } else if sign {
            self.builder.ins().icmp(ints, left, right)
        } else {
            self.builder.ins().icmp(uints, left, right)
        };
        Ok(self.cast_bool(value))
    }

    fn logical_and(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
    ) -> Result<Value, GenerateError<'b>> {
        let left = self.expr(left)?;
        let left = self.truth(left);
        let pass = self.builder.create_block();
        let join = self.builder.create_block();
        let temp = self.builder.declare_var(types::I8);
        let zero = self.builder.ins().iconst(types::I8, 0);
        self.builder.def_var(temp, zero);
        self.builder.ins().brif(left, pass, &[], join, &[]);
        self.builder.switch_to_block(pass);
        let right = self.expr(right)?;
        let right = self.truth(right);
        let right = self.cast_bool(right);
        self.builder.def_var(temp, right);
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(pass);
        self.builder.switch_to_block(join);
        self.builder.seal_block(join);
        Ok(self.builder.use_var(temp))
    }

    fn logical_or(
        &mut self,
        left: Analysis<'b>,
        right: Analysis<'b>,
    ) -> Result<Value, GenerateError<'b>> {
        let left = self.expr(left)?;
        let left = self.truth(left);
        let pass = self.builder.create_block();
        let join = self.builder.create_block();
        let temp = self.builder.declare_var(types::I8);
        let one = self.builder.ins().iconst(types::I8, 1);
        self.builder.def_var(temp, one);
        self.builder.ins().brif(left, join, &[], pass, &[]);
        self.builder.switch_to_block(pass);
        let right = self.expr(right)?;
        let right = self.truth(right);
        let right = self.cast_bool(right);
        self.builder.def_var(temp, right);
        if !self.done() {
            self.builder.ins().jump(join, &[]);
        }
        self.builder.seal_block(pass);
        self.builder.switch_to_block(join);
        self.builder.seal_block(join);
        Ok(self.builder.use_var(temp))
    }

    fn load(&mut self, addr: Value, typing: &Type<'b>) -> Result<Value, GenerateError<'b>> {
        let kind = scalar_type(typing, self.pointer)
            .ok_or_else(|| self.error(ErrorKind::Normalize, Span::void()))?;
        let value = self.builder.ins().load(kind, MemFlags::new(), addr, 0);
        if matches!(resolved(typing).kind, TypeKind::Boolean) {
            let zero = self.builder.ins().iconst(types::I8, 0);
            let value = self.builder.ins().icmp(IntCC::NotEqual, value, zero);
            Ok(self.cast_bool(value))
        } else {
            Ok(value)
        }
    }

    fn store(&mut self, addr: Value, typing: &Type<'b>, value: Value) {
        let value = match scalar_type(typing, self.pointer) {
            Some(kind) if self.builder.func.dfg.value_type(value) != kind => {
                self.extend(value, kind, signed(typing))
            }
            _ => value,
        };
        self.builder.ins().store(MemFlags::new(), value, addr, 0);
    }

    fn write(&mut self, addr: Value, typing: &Type<'b>, value: Value) {
        if indirect(typing) {
            self.copy(typing, value, addr);
        } else {
            self.store(addr, typing, value);
        }
    }

    fn copy(&mut self, typing: &Type<'b>, src: Value, dst: Value) {
        let size = layout(typing).size;
        for offset in 0..size {
            let value = self
                .builder
                .ins()
                .load(types::I8, MemFlags::new(), src, offset as i32);
            self.builder
                .ins()
                .store(MemFlags::new(), value, dst, offset as i32);
        }
    }

    fn extend(&mut self, value: Value, kind: IrType, sign: bool) -> Value {
        let from = self.builder.func.dfg.value_type(value);
        if from == kind {
            return value;
        }
        if from.is_int() && kind.is_int() {
            if from.bits() > kind.bits() {
                self.builder.ins().ireduce(kind, value)
            } else if sign {
                self.builder.ins().sextend(kind, value)
            } else {
                self.builder.ins().uextend(kind, value)
            }
        } else if from.is_float() && kind.is_float() {
            if from.bits() > kind.bits() {
                self.builder.ins().fdemote(kind, value)
            } else {
                self.builder.ins().fpromote(kind, value)
            }
        } else {
            value
        }
    }
}

fn symbol<'a>(analysis: &Analysis<'a>) -> Option<Str<'a>> {
    match &analysis.kind {
        AnalysisKind::Usage(name) => Some(*name),
        AnalysisKind::Symbol(target) => Some(target.name),
        _ => None,
    }
}

fn member_index<'a>(typing: &Type<'a>, name: Str<'a>) -> Option<usize> {
    member_names_of(typing)
        .iter()
        .position(|item| *item == name)
}

fn member_names_of<'a>(typing: &Type<'a>) -> Vec<Str<'a>> {
    match &resolved(typing).kind {
        TypeKind::Structure(value) | TypeKind::Union(value) => {
            value.members.iter().filter_map(field_name).collect()
        }
        TypeKind::Tuple { .. } | TypeKind::Array { .. } => Vec::new(),
        _ => Vec::new(),
    }
}

fn field_name<'a>(typing: &Type<'a>) -> Option<Str<'a>> {
    match &resolved(typing).kind {
        TypeKind::Binding(value) => Some(value.target),
        TypeKind::Function(value) if !value.target.is_empty() => Some(value.target),
        _ => None,
    }
}

fn field_type<'a>(typing: &Type<'a>, index: usize) -> Option<Type<'a>> {
    match &resolved(typing).kind {
        TypeKind::Tuple { members } => members.get(index).cloned(),
        TypeKind::Structure(value) | TypeKind::Union(value) => value.members.get(index).cloned(),
        TypeKind::Array { member, .. } => Some((**member).clone()),
        _ => None,
    }
}

fn field_offset<'a>(typing: &Type<'a>, index: usize) -> Option<u32> {
    match &resolved(typing).kind {
        TypeKind::Tuple { members } => offset_of(members, index),
        TypeKind::Structure(value) => offset_of(&value.members, index),
        TypeKind::Union(value) => (index < value.members.len()).then_some(0),
        TypeKind::Array { member, .. } => Some(layout(member).size * index as u32),
        _ => None,
    }
}

fn offset_of<'a>(members: &[Type<'a>], index: usize) -> Option<u32> {
    if index >= members.len() {
        return None;
    }
    let mut offset = 0u32;
    for member in members.iter().take(index) {
        let data = layout(member);
        offset = pad(offset, data.align);
        offset += data.size;
    }
    let data = layout(&members[index]);
    Some(pad(offset, data.align))
}

fn element_type<'a>(typing: &Type<'a>) -> Option<Type<'a>> {
    match &resolved(typing).kind {
        TypeKind::Array { member, .. } => Some((**member).clone()),
        TypeKind::Pointer { target } => Some((**target).clone()),
        _ => None,
    }
}

fn signed<'a>(typing: &Type<'a>) -> bool {
    matches!(
        resolved(typing).kind,
        TypeKind::Integer { signed: true, .. }
    )
}

fn field_like<'a>(_output: &Option<Type<'a>>, _value: &Value, _span: Span) -> Type<'a> {
    Type::from(TypeKind::Unknown)
}
