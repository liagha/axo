use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::{Function, Identity, Interface, Str},
        generator::{cranelift::{field_offset, field_type, layout, lower, resolved, Entity}, ErrorKind, GenerateError, FunctionError},
        internal::{Artifact, Session},
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    cranelift_jit::{JITBuilder, JITModule},
    cranelift_module::default_libcall_names,
    std::{ffi::CStr, marker::PhantomData, mem::transmute},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Sequence(Vec<Value>),
    Composite(Vec<Value>),
    Empty,
}

pub struct Engine<'a>(PhantomData<&'a ()>);

impl<'a> Default for Engine<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn reset(&mut self) {}

    pub fn process<'b>(
        &mut self,
        session: &Session<'b>,
        keys: &[Identity],
    ) -> Result<(), GenerateError<'b>> {
        let plan = Plan::build(session, keys, None);
        if plan.items.is_empty() {
            return Ok(());
        }
        let mut module =
            Self::module().map_err(|error| GenerateError::new(ErrorKind::Verification(error), Span::void()))?;
        lower(&mut module, plan.items).map_err(first)?;
        Ok(())
    }

    pub fn execute_line<'b>(
        &mut self,
        session: &Session<'b>,
        key: Identity,
    ) -> Result<Option<Value>, GenerateError<'b>> {
        let plan = Plan::build(session, &session.all_source_keys(), Some(key));
        if plan.output.is_none() && plan.body.is_empty() {
            return Ok(None);
        }

        let mut module = Self::module().map_err(|error| GenerateError::new(ErrorKind::Verification(error), Span::void()))?;
        let entities = lower(&mut module, plan.items).map_err(first)?;
        let Entity::Function(func) = entities
            .get(&plan.name)
            .cloned()
            .ok_or_else(|| {
                GenerateError::new(
                    ErrorKind::Function(FunctionError::Undefined {
                        name: plan.name.as_str().unwrap_or_default().to_string(),
                    }),
                    Span::void(),
                )
            })?
        else {
            return Ok(Some(Value::Empty));
        };

        module
            .finalize_definitions()
            .map_err(|error| GenerateError::new(ErrorKind::Verification(error.to_string()), Span::void()))?;

        let code = module.get_finalized_function(func.id);
        let value = unsafe { call(code, plan.output.as_ref()) };
        Ok(Some(value))
    }

    fn module() -> Result<JITModule, String> {
        let mut builder = JITBuilder::new(default_libcall_names()).map_err(|error| error.to_string())?;
        builder.symbol("malloc", libc_malloc as *const u8);
        builder.symbol("free", libc_free as *const u8);
        Ok(JITModule::new(builder))
    }
}

struct Plan<'a> {
    name: Str<'a>,
    items: Vec<Analysis<'a>>,
    body: Vec<Analysis<'a>>,
    output: Option<Type<'a>>,
}

impl<'a> Plan<'a> {
    fn build(session: &Session<'a>, keys: &[Identity], line: Option<Identity>) -> Self {
        let mut defs = Vec::new();
        let mut body = Vec::new();
        let name = Str::from("__dialog_eval");

        let mut ordered = session.source_keys(keys);
        ordered.sort();

        for key in ordered {
            if line.is_some() && line != Some(key) && (key & 0x40000000) != 0 {
                continue;
            }
            let Some(record) = session.records.get(&key) else {
                continue;
            };
            let Some(Artifact::Analyses(items)) = record.fetch(3) else {
                continue;
            };
            for item in items {
                if top(item) {
                    defs.push(item.clone());
                } else {
                    body.push(item.clone());
                }
            }
        }

        let output = body.last().map(|item| item.typing.clone());

        let mut items = defs;
        if !body.is_empty() {
            items.push(wrapper(name, &body, output.clone()));
        }

        Self { name, items, body, output }
    }
}

fn top(analysis: &Analysis<'_>) -> bool {
    matches!(
        analysis.kind,
        AnalysisKind::Structure(_) | AnalysisKind::Union(_) | AnalysisKind::Function(_) | AnalysisKind::Module(_, _)
    )
}

fn wrapper<'a>(name: Str<'a>, body: &[Analysis<'a>], output: Option<Type<'a>>) -> Analysis<'a> {
    let body_type = output.clone().unwrap_or_else(|| Type::from(TypeKind::Void));
    let block = Analysis::new(AnalysisKind::Block(body.to_vec()), Span::void(), body_type);
    let func = Function::new(name, Vec::new(), Some(Box::new(block)), output, Interface::Axo, false, false);
    Analysis::new(AnalysisKind::Function(func), Span::void(), Type::from(TypeKind::Unknown))
}

fn first<'a>(errors: Vec<GenerateError<'a>>) -> GenerateError<'a> {
    errors.into_iter().next().unwrap_or_else(|| {
        GenerateError::new(ErrorKind::Verification("Cranelift evaluation failed".to_string()), Span::void())
    })
}

unsafe extern "C" fn libc_malloc(size: usize) -> *mut u8 {
    std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(size.max(1), 8))
}

unsafe extern "C" fn libc_free(_ptr: *mut u8) {}

unsafe fn call(code: *const u8, output: Option<&Type<'_>>) -> Value {
    let Some(output) = output else {
        let func: extern "C" fn() = transmute(code);
        func();
        return Value::Empty;
    };

    match &resolved(output).kind {
        TypeKind::Integer { size, signed } => match *size {
            1 | 8 => {
                let func: extern "C" fn() -> i8 = transmute(code);
                let value = func();
                if *signed { Value::Integer(value as i64) } else { Value::Integer((value as u8) as i64) }
            }
            16 => {
                let func: extern "C" fn() -> i16 = transmute(code);
                let value = func();
                if *signed { Value::Integer(value as i64) } else { Value::Integer((value as u16) as i64) }
            }
            32 => {
                let func: extern "C" fn() -> i32 = transmute(code);
                let value = func();
                if *signed { Value::Integer(value as i64) } else { Value::Integer((value as u32) as i64) }
            }
            64 => {
                let func: extern "C" fn() -> i64 = transmute(code);
                let value = func();
                Value::Integer(value)
            }
            _ => {
                let func: extern "C" fn() -> i64 = transmute(code);
                Value::Integer(func())
            }
        },
        TypeKind::Float { size } => {
            if *size == 32 {
                let func: extern "C" fn() -> f32 = transmute(code);
                Value::Float(func() as f64)
            } else {
                let func: extern "C" fn() -> f64 = transmute(code);
                Value::Float(func())
            }
        }
        TypeKind::Boolean => {
            let func: extern "C" fn() -> i8 = transmute(code);
            Value::Boolean(func() != 0)
        }
        TypeKind::Character => {
            let func: extern "C" fn() -> u32 = transmute(code);
            Value::Character(char::from_u32(func()).unwrap_or('\0'))
        }
        TypeKind::String => {
            let func: extern "C" fn() -> *const u8 = transmute(code);
            text(func())
        }
        TypeKind::Pointer { .. } => {
            let func: extern "C" fn() -> usize = transmute(code);
            Value::Integer(func() as i64)
        }
        TypeKind::Array { .. } | TypeKind::Tuple { .. } | TypeKind::Structure(_) | TypeKind::Union(_) => {
            let mut bytes = vec![0u8; layout(output).size as usize];
            let func: extern "C" fn(*mut u8) = transmute(code);
            func(bytes.as_mut_ptr());
            decode(output, &bytes, 0)
        }
        _ => Value::Empty,
    }
}

unsafe fn text(ptr: *const u8) -> Value {
    if ptr.is_null() {
        Value::String(String::new())
    } else {
        Value::String(CStr::from_ptr(ptr.cast()).to_string_lossy().into_owned())
    }
}

fn decode(typing: &Type<'_>, bytes: &[u8], offset: usize) -> Value {
    match &resolved(typing).kind {
        TypeKind::Integer { size, signed } => Value::Integer(match *size {
            1 | 8 => {
                let value = bytes[offset] as i64;
                if *signed { (value as i8) as i64 } else { value }
            }
            16 => {
                let value = u16::from_ne_bytes(load::<2>(bytes, offset));
                if *signed { (value as i16) as i64 } else { value as i64 }
            }
            32 => {
                let value = u32::from_ne_bytes(load::<4>(bytes, offset));
                if *signed { (value as i32) as i64 } else { value as i64 }
            }
            64 => i64::from_ne_bytes(load::<8>(bytes, offset)),
            _ => i64::from_ne_bytes(load::<8>(bytes, offset)),
        }),
        TypeKind::Float { size } => {
            if *size == 32 {
                Value::Float(f32::from_ne_bytes(load::<4>(bytes, offset)) as f64)
            } else {
                Value::Float(f64::from_ne_bytes(load::<8>(bytes, offset)))
            }
        }
        TypeKind::Boolean => Value::Boolean(bytes[offset] != 0),
        TypeKind::Character => {
            let value = u32::from_ne_bytes(load::<4>(bytes, offset));
            Value::Character(char::from_u32(value).unwrap_or('\0'))
        }
        TypeKind::String => {
            let ptr = usize::from_ne_bytes(load::<8>(bytes, offset)) as *const u8;
            unsafe { text(ptr) }
        }
        TypeKind::Pointer { .. } => Value::Integer(usize::from_ne_bytes(load::<8>(bytes, offset)) as i64),
        TypeKind::Array { member, size } => {
            let item = layout(member);
            let step = stride(item.size, item.align);
            let mut values = Vec::new();
            for index in 0..*size as usize {
                values.push(decode(member, bytes, offset + index * step));
            }
            Value::Sequence(values)
        }
        TypeKind::Tuple { members } => {
            let mut values = Vec::new();
            for index in 0..members.len() {
                if let Some(item) = field_type(typing, index) {
                    let shift = field_offset(typing, index).unwrap_or(0) as usize;
                    values.push(decode(&item, bytes, offset + shift));
                }
            }
            Value::Sequence(values)
        }
        TypeKind::Structure(value) => {
            let mut values = Vec::new();
            for index in 0..value.members.len() {
                if let Some(item) = field_type(typing, index) {
                    let shift = field_offset(typing, index).unwrap_or(0) as usize;
                    values.push(decode(&item, bytes, offset + shift));
                }
            }
            Value::Composite(values)
        }
        TypeKind::Union(value) => {
            let mut values = Vec::new();
            for index in 0..value.members.len() {
                if let Some(item) = field_type(typing, index) {
                    values.push(decode(&item, bytes, offset));
                }
            }
            Value::Composite(values)
        }
        _ => Value::Empty,
    }
}

fn stride(size: u32, align: u8) -> usize {
    let align = align.max(1) as usize;
    let size = size as usize;
    let rest = size % align;
    if rest == 0 { size } else { size + align - rest }
}

fn load<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    let mut value = [0u8; N];
    let end = offset + N;
    if end <= bytes.len() {
        value.copy_from_slice(&bytes[offset..end]);
    }
    value
}
