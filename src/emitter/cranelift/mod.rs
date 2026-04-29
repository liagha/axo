mod arithmetic;
mod bitwise;
mod comparison;
mod composite;
mod error;
pub mod evaluate;
mod functions;
mod logical;
mod primitives;
mod variables;

pub use evaluate::{Engine, Value as EvalValue};

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
    Structure,
    Union,
    Function(FuncData<'a>),
    Module,
}

#[derive(Clone, Debug)]
pub struct FuncData<'a> {
    pub id: FuncId,
    pub sig: Signature,
    pub output: Option<Type<'a>>,
    pub indirect: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Layout {
    pub(crate) size: u32,
    pub(crate) align: u8,
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
                entities.insert(value.target, Entity::Structure);
            }
            AnalysisKind::Union(value) => {
                entities.insert(value.target, Entity::Union);
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

    Ok(FuncData { id, sig, output, indirect: use_memory })
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
    let head = builder.create_block();
    builder.switch_to_block(head);
    builder.append_block_params_for_function_params(head);
    builder.seal_block(head);

    let mut lower = Lower {
        module,
        builder,
        pointer,
        entities: entities.clone(),
        func,
        loops: Vec::new(),
        ret: None,
    };

    let params = lower.builder.block_params(head).to_vec();
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

pub(crate) fn resolved<'a>(typing: &Type<'a>) -> Type<'a> {
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

pub(crate) fn indirect<'a>(typing: &Type<'a>) -> bool {
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
        1 | 8 => types::I8,
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

pub(crate) fn layout<'a>(typing: &Type<'a>) -> Layout {
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
    let rest = size % align;
    if rest == 0 {
        size
    } else {
        size + align - rest
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
        let item = field_type(&typing, index as usize).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    target: String::new(),
                    member: index.to_string(),
                }),
                span,
            )
        })?;
        let offs = field_offset(&typing, index as usize).ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::UnknownField {
                    target: String::new(),
                    member: index.to_string(),
                }),
                span,
            )
        })?;
        let addr = if offs == 0 {
            base
        } else {
            self.builder.ins().iadd_imm(base, offs as i64)
        };
        Ok((addr, item))
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
            AnalysisKind::Integer { value, size, .. } => self.integer(value, size),
            AnalysisKind::Float { value, size } => self.float(value, size, span),
            AnalysisKind::Boolean { value } => self.boolean(value),
            AnalysisKind::Character { value } => self.character(value),
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
                let sign = signed(&left.typing) && signed(&right.typing);
                self.bitwise(*left, *right, span, move |this, left, right| {
                    if sign {
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
            | AnalysisKind::Module(_, _)
            | AnalysisKind::Function(_) => Ok(self.builder.ins().iconst(types::I64, 0)),
            AnalysisKind::Block(values) => self.block(values),
            AnalysisKind::Conditional(condition, truth, fall) => {
                self.conditional(typing, *condition, *truth, fall.map(|item| *item), span)
            }
            AnalysisKind::While(condition, body) => self.loop_expr(typing, *condition, *body),
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
    member_names_of(typing).iter().position(|item| *item == name)
}

fn member_names_of<'a>(typing: &Type<'a>) -> Vec<Str<'a>> {
    match &resolved(typing).kind {
        TypeKind::Structure(value) | TypeKind::Union(value) => {
            value.members.iter().filter_map(field_name).collect()
        }
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

pub(crate) fn field_type<'a>(typing: &Type<'a>, index: usize) -> Option<Type<'a>> {
    match &resolved(typing).kind {
        TypeKind::Tuple { members } => members.get(index).cloned(),
        TypeKind::Structure(value) | TypeKind::Union(value) => value.members.get(index).cloned(),
        TypeKind::Array { member, .. } => Some((**member).clone()),
        _ => None,
    }
}

pub(crate) fn field_offset<'a>(typing: &Type<'a>, index: usize) -> Option<u32> {
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
