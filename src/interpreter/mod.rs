#![allow(unused)]

mod error;
mod interpreter;
mod translator;

pub use {error::*, interpreter::*};

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        combinator::{Action, Operation, Operator},
        data::{
            memory::{null_mut, Arc},
            CString, Function, Identity, Interface, Str,
        },
        internal::{
            platform::{read_dir, read_to_string, Lock},
            time::Duration,
            RecordKind, Session, SessionError,
        },
        reporter::Error,
        resolver::Type,
    },
    libffi::middle::{Arg, Cif, CodePtr, Type as FfiType},
    libloading::{Library, Symbol},
    std::{
        collections::HashMap,
        ffi::{c_char, c_void, CStr},
        path::PathBuf,
    },
};

pub type InterpretError<'error> = Error<'error, ErrorKind>;
pub type DynamicFunction = Arc<dyn Fn(&[Value]) -> Result<Value, ErrorKind> + Send + Sync>;

#[derive(Clone)]
pub struct Signature {
    pub returns: String,
    pub variadic: bool,
    pub fixed: usize,
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            returns: String::from("Integer"),
            variadic: false,
            fixed: 0,
        }
    }
}

pub struct InterpretAction<'source> {
    pub core: Arc<Lock<Interpreter<'source>>>,
}

impl<'source> InterpretAction<'source> {
    pub fn new(core: Arc<Lock<Interpreter<'source>>>) -> Self {
        Self { core }
    }
}

pub fn interpret<'source>(
    session: &mut Session<'source>,
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
                        && record.module.is_some()
                        && record.analyses.is_some()
                })
                .unwrap_or(false)
        })
        .collect();
    sources.sort();

    let signatures = Interpreter::extract_signatures();
    let library = load_library(session);
    let start = core.code.len();

    for key in &sources {
        let Some(record) = session.records.get(key) else {
            continue;
        };
        let (Some(stem), Some(analyses)) = (record.location.stem(), record.analyses.clone()) else {
            continue;
        };

        for analysis in &analyses {
            if let AnalysisKind::Function(function) = &analysis.kind {
                if matches!(function.interface, Interface::C) {
                    bind_function(core, function, &library, &signatures);
                }
            }
        }

        let stem = Str::from(stem.to_string());
        core.modules.insert(stem, analyses.clone());
    }

    core.compile();

    if session.errors.is_empty() && core.code.len() > start {
        core.pointer = start;
        core.frames.clear();
        core.stack.clear();

        if let Err(error) = core.run() {
            session.errors.push(SessionError::Interpret(error));
        }
    }
}

impl<'error> Interpreter<'error> {
    fn extract_signatures() -> HashMap<String, Signature> {
        let mut map = HashMap::new();
        let mut dirs = vec![PathBuf::from(".")];

        while let Some(dir) = dirs.pop() {
            let Ok(entries) = read_dir(&dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name != ".git" && name != "target" {
                        dirs.push(path);
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("axo") {
                    Self::parse_file(&path, &mut map);
                }
            }
        }
        map
    }

    fn parse_file(path: &PathBuf, map: &mut HashMap<String, Signature>) {
        let Ok(content) = read_to_string(path) else {
            return;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("func ") {
                Self::parse_function(trimmed, map);
            }
        }
    }

    fn parse_function(line: &str, map: &mut HashMap<String, Signature>) {
        let after = &line[5..];
        let Some(paren) = after.find('(') else {
            return;
        };
        let name = after[..paren].trim().to_string();

        let mut variadic = false;
        let mut fixed = 0;

        if let Some(end) = after.find(')') {
            let args = &after[paren + 1..end];
            if args.contains("...") {
                variadic = true;
                let before = args.split("...").next().unwrap_or("");
                fixed = before.split(',').filter(|s| !s.trim().is_empty()).count();
            }
        }

        let mut returns = String::from("Empty");
        if let Some(colon) = after.rfind(':') {
            if colon > paren {
                returns = Self::parse_type(&after[colon + 1..]);
            }
        } else if let Some(arrow) = after.find("->") {
            if arrow > paren {
                returns = Self::parse_type(&after[arrow + 2..]);
            }
        }

        map.insert(name, Signature { returns, variadic, fixed });
    }

    fn parse_type(text: &str) -> String {
        let text = text.trim();
        let text = text.split_whitespace().next().unwrap_or("Empty");
        text.replace('{', "").replace(';', "")
    }
}

fn load_library(session: &Session) -> Option<Library> {
    let discard = session.get_directive(Str::from("Discard")).is_some();
    let build = if discard {
        std::env::temp_dir().join("axo").join("build")
    } else {
        session.base().join("build")
    };

    let extension = std::env::consts::DLL_EXTENSION;
    let path = build.join(format!("lib_axo.{}", extension));
    unsafe { Library::new(path).ok() }
}

fn bind_function(
    core: &mut Interpreter,
    function: &Function<Str, Analysis, Option<Box<Analysis>>, Option<Type>>,
    library: &Option<Library>,
    signatures: &HashMap<String, Signature>,
) {
    let name = function.target.as_str().unwrap_or_default();
    let fallback = || -> DynamicFunction { Arc::new(|_: &[Value]| Err(ErrorKind::OutOfBounds)) };

    let closure = if let Some(lib) = library {
        unsafe {
            if let Ok(symbol) = lib.get::<*mut c_void>(name.as_bytes()) {
                let pointer = *symbol;
                let signature = signatures.get(name).cloned().unwrap_or_default();
                build_closure(CodePtr::from_ptr(pointer), signature)
            } else {
                fallback()
            }
        }
    } else {
        fallback()
    };

    core.foreign.push(Foreign::Dynamic(closure));
    let index = core.foreign.len() - 1;
    core.native(name, index);
}

#[derive(Debug)]
enum FfiValue {
    I64(i64),
    F64(f64),
    U8(u8),
    U32(u32),
    Ptr(*mut c_void),
}

fn build_closure(address: CodePtr, signature: Signature) -> DynamicFunction {
    let addr_usize = address.as_ptr() as usize;

    Arc::new(move |inputs: &[Value]| -> Result<Value, ErrorKind> {
        let address = CodePtr::from_ptr(addr_usize as *mut c_void);

        let mut types = Vec::with_capacity(inputs.len());
        let mut values = Vec::with_capacity(inputs.len());
        let mut strings = Vec::new();

        for input in inputs {
            match input {
                Value::Integer(v) => {
                    types.push(FfiType::i64());
                    values.push(FfiValue::I64(*v));
                }
                Value::Float(v) => {
                    types.push(FfiType::f64());
                    values.push(FfiValue::F64(*v));
                }
                Value::Boolean(v) => {
                    types.push(FfiType::u8());
                    values.push(FfiValue::U8(if *v { 1 } else { 0 }));
                }
                Value::Character(v) => {
                    types.push(FfiType::u32());
                    values.push(FfiValue::U32(*v as u32));
                }
                Value::Text(v) => {
                    types.push(FfiType::pointer());
                    if let Ok(string) = CString::new(v.clone()) {
                        strings.push(string);
                        values.push(FfiValue::Ptr(strings.last().unwrap().as_ptr() as *mut c_void));
                    } else {
                        values.push(FfiValue::Ptr(null_mut()));
                    }
                }
                Value::Pointer(v) => {
                    types.push(FfiType::pointer());
                    values.push(FfiValue::Ptr(*v as *mut c_void));
                }
                Value::Structure(fields) => {
                    if let Some(Value::Float(f)) = fields.first() {
                        types.push(FfiType::f64());
                        values.push(FfiValue::F64(*f));
                    } else {
                        types.push(FfiType::pointer());
                        values.push(FfiValue::Ptr(null_mut()));
                    }
                }
                _ => {
                    types.push(FfiType::pointer());
                    values.push(FfiValue::Ptr(null_mut()));
                }
            }
        }

        let rtype = match signature.returns.as_str() {
            "Float" => FfiType::f64(),
            "Boolean" | "UInt8" => FfiType::u8(),
            "String" => FfiType::pointer(),
            "Empty" => FfiType::void(),
            "Character" => FfiType::u32(),
            _ => FfiType::i64(),
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

        let cif = Cif::new(types.into_iter(), rtype);

        unsafe {
            match signature.returns.as_str() {
                "Float" => {
                    let ret: f64 = cif.call(address, &args);
                    Ok(Value::Float(ret))
                }
                "Boolean" => {
                    let ret: u8 = cif.call(address, &args);
                    Ok(Value::Boolean(ret != 0))
                }
                "UInt8" => {
                    let ret: u8 = cif.call(address, &args);
                    Ok(Value::Integer(ret as i64))
                }
                "String" => {
                    let ret: *mut c_char = cif.call(address, &args);
                    if ret.is_null() {
                        Ok(Value::Text(String::new()))
                    } else {
                        let text = CStr::from_ptr(ret);
                        Ok(Value::Text(text.to_string_lossy().into_owned()))
                    }
                }
                "Character" => {
                    let ret: u32 = cif.call(address, &args);
                    Ok(Value::Character(char::from_u32(ret).unwrap_or('\0')))
                }
                "Empty" => {
                    cif.call::<()>(address, &args);
                    Ok(Value::Empty)
                }
                _ => {
                    let ret: i64 = cif.call(address, &args);
                    Ok(Value::Integer(ret))
                }
            }
        }
    })
}

impl<'source> Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for InterpretAction<'source>
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;
        let mut core = self.core.write().unwrap();

        let initial = session.errors.len();
        session.report_start("interpreting");

        let mut sources: Vec<_> = session
            .records
            .iter()
            .filter(|(_, r)| r.kind == RecordKind::Source && r.module.is_some())
            .map(|(&k, _)| k)
            .collect();
        sources.sort();
        interpret(session, &mut core, &sources);

        let duration = Duration::from_nanos(session.timer.lap().unwrap_or_default());
        session.report_finish("interpreting", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}
