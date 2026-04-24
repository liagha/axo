mod error;
mod interpreter;
mod translator;

pub use {error::*, interpreter::*};

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        combinator::{Combinator, Operation, Operator},
        data::{memory::{null_mut, Arc}, CString, Function, Identity, Interface, Str},
        internal::{
            foreign::{CChar, CStr, CVoid},
            platform::{temp_dir, Lock, DLL_EXTENSION},
            Artifact, RecordKind, Session, SessionError,
        },
        reporter::Error,
        resolver::{Type, TypeKind},
    },
    libffi::middle::{Arg, Cif, CodePtr, Type as FfiType},
    libloading::{Library},
};

pub type InterpretError<'error> = Error<'error, ErrorKind>;
pub type DynamicFunction = Arc<dyn Fn(&[Value]) -> Result<Value, ErrorKind> + Send + Sync>;

#[derive(Clone, Copy)]
enum NativeType {
    Integer,
    Float,
    Boolean,
    Character,
    String,
    Pointer,
    Void,
    U8,
}

pub struct InterpretCombinator<'source> {
    pub core: Arc<Lock<Interpreter<'source>>>,
}

impl<'source> InterpretCombinator<'source> {
    pub fn new(core: Arc<Lock<Interpreter<'source>>>) -> Self {
        Self { core }
    }
}

impl<'source> Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for InterpretCombinator<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;
        let mut core = self.core.write().unwrap();

        let mut sources: Vec<_> = session
            .records
            .iter()
            .filter(|(_, r)| r.kind == RecordKind::Source && r.fetch(0).is_some())
            .map(|(&k, _)| k)
            .collect();
        sources.sort();

        Interpreter::execute(session, &mut core, &sources);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'source> Interpreter<'source> {
    fn value_type(typing: &Type<'source>) -> Type<'source> {
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

    fn literal(element: &crate::parser::Element<'source>) -> Option<Value> {
        match &element.kind {
            crate::parser::ElementKind::Literal(token) => match &token.kind {
                crate::scanner::TokenKind::Integer(value) => Some(Value::Integer(*value as i64)),
                crate::scanner::TokenKind::Float(value) => Some(Value::Float(f64::from(*value))),
                crate::scanner::TokenKind::Boolean(value) => Some(Value::Boolean(*value)),
                crate::scanner::TokenKind::Character(value) => Some(Value::Character(*value as char)),
                crate::scanner::TokenKind::String(value) => Some(Value::String(value.to_string())),
                _ => None,
            },
            _ => None,
        }
    }

    fn bind_values(session: &Session<'source>, core: &mut Interpreter<'source>) {
        core.values.clear();

        for symbol in session.resolver.registry.values() {
            if let crate::parser::SymbolKind::Binding(binding) = &symbol.kind {
                if let Some(value) = &binding.value {
                    if let Some(name) = binding.target.target() {
                        if let Some(value) = Self::literal(value) {
                            core.bind_value(name, value);
                        }
                    }
                }
            }
        }
    }

    fn assemble(
        session: &Session<'source>,
        core: &mut Interpreter<'source>,
        keys: &[Identity],
    ) {
        let mut sources: Vec<_> = keys
            .iter()
            .copied()
            .filter(|key| {
                session
                    .records
                    .get(key)
                    .map(|record| {
                        record.kind == RecordKind::Source
                            && record.fetch(0).is_some()
                            && record.fetch(3).is_some()
                    })
                    .unwrap_or(false)
            })
            .collect();
        sources.sort();

        let library = Self::load_library(session);

        Self::bind_values(session, core);

        for key in &sources {
            let Some(record) = session.records.get(key) else {
                continue;
            };

            let analyses = if let Some(Artifact::Analyses(a)) = record.fetch(3) {
                a.clone()
            } else {
                continue;
            };

            let Some(stem) = record.location.stem() else {
                continue;
            };

            for analysis in &analyses {
                if let AnalysisKind::Function(function) = &analysis.kind {
                    if matches!(function.interface, Interface::C) {
                        Self::bind_function(core, function, &analysis.typing, &library);
                    }
                }
            }

            let stem = Str::from(stem.to_string());
            core.modules.insert(stem, ());
            core.units.push(CompilationUnit {
                stem,
                analyses,
            });
        }

        core.compile();
    }

    pub fn process(
        session: &mut Session<'source>,
        core: &mut Interpreter<'source>,
        keys: &[Identity],
    ) {
        let start = core.code.len();

        Self::assemble(session, core, keys);

        if session.errors.is_empty() && core.code.len() > start {
            core.begin(start);

            if let Err(error) = core.run() {
                session.errors.push(SessionError::Interpret(error));
            }
        }
    }

    pub fn execute(
        session: &mut Session<'source>,
        core: &mut Interpreter<'source>,
        keys: &[Identity],
    ) {
        Self::process(session, core, keys);
    }

    pub fn evaluate(
        session: &Session<'source>,
        core: &mut Interpreter<'source>,
        key: Identity,
    ) -> Result<Option<Value>, InterpretError<'source>> {
        let Some(record) = session.records.get(&key) else {
            return Ok(None);
        };

        let Some(Artifact::Analyses(analyses)) = record.fetch(3) else {
            return Ok(None);
        };

        let Some(stem) = record.location.stem() else {
            return Ok(None);
        };

        core.reset();
        let mut keys: Vec<_> = session
            .records
            .iter()
            .filter(|(identity, record)| {
                **identity != key && record.kind == RecordKind::Source && record.fetch(0).is_some()
            })
            .map(|(&identity, _)| identity)
            .collect();
        keys.sort();
        Self::assemble(session, core, &keys);

        if !core.code.is_empty() {
            core.begin(0);
            core.run()?;
        }

        core.eval(Str::from(stem.to_string()), analyses.clone())
    }

    pub fn execute_line(
        session: &Session<'source>,
        core: &mut Interpreter<'source>,
        key: Identity,
    ) -> Result<Option<Value>, InterpretError<'source>> {
        let Some(record) = session.records.get(&key) else {
            return Ok(None);
        };

        let Some(Artifact::Analyses(analyses)) = record.fetch(3) else {
            return Ok(None);
        };

        let Some(stem) = record.location.stem() else {
            return Ok(None);
        };

        let library = Self::load_library(session);

        Self::bind_values(session, core);

        for analysis in analyses {
            if let AnalysisKind::Function(function) = &analysis.kind {
                if matches!(function.interface, Interface::C) {
                    Self::bind_function(core, function, &analysis.typing, &library);
                }
            }
        }

        core.modules.insert(Str::from(stem.to_string()), ());
        core.eval(Str::from(stem.to_string()), analyses.clone())
    }

    fn load_library(session: &Session) -> Option<Library> {
        let discard = session.get_directive(Str::from("Discard")).is_some();
        let build = if discard {
            temp_dir().join("axo").join("build")
        } else {
            session.base().join("build")
        };

        let path = build.join(format!("lib_axo.{}", DLL_EXTENSION));
        unsafe { Library::new(path).ok() }
    }

    fn bind_function(
        core: &mut Interpreter<'source>,
        function: &Function<Str<'source>, Analysis<'source>, Option<Box<Analysis<'source>>>, Option<Type<'source>>>,
        typing: &Type<'source>,
        library: &Option<Library>,
    ) {
        let name = function.target.as_str().unwrap_or_default();
        let fallback =
            || -> DynamicFunction { Arc::new(|_: &[Value]| Err(ErrorKind::OutOfBounds)) };

        let closure = if let Some(lib) = library {
            let symbol_result = unsafe { lib.get::<*mut CVoid>(name.as_bytes()) };

            if let Ok(symbol) = symbol_result {
                let pointer = *symbol;
                let mut members = Vec::with_capacity(function.members.len());

                for member in &function.members {
                    let typing = Self::value_type(&member.typing);

                    members.push(match &typing.kind {
                        TypeKind::Integer { .. } => NativeType::Integer,
                        TypeKind::Float { .. } => NativeType::Float,
                        TypeKind::Boolean => NativeType::Boolean,
                        TypeKind::Character => NativeType::Character,
                        TypeKind::String => NativeType::String,
                        TypeKind::Pointer { .. }
                        | TypeKind::Array { .. }
                        | TypeKind::Structure(_)
                        | TypeKind::Union(_) => NativeType::Pointer,
                        _ => NativeType::Pointer,
                    });
                }

                let output = match function.output.as_ref().map(|t| &t.kind) {
                    Some(TypeKind::Float { .. }) => NativeType::Float,
                    Some(TypeKind::Boolean) => NativeType::Boolean,
                    Some(TypeKind::Integer {
                             size: 8,
                             signed: false,
                         }) => NativeType::U8,
                    Some(TypeKind::String) => NativeType::String,
                    Some(TypeKind::Void) | None => NativeType::Void,
                    Some(TypeKind::Character) => NativeType::Character,
                    _ => NativeType::Integer,
                };

                build_closure(CodePtr::from_ptr(pointer), members, output)
            } else {
                fallback()
            }
        } else {
            fallback()
        };

        core.foreign.push(Foreign::Dynamic(closure));
        let index = core.foreign.len() - 1;
        core.native(typing.clone(), index);
    }
}

#[derive(Debug)]
enum FfiValue {
    I64(i64),
    F64(f64),
    U8(u8),
    U32(u32),
    Ptr(*mut CVoid),
}

fn build_closure(
    address: CodePtr,
    members: Vec<NativeType>,
    output: NativeType,
) -> DynamicFunction {
    let address = address.as_ptr() as usize;

    Arc::new(move |inputs: &[Value]| -> Result<Value, ErrorKind> {
        let address = CodePtr::from_ptr(address as *mut CVoid);

        let mut types = Vec::with_capacity(inputs.len());
        let mut values = Vec::with_capacity(inputs.len());
        let mut strings = Vec::new();

        for (input, native) in inputs.iter().zip(members.iter()) {
            match native {
                NativeType::Integer | NativeType::U8 => {
                    types.push(FfiType::i64());
                    if let Value::Integer(v) = input {
                        values.push(FfiValue::I64(*v));
                    } else {
                        values.push(FfiValue::I64(0));
                    }
                }
                NativeType::Float => {
                    types.push(FfiType::f64());
                    if let Value::Float(v) = input {
                        values.push(FfiValue::F64(*v));
                    } else {
                        values.push(FfiValue::F64(0.0));
                    }
                }
                NativeType::Boolean => {
                    types.push(FfiType::u8());
                    if let Value::Boolean(v) = input {
                        values.push(FfiValue::U8(if *v { 1 } else { 0 }));
                    } else {
                        values.push(FfiValue::U8(0));
                    }
                }
                NativeType::Character => {
                    types.push(FfiType::u32());
                    if let Value::Character(v) = input {
                        values.push(FfiValue::U32(*v as u32));
                    } else {
                        values.push(FfiValue::U32(0));
                    }
                }
                NativeType::String => {
                    types.push(FfiType::pointer());
                    if let Value::String(v) = input {
                        if let Ok(string) = CString::new(v.clone()) {
                            strings.push(string);
                            values.push(FfiValue::Ptr(
                                strings.last().unwrap().as_ptr() as *mut CVoid
                            ));
                        } else {
                            values.push(FfiValue::Ptr(null_mut()));
                        }
                    } else {
                        values.push(FfiValue::Ptr(null_mut()));
                    }
                }
                NativeType::Pointer => {
                    types.push(FfiType::pointer());
                    if let Value::Pointer(v) = input {
                        values.push(FfiValue::Ptr(*v as *mut CVoid));
                    } else {
                        values.push(FfiValue::Ptr(null_mut()));
                    }
                }
                NativeType::Void => {
                    types.push(FfiType::pointer());
                    values.push(FfiValue::Ptr(null_mut()));
                }
            }
        }

        let result = match output {
            NativeType::Float => FfiType::f64(),
            NativeType::Boolean => FfiType::u8(),
            NativeType::U8 => FfiType::u8(),
            NativeType::String => FfiType::pointer(),
            NativeType::Void => FfiType::void(),
            NativeType::Character => FfiType::u32(),
            NativeType::Integer | NativeType::Pointer => FfiType::i64(),
        };

        let args: Vec<Arg> = values
            .iter()
            .map(|v| match v {
                FfiValue::I64(x) => Arg::new(x),
                FfiValue::F64(x) => Arg::new(x),
                FfiValue::U8(x) => Arg::new(x),
                FfiValue::U32(x) => Arg::new(x),
                FfiValue::Ptr(x) => Arg::new(x),
            })
            .collect();

        let cif = Cif::new(types.into_iter(), result);

        match output {
            NativeType::Float => {
                let ret: f64 = unsafe { cif.call(address, &args) };
                Ok(Value::Float(ret))
            }
            NativeType::Boolean => {
                let ret: u8 = unsafe { cif.call(address, &args) };
                Ok(Value::Boolean(ret != 0))
            }
            NativeType::U8 => {
                let ret: u8 = unsafe { cif.call(address, &args) };
                Ok(Value::Integer(ret as i64))
            }
            NativeType::String => {
                let ret: *mut CChar = unsafe { cif.call(address, &args) };
                if ret.is_null() {
                    Ok(Value::String(String::new()))
                } else {
                    let text = unsafe { CStr::from_ptr(ret) };
                    Ok(Value::String(text.to_string_lossy().into_owned()))
                }
            }
            NativeType::Character => {
                let ret: u32 = unsafe { cif.call(address, &args) };
                Ok(Value::Character(char::from_u32(ret).unwrap_or('\0')))
            }
            NativeType::Void => {
                unsafe { cif.call::<()>(address, &args) };
                Ok(Value::Empty)
            }
            NativeType::Integer | NativeType::Pointer => {
                let ret: i64 = unsafe { cif.call(address, &args) };
                Ok(Value::Integer(ret))
            }
        }
    })
}
