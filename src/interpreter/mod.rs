#![allow(unused)]

mod error;
mod translator;
mod interpreter;

use interpreter::*;
use {
    crate::{
        analyzer::{AnalysisKind},
        combinator::{Action, Operation, Operator},
        data::{memory::Arc, CString, Interface, Str},
        internal::{
            platform::Lock,
            time::Duration,
            CompileError, InputKind, Session,
        },
        interpreter::error::ErrorKind,
        reporter::Error,
    },
};

pub type InterpretError<'error> = Error<'error, ErrorKind>;

pub struct InterpretAction;

#[repr(C)]
pub struct FfiType {
    pub size: usize,
    pub alignment: u16,
    pub type_: u16,
    pub elements: *mut *mut FfiType,
}

#[repr(C)]
pub struct FfiCif {
    pub abi: u32,
    pub nargs: u32,
    pub arg_types: *mut *mut FfiType,
    pub rtype: *mut FfiType,
    pub bytes: u32,
    pub flags: u32,
}

#[cfg(all(unix, target_arch = "x86_64"))]
const FFI_DEFAULT_ABI: u32 = 2;
#[cfg(all(unix, target_arch = "aarch64"))]
const FFI_DEFAULT_ABI: u32 = 1;
#[cfg(windows)]
const FFI_DEFAULT_ABI: u32 = 1;
#[cfg(not(any(all(unix, target_arch = "x86_64"), all(unix, target_arch = "aarch64"), windows)))]
const FFI_DEFAULT_ABI: u32 = 2;

struct LibFfi {
    prep_cif: extern "C" fn(*mut FfiCif, u32, u32, *mut FfiType, *mut *mut FfiType) -> i32,
    prep_cif_var: Option<extern "C" fn(*mut FfiCif, u32, u32, u32, *mut FfiType, *mut *mut FfiType) -> i32>,
    call: extern "C" fn(*mut FfiCif, extern "C" fn(), *mut sys::c_void, *mut *mut sys::c_void),
    type_sint64: *mut FfiType,
    type_double: *mut FfiType,
    type_pointer: *mut FfiType,
    type_uint8: *mut FfiType,
    type_uint32: *mut FfiType,
    type_void: *mut FfiType,
}

unsafe impl Send for LibFfi {}
unsafe impl Sync for LibFfi {}

enum FfiArg {
    Sint64(i64),
    Double(f64),
    Pointer(*mut sys::c_void),
    Uint8(u8),
    Uint32(u32),
}

impl InterpretAction {
    fn extract_c_signatures() -> std::collections::HashMap<String, (String, bool, usize)> {
        let mut map = std::collections::HashMap::new();
        let mut dirs = vec![std::path::PathBuf::from(".")];
        while let Some(dir) = dirs.pop() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if dir_name != ".git" && dir_name != "target" {
                            dirs.push(path);
                        }
                    } else if path.extension().and_then(|s| s.to_str()) == Some("axo") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with("func ") {
                                    let after_func = &trimmed[5..];
                                    if let Some(paren_idx) = after_func.find('(') {
                                        let name = after_func[..paren_idx].trim().to_string();

                                        let mut is_var = false;
                                        let mut fixed_args = 0;

                                        if let Some(paren_end) = after_func.find(')') {
                                            let args_str = &after_func[paren_idx + 1..paren_end];
                                            if args_str.contains("...") {
                                                is_var = true;
                                                let before_dots = args_str.split("...").next().unwrap_or("");
                                                fixed_args = before_dots.split(',').filter(|s| !s.trim().is_empty()).count();
                                            }
                                        }

                                        let mut ret_type = "Empty".to_string();
                                        if let Some(colon_idx) = after_func.rfind(':') {
                                            if colon_idx > paren_idx {
                                                let type_str = after_func[colon_idx + 1..].trim();
                                                let type_str = type_str.split_whitespace().next().unwrap_or("Empty");
                                                ret_type = type_str.replace('{', "").replace(';', "");
                                            }
                                        } else if let Some(arrow_idx) = after_func.find("->") {
                                            if arrow_idx > paren_idx {
                                                let type_str = after_func[arrow_idx + 2..].trim();
                                                let type_str = type_str.split_whitespace().next().unwrap_or("Empty");
                                                ret_type = type_str.replace('{', "").replace(';', "");
                                            }
                                        }
                                        map.insert(name, (ret_type, is_var, fixed_args));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        map
    }
}

impl<'source> Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for InterpretAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        let initial = session.errors.len();
        session.report_start("interpreting");

        let mut sources = Vec::new();

        for (&key, record) in session.records.iter() {
            if record.kind == InputKind::Source && record.module.is_some() {
                sources.push(key);
            }
        }
        sources.sort();

        let mut vm = Machine::new(1024);

        let libffi_opt = Library::load("libffi.so")
            .or_else(|| Library::load("libffi.so.8"))
            .or_else(|| Library::load("libffi.so.7"))
            .or_else(|| Library::load("libffi.dylib"))
            .or_else(|| Library::load("ffi.dll"));

        let ffi = libffi_opt.and_then(|lib| {
            unsafe {
                let prep_cif_var_ptr = lib.symbol("ffi_prep_cif_var");
                Some(Arc::new(LibFfi {
                    prep_cif: std::mem::transmute(lib.symbol("ffi_prep_cif")?),
                    prep_cif_var: prep_cif_var_ptr.map(|p| std::mem::transmute(p)),
                    call: std::mem::transmute(lib.symbol("ffi_call")?),
                    type_sint64: lib.symbol("ffi_type_sint64")? as *mut FfiType,
                    type_double: lib.symbol("ffi_type_double")? as *mut FfiType,
                    type_pointer: lib.symbol("ffi_type_pointer")? as *mut FfiType,
                    type_uint8: lib.symbol("ffi_type_uint8")? as *mut FfiType,
                    type_uint32: lib.symbol("ffi_type_uint32")? as *mut FfiType,
                    type_void: lib.symbol("ffi_type_void")? as *mut FfiType,
                }))
            }
        });

        for &key in &sources {
            if let Some(record) = session.records.get(&key) {
                let location = record.location;
                if let Some(stem) = location.stem() {
                    let text = Str::from(stem.to_string());
                    if let Some(analyses) = record.analyses.clone() {
                        vm.modules.insert(text, analyses);
                    }
                }
            }
        }

        let modules: Vec<_> = vm.modules.values().flat_map(|items| items.iter()).cloned().collect();
        let signatures = Self::extract_c_signatures();

        for analysis in &modules {
            if let AnalysisKind::Function(function) = &analysis.kind {
                if matches!(function.interface, Interface::C) {
                    let name = function.target.as_str().unwrap_or_default();

                    if let Ok(string) = CString::new(name) {
                        let pointer = unsafe {
                            libc::dlsym(libc::RTLD_DEFAULT, string.as_ptr())
                        };

                        if !pointer.is_null() && ffi.is_some() {
                            let ffi_clone = ffi.clone().unwrap();
                            let address = pointer as usize;
                            let name_str = name.to_string();
                            let (ret_type, is_var, fixed_args) = signatures.get(&name_str).cloned().unwrap_or_else(|| ("Integer".to_string(), false, 0));

                            let execute = Arc::new(move |inputs: &[Value]| -> Result<Value, ErrorKind> {
                                let mut ffi_args = Vec::with_capacity(inputs.len());
                                let mut arg_types = Vec::with_capacity(inputs.len());
                                let mut c_strings = Vec::new();

                                for input in inputs {
                                    match input {
                                        Value::Integer(v) => {
                                            arg_types.push(ffi_clone.type_sint64);
                                            ffi_args.push(FfiArg::Sint64(*v));
                                        }
                                        Value::Float(v) => {
                                            arg_types.push(ffi_clone.type_double);
                                            ffi_args.push(FfiArg::Double(*v));
                                        }
                                        Value::Boolean(v) => {
                                            arg_types.push(ffi_clone.type_uint8);
                                            ffi_args.push(FfiArg::Uint8(if *v { 1 } else { 0 }));
                                        }
                                        Value::Character(v) => {
                                            arg_types.push(ffi_clone.type_uint32);
                                            ffi_args.push(FfiArg::Uint32(*v as u32));
                                        }
                                        Value::Text(v) => {
                                            arg_types.push(ffi_clone.type_pointer);
                                            if let Ok(c_str) = CString::new(v.clone()) {
                                                c_strings.push(c_str);
                                                ffi_args.push(FfiArg::Pointer(c_strings.last().unwrap().as_ptr() as *mut sys::c_void));
                                            } else {
                                                ffi_args.push(FfiArg::Pointer(std::ptr::null_mut()));
                                            }
                                        }
                                        Value::Pointer(v) => {
                                            arg_types.push(ffi_clone.type_pointer);
                                            ffi_args.push(FfiArg::Pointer(*v as *mut sys::c_void));
                                        }
                                        Value::Structure(fields) => {
                                            if let Some(Value::Float(f)) = fields.get(0) {
                                                arg_types.push(ffi_clone.type_double);
                                                ffi_args.push(FfiArg::Double(*f));
                                            } else {
                                                arg_types.push(ffi_clone.type_pointer);
                                                ffi_args.push(FfiArg::Pointer(std::ptr::null_mut()));
                                            }
                                        }
                                        _ => {
                                            arg_types.push(ffi_clone.type_pointer);
                                            ffi_args.push(FfiArg::Pointer(std::ptr::null_mut()));
                                        }
                                    }
                                }

                                let mut arg_values = Vec::with_capacity(ffi_args.len());
                                for arg in &mut ffi_args {
                                    let ptr = match arg {
                                        FfiArg::Sint64(v) => v as *mut _ as *mut sys::c_void,
                                        FfiArg::Double(v) => v as *mut _ as *mut sys::c_void,
                                        FfiArg::Pointer(v) => v as *mut _ as *mut sys::c_void,
                                        FfiArg::Uint8(v) => v as *mut _ as *mut sys::c_void,
                                        FfiArg::Uint32(v) => v as *mut _ as *mut sys::c_void,
                                    };
                                    arg_values.push(ptr);
                                }

                                let mut cif: FfiCif = unsafe { std::mem::zeroed() };
                                let rtype = match ret_type.as_str() {
                                    "Float" => ffi_clone.type_double,
                                    "Boolean" => ffi_clone.type_uint8,
                                    "UInt8" => ffi_clone.type_uint8,
                                    "String" => ffi_clone.type_pointer,
                                    "Empty" => ffi_clone.type_void,
                                    "Character" => ffi_clone.type_uint32,
                                    _ => ffi_clone.type_sint64,
                                };

                                unsafe {
                                    let status = if is_var && ffi_clone.prep_cif_var.is_some() {
                                        let nfixed = std::cmp::min(fixed_args as u32, arg_types.len() as u32);
                                        ffi_clone.prep_cif_var.unwrap()(
                                            &mut cif,
                                            FFI_DEFAULT_ABI,
                                            nfixed,
                                            arg_types.len() as u32,
                                            rtype,
                                            arg_types.as_mut_ptr(),
                                        )
                                    } else {
                                        (ffi_clone.prep_cif)(
                                            &mut cif,
                                            FFI_DEFAULT_ABI,
                                            arg_types.len() as u32,
                                            rtype,
                                            arg_types.as_mut_ptr(),
                                        )
                                    };

                                    if status != 0 {
                                        return Err(ErrorKind::TypeMismatch);
                                    }

                                    let func: extern "C" fn() = std::mem::transmute(address);

                                    if ret_type == "Float" {
                                        let mut ret: f64 = 0.0;
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        Ok(Value::Float(ret))
                                    } else if ret_type == "Boolean" {
                                        let mut ret: u8 = 0;
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        Ok(Value::Boolean(ret != 0))
                                    } else if ret_type == "UInt8" {
                                        let mut ret: u8 = 0;
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        Ok(Value::Integer(ret as i64))
                                    } else if ret_type == "String" {
                                        let mut ret: *mut sys::c_void = std::ptr::null_mut();
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        if ret.is_null() {
                                            Ok(Value::Text(String::new()))
                                        } else {
                                            let c_str = std::ffi::CStr::from_ptr(ret as *const i8);
                                            Ok(Value::Text(c_str.to_string_lossy().into_owned()))
                                        }
                                    } else if ret_type == "Character" {
                                        let mut ret: u32 = 0;
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        Ok(Value::Character(std::char::from_u32(ret).unwrap_or('\0')))
                                    } else if ret_type == "Empty" {
                                        let mut ret: i64 = 0;
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        Ok(Value::Empty)
                                    } else {
                                        let mut ret: i64 = 0;
                                        (ffi_clone.call)(&mut cif, func, &mut ret as *mut _ as *mut sys::c_void, arg_values.as_mut_ptr());
                                        Ok(Value::Integer(ret))
                                    }
                                }
                            });

                            vm.foreign.push(Foreign::Dynamic(execute));
                            vm.native(name, vm.foreign.len() - 1);
                        } else {
                            let execute = Arc::new(move |_: &[Value]| -> Result<Value, ErrorKind> {
                                Err(ErrorKind::OutOfBounds)
                            });
                            vm.foreign.push(Foreign::Dynamic(execute));
                            vm.native(name, vm.foreign.len() - 1);
                        }
                    } else {
                        let execute = Arc::new(move |_: &[Value]| -> Result<Value, ErrorKind> {
                            Err(ErrorKind::OutOfBounds)
                        });
                        vm.foreign.push(Foreign::Dynamic(execute));
                        vm.native(name, vm.foreign.len() - 1);
                    }
                }
            }
        }

        vm.compile();
        let entry = vm.address("main");

        if session.errors.is_empty() {
            if let Some(address) = entry {
                vm.pointer = address;
            }

            vm.frames.clear();

            if let Err(error) = vm.run() {
                if !matches!(error.kind, ErrorKind::InvalidFrame) {
                    session.errors.push(CompileError::Interpret(error.clone()));
                }
            }
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap_or_default());
        session.report_finish("interpreting", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}