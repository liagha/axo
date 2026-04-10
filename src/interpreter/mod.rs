#![allow(unused)]

mod error;
mod translator;
mod interpreter;

pub use {
    error::ErrorKind,
    interpreter::*,
};

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        combinator::{Action, Operation, Operator},
        data::Identity,
        data::{
            memory::{
                Arc, transmute, zeroed,
                null_mut,
            },
            Function, CString, Interface, Str
        },
        internal::{
            platform::{
                Lock,
                read_dir, read_to_string,
            },
            time::Duration,
            CompileError, InputKind, Session,
        },
        resolver::Type,
        reporter::Error,
    },
    std::{
        collections::HashMap,
        ffi::{c_void, CStr},
        path::PathBuf,
    },
};

pub type InterpretError<'error> = Error<'error, ErrorKind>;
pub type DynamicFunction = Arc<dyn Fn(&[Value]) -> Result<Value, ErrorKind> + Send + Sync>;

#[repr(C)]
pub struct ForeignType {
    pub size: usize,
    pub alignment: u16,
    pub kind: u16,
    pub elements: *mut *mut ForeignType,
}

#[repr(C)]
pub struct ForeignCall {
    pub abi: u32,
    pub nargs: u32,
    pub arg_types: *mut *mut ForeignType,
    pub rtype: *mut ForeignType,
    pub bytes: u32,
    pub flags: u32,
}

#[cfg(all(unix, target_arch = "x86_64"))]
const DEFAULT_ABI: u32 = 2;
#[cfg(all(unix, target_arch = "aarch64"))]
const DEFAULT_ABI: u32 = 1;
#[cfg(windows)]
const DEFAULT_ABI: u32 = 1;
#[cfg(not(any(all(unix, target_arch = "x86_64"), all(unix, target_arch = "aarch64"), windows)))]
const DEFAULT_ABI: u32 = 2;

struct Api {
    prep_call: extern "C" fn(*mut ForeignCall, u32, u32, *mut ForeignType, *mut *mut ForeignType) -> i32,
    prep_var: Option<extern "C" fn(*mut ForeignCall, u32, u32, u32, *mut ForeignType, *mut *mut ForeignType) -> i32>,
    call: extern "C" fn(*mut ForeignCall, extern "C" fn(), *mut c_void, *mut *mut c_void),
    sint64: *mut ForeignType,
    double: *mut ForeignType,
    pointer: *mut ForeignType,
    uint8: *mut ForeignType,
    uint32: *mut ForeignType,
    void: *mut ForeignType,
}

unsafe impl Send for Api {}
unsafe impl Sync for Api {}

enum ForeignValue {
    Sint64(i64),
    Double(f64),
    Pointer(*mut c_void),
    Uint8(u8),
    Uint32(u32),
}

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
                    record.kind == InputKind::Source
                        && record.module.is_some()
                        && record.analyses.is_some()
                })
                .unwrap_or(false)
        })
        .collect();
    sources.sort();

    let api = load_api();
    let signatures = Interpreter::extract_signatures();
    let handle = load_shared(session);
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
                    bind_function(core, function, handle, &api, &signatures);
                }
            }
        }

        let stem = Str::from(stem.to_string());
        core.modules.insert(stem, analyses.clone());
        core.extend(stem, analyses);
    }

    if session.errors.is_empty() && core.code.len() > start {
        core.pointer = start;
        core.frames.clear();
        core.stack.clear();

        if let Err(error) = core.run() {
            session.errors.push(CompileError::Interpret(error));
        }
    }
}

impl<'error> Interpreter<'error> {
    fn extract_signatures() -> HashMap<String, Signature> {
        let mut map = HashMap::new();
        let mut dirs = vec![PathBuf::from(".")];

        while let Some(dir) = dirs.pop() {
            let Ok(entries) = read_dir(&dir) else { continue };

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
        let Ok(content) = read_to_string(path) else { return };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("func ") {
                Self::parse_function(trimmed, map);
            }
        }
    }

    fn parse_function(line: &str, map: &mut HashMap<String, Signature>) {
        let after = &line[5..];
        let Some(paren) = after.find('(') else { return };
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

fn load_library(names: &[&str]) -> Option<Library> {
    for name in names {
        if let Some(lib) = Library::load(name) {
            return Some(lib);
        }
    }
    None
}

fn load_api() -> Option<Arc<Api>> {
    let names = ["libffi.so", "libffi.so.8", "libffi.so.7", "libffi.dylib", "ffi.dll"];
    let lib = load_library(&names)?;

    unsafe {
        Some(Arc::new(Api {
            prep_call: transmute(lib.symbol("ffi_prep_cif")?),
            prep_var: lib.symbol("ffi_prep_cif_var").map(|p| transmute(p)),
            call: transmute(lib.symbol("ffi_call")?),
            sint64: lib.symbol("ffi_type_sint64")? as *mut ForeignType,
            double: lib.symbol("ffi_type_double")? as *mut ForeignType,
            pointer: lib.symbol("ffi_type_pointer")? as *mut ForeignType,
            uint8: lib.symbol("ffi_type_uint8")? as *mut ForeignType,
            uint32: lib.symbol("ffi_type_uint32")? as *mut ForeignType,
            void: lib.symbol("ffi_type_void")? as *mut ForeignType,
        }))
    }
}

fn load_shared(session: &Session) -> *mut c_void {
    let base = session.base();
    let build = base.join("build");
    let library = build.join("lib_axo.so");
    let path = library.to_str().unwrap_or_default();
    let string = CString::new(path).unwrap_or_default();

    unsafe { libc::dlopen(string.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL) }
}

fn bind_function(
    core: &mut Interpreter,
    function: &Function<Str, Analysis, Option<Box<Analysis>>, Option<Type>>,
    handle: *mut c_void,
    api: &Option<Arc<Api>>,
    signatures: &HashMap<String, Signature>,
) {
    let name = function.target.as_str().unwrap_or_default();
    let fallback = || -> DynamicFunction { Arc::new(|_: &[Value]| Err(ErrorKind::OutOfBounds)) };

    let closure = if let Ok(string) = CString::new(name) {
        let pointer = unsafe {
            if handle.is_null() {
                null_mut()
            } else {
                libc::dlsym(handle, string.as_ptr())
            }
        };

        if !pointer.is_null() && api.is_some() {
            let address = pointer as usize;
            let signature = signatures.get(name).cloned().unwrap_or_default();
            build_closure(api.as_ref().unwrap().clone(), address, signature)
        } else {
            fallback()
        }
    } else {
        fallback()
    };

    core.foreign.push(Foreign::Dynamic(closure));
    let index = core.foreign.len() - 1;
    core.native(name, index);
}

fn build_closure(api: Arc<Api>, address: usize, signature: Signature) -> DynamicFunction {
    Arc::new(move |inputs: &[Value]| -> Result<Value, ErrorKind> {
        let mut args = Vec::with_capacity(inputs.len());
        let mut types = Vec::with_capacity(inputs.len());
        let mut strings = Vec::new();

        for input in inputs {
            match input {
                Value::Integer(v) => {
                    types.push(api.sint64);
                    args.push(ForeignValue::Sint64(*v));
                }
                Value::Float(v) => {
                    types.push(api.double);
                    args.push(ForeignValue::Double(*v));
                }
                Value::Boolean(v) => {
                    types.push(api.uint8);
                    args.push(ForeignValue::Uint8(if *v { 1 } else { 0 }));
                }
                Value::Character(v) => {
                    types.push(api.uint32);
                    args.push(ForeignValue::Uint32(*v as u32));
                }
                Value::Text(v) => {
                    types.push(api.pointer);
                    if let Ok(string) = CString::new(v.clone()) {
                        strings.push(string);
                        args.push(ForeignValue::Pointer(strings.last().unwrap().as_ptr() as *mut c_void));
                    } else {
                        args.push(ForeignValue::Pointer(null_mut()));
                    }
                }
                Value::Pointer(v) => {
                    types.push(api.pointer);
                    args.push(ForeignValue::Pointer(*v as *mut c_void));
                }
                Value::Structure(fields) => {
                    if let Some(Value::Float(f)) = fields.first() {
                        types.push(api.double);
                        args.push(ForeignValue::Double(*f));
                    } else {
                        types.push(api.pointer);
                        args.push(ForeignValue::Pointer(null_mut()));
                    }
                }
                _ => {
                    types.push(api.pointer);
                    args.push(ForeignValue::Pointer(null_mut()));
                }
            }
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in &mut args {
            let pointer = match arg {
                ForeignValue::Sint64(v) => v as *mut _ as *mut c_void,
                ForeignValue::Double(v) => v as *mut _ as *mut c_void,
                ForeignValue::Pointer(v) => v as *mut _ as *mut c_void,
                ForeignValue::Uint8(v) => v as *mut _ as *mut c_void,
                ForeignValue::Uint32(v) => v as *mut _ as *mut c_void,
            };
            values.push(pointer);
        }

        let mut call: ForeignCall = unsafe { zeroed() };
        let rtype = match signature.returns.as_str() {
            "Float" => api.double,
            "Boolean" | "UInt8" => api.uint8,
            "String" => api.pointer,
            "Empty" => api.void,
            "Character" => api.uint32,
            _ => api.sint64,
        };

        unsafe {
            let status = if signature.variadic && api.prep_var.is_some() {
                let fixed = u32::min(signature.fixed as u32, types.len() as u32);
                api.prep_var.unwrap()(
                    &mut call,
                    DEFAULT_ABI,
                    fixed,
                    types.len() as u32,
                    rtype,
                    types.as_mut_ptr(),
                )
            } else {
                (api.prep_call)(
                    &mut call,
                    DEFAULT_ABI,
                    types.len() as u32,
                    rtype,
                    types.as_mut_ptr(),
                )
            };

            if status != 0 {
                return Err(ErrorKind::TypeMismatch);
            }

            let function: extern "C" fn() = transmute(address);

            match signature.returns.as_str() {
                "Float" => {
                    let mut ret: f64 = 0.0;
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
                    Ok(Value::Float(ret))
                }
                "Boolean" => {
                    let mut ret: u8 = 0;
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
                    Ok(Value::Boolean(ret != 0))
                }
                "UInt8" => {
                    let mut ret: u8 = 0;
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
                    Ok(Value::Integer(ret as i64))
                }
                "String" => {
                    let mut ret: *mut c_void = null_mut();
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
                    if ret.is_null() {
                        Ok(Value::Text(String::new()))
                    } else {
                        let text = CStr::from_ptr(ret as *const i8);
                        Ok(Value::Text(text.to_string_lossy().into_owned()))
                    }
                }
                "Character" => {
                    let mut ret: u32 = 0;
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
                    Ok(Value::Character(char::from_u32(ret).unwrap_or('\0')))
                }
                "Empty" => {
                    let mut ret: i64 = 0;
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
                    Ok(Value::Empty)
                }
                _ => {
                    let mut ret: i64 = 0;
                    (api.call)(&mut call, function, &mut ret as *mut _ as *mut c_void, values.as_mut_ptr());
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
            .filter(|(_, r)| r.kind == InputKind::Source && r.module.is_some())
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
