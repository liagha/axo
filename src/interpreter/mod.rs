#![allow(unused)]

mod error;
mod translator;

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        combinator::{Action, Operation, Operator},
        data::memory::Arc,
        internal::{
            hash::Map,
            platform::{create_dir_all, Command, Lock},
            time::Duration,
            CompileError, InputKind, Session,
        },
        interpreter::error::ErrorKind,
        reporter::Error,
        tracker::Span,
    },
};
use crate::data::Str;

pub type InterpretError<'error> = Error<'error, ErrorKind>;

pub type Native<'error> = fn(&[Value], Span<'error>) -> Result<Value, InterpretError<'error>>;

#[cfg(unix)]
mod sys {
    pub use libc::{c_void, dlopen, dlsym, RTLD_LAZY};
}

#[cfg(windows)]
mod sys {
    pub type c_void = std::ffi::c_void;
    pub type Module = *mut c_void;
    pub type Pointer = *mut c_void;
    pub const RTLD_LAZY: i32 = 0;

    extern "system" {
        pub fn LoadLibraryA(path: *const i8) -> Module;
        pub fn GetProcAddress(module: Module, name: *const i8) -> Pointer;
    }

    pub unsafe fn dlopen(path: *const i8, _mode: i32) -> Module { LoadLibraryA(path) }
    pub unsafe fn dlsym(handle: Module, symbol: *const i8) -> Pointer { GetProcAddress(handle, symbol) }
}

pub struct Library {
    handle: *mut sys::c_void,
}

impl Library {
    pub fn load(path: &str) -> Option<Self> {
        let string = std::ffi::CString::new(path).ok()?;
        let handle = unsafe { sys::dlopen(string.as_ptr(), sys::RTLD_LAZY) };
        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
        }
    }

    pub fn symbol(&self, name: &str) -> Option<*mut sys::c_void> {
        let string = std::ffi::CString::new(name).ok()?;
        let pointer = unsafe { sys::dlsym(self.handle, string.as_ptr()) };
        if pointer.is_null() {
            None
        } else {
            Some(pointer)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Signature {
    VoidVoid,
    VoidInt64,
    VoidFloat64,
    VoidInt32,
    VoidPtr,
    VoidUint8,
    Int64Int64,
    Int64Int64PtrInt64,
    PtrUint64,
    PtrPtr,
    PtrInt64,
    Int64Ptr,
    Uint64Int64,
    Uint8Int64,
    Int32Uint8,
    Uint8Int32,
    Int64Int32,
    PtrPtrUint64,
    PtrPtrUint64Int64Int64Int64Int64,
    Int64PtrUint64,
    Int64Int64PtrUint64,
    Uint64Int64PtrUint64,
    Int64PtrInt64Int64,
    Int64Int64Int64Int64,
    Uint64Ptr,
    Uint8PtrUint64,
    BoolUint8,
    PtrPtrUint64Uint64,
    Float64Ptr,
    PtrVoid,
    BoolPtrPtr,
    BoolPtrUint64Ptr,
    BoolPtrUint64,
}

#[derive(Clone)]
pub struct Dynamic {
    pub pointer: *mut sys::c_void,
    pub signature: Signature,
}

unsafe impl Send for Dynamic {}
unsafe impl Sync for Dynamic {}

#[derive(Clone)]
pub enum NativeFunction<'error> {
    Rust(Native<'error>),
    Dynamic(Dynamic),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    Text(String),
    Sequence(Vec<Value>),
    Pointer(usize),
    Empty,
}

#[derive(Clone, Debug)]
pub enum Opcode {
    Push(Value),
    Pop,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Negate,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    LogicalXor,
    BitwiseAnd,
    BitwiseOr,
    BitwiseNot,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    Jump(usize),
    JumpTrue(usize),
    JumpFalse(usize),
    Load(usize),
    Store(usize),
    Call(usize),
    NativeCall(usize, usize),
    Return,
    Halt,
    MakeSequence(usize),
    Index,
}

#[derive(Clone, Debug)]
pub struct Instruction<'error> {
    pub opcode: Opcode,
    pub span: Span<'error>,
}

pub struct Machine<'error> {
    stack: Vec<Value>,
    frames: Vec<usize>,
    memory: Vec<Value>,
    code: Vec<Instruction<'error>>,
    natives: Vec<NativeFunction<'error>>,
    pointer: usize,
    running: bool,
}

pub fn print<'error>(arguments: &[Value], _span: Span<'error>) -> Result<Value, InterpretError<'error>> {
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            std::print!(" ");
        }
        match argument {
            Value::Integer(value) => std::print!("{}", value),
            Value::Float(value) => std::print!("{}", value),
            Value::Boolean(value) => std::print!("{}", value),
            Value::Character(value) => std::print!("{}", value),
            Value::Text(value) => std::print!("{}", value),
            Value::Sequence(value) => std::print!("{:?}", value),
            Value::Pointer(value) => std::print!("{:#x}", value),
            Value::Empty => std::print!("empty"),
        }
    }

    println!();

    Ok(Value::Empty)
}

impl<'error> Machine<'error> {
    pub fn new(code: Vec<Instruction<'error>>, capacity: usize, natives: Vec<NativeFunction<'error>>) -> Self {
        let mut bundled: Vec<NativeFunction<'error>> = vec![NativeFunction::Rust(print)];
        bundled.extend(natives);

        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            memory: vec![Value::Empty; capacity],
            code,
            natives: bundled,
            pointer: 0,
            running: false,
        }
    }

    fn error(&self, kind: ErrorKind, span: Span<'error>) -> InterpretError<'error> {
        Error::new(kind, span)
    }

    fn current(&self) -> Span<'error> {
        self.code[self.pointer.saturating_sub(1)].span
    }

    pub fn run(&mut self) -> Result<(), InterpretError<'error>> {
        self.running = true;
        while self.running && self.pointer < self.code.len() {
            self.step()?;
        }
        Ok(())
    }

    fn step(&mut self) -> Result<(), InterpretError<'error>> {
        let instruction = self.code[self.pointer].clone();
        self.pointer += 1;

        match instruction.opcode {
            Opcode::Push(value) => self.stack.push(value),
            Opcode::Pop => {
                self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, instruction.span))?;
            }
            Opcode::Add => self.add()?,
            Opcode::Subtract => self.subtract()?,
            Opcode::Multiply => self.multiply()?,
            Opcode::Divide => self.divide()?,
            Opcode::Modulus => self.modulus()?,
            Opcode::Negate => self.negate()?,
            Opcode::Equal => self.equal()?,
            Opcode::NotEqual => self.not_equal()?,
            Opcode::Less => self.less()?,
            Opcode::Greater => self.greater()?,
            Opcode::LessEqual => self.less_equal()?,
            Opcode::GreaterEqual => self.greater_equal()?,
            Opcode::LogicalAnd => self.logical_and()?,
            Opcode::LogicalOr => self.logical_or()?,
            Opcode::LogicalNot => self.logical_not()?,
            Opcode::LogicalXor => self.logical_xor()?,
            Opcode::BitwiseAnd => self.bitwise_and()?,
            Opcode::BitwiseOr => self.bitwise_or()?,
            Opcode::BitwiseNot => self.bitwise_not()?,
            Opcode::BitwiseXor => self.bitwise_xor()?,
            Opcode::ShiftLeft => self.shift_left()?,
            Opcode::ShiftRight => self.shift_right()?,
            Opcode::Jump(target) => self.jump(target)?,
            Opcode::JumpTrue(target) => self.jump_true(target)?,
            Opcode::JumpFalse(target) => self.jump_false(target)?,
            Opcode::Load(address) => self.load(address)?,
            Opcode::Store(address) => self.store(address)?,
            Opcode::Call(target) => self.call(target)?,
            Opcode::NativeCall(target, count) => self.native_call(target, count)?,
            Opcode::Return => self.finish()?,
            Opcode::Halt => self.running = false,
            Opcode::MakeSequence(size) => self.make_sequence(size)?,
            Opcode::Index => self.index()?,
        }

        Ok(())
    }

    fn add(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left + right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn subtract(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left - right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn multiply(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left * right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn divide(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                if right == 0 {
                    return Err(self.error(ErrorKind::OutOfBounds, span));
                }
                Value::Integer(left / right)
            }
            (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn modulus(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                if right == 0 {
                    return Err(self.error(ErrorKind::OutOfBounds, span));
                }
                Value::Integer(left % right)
            }
            (Value::Float(left), Value::Float(right)) => Value::Float(left % right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn negate(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Integer(value) => Value::Integer(-value),
            Value::Float(value) => Value::Float(-value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn equal(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.stack.push(Value::Boolean(left == right));
        Ok(())
    }

    fn not_equal(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.stack.push(Value::Boolean(left != right));
        Ok(())
    }

    fn less(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left < right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left < right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn greater(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left > right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left > right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn less_equal(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left <= right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left <= right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn greater_equal(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left >= right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left >= right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_and(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left && right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_or(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left || right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_not(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Boolean(value) => Value::Boolean(!value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_xor(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left ^ right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_and(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left & right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_or(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left | right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_not(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Integer(value) => Value::Integer(!value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_xor(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left ^ right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn shift_left(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left << right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn shift_right(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left >> right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn jump(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if target >= self.code.len() {
            return Err(self.error(ErrorKind::OutOfBounds, span));
        }
        self.pointer = target;
        Ok(())
    }

    fn jump_true(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let condition = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match condition {
            Value::Boolean(true) => self.jump(target)?,
            Value::Boolean(false) => {}
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }

        Ok(())
    }

    fn jump_false(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let condition = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match condition {
            Value::Boolean(false) => self.jump(target)?,
            Value::Boolean(true) => {}
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }

        Ok(())
    }

    fn load(&mut self, address: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.memory[address].clone();
        self.stack.push(value);
        Ok(())
    }

    fn store(&mut self, address: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.memory[address] = value;
        Ok(())
    }

    fn integer(&self, args: &[Value], index: usize, span: Span<'error>) -> Result<i64, InterpretError<'error>> {
        if let Some(Value::Integer(value)) = args.get(index) {
            Ok(*value)
        } else {
            Err(self.error(ErrorKind::TypeMismatch, span))
        }
    }

    fn pointer(&self, args: &[Value], index: usize, span: Span<'error>) -> Result<usize, InterpretError<'error>> {
        if let Some(Value::Pointer(value)) = args.get(index) {
            Ok(*value)
        } else {
            Err(self.error(ErrorKind::TypeMismatch, span))
        }
    }

    fn float(&self, args: &[Value], index: usize, span: Span<'error>) -> Result<f64, InterpretError<'error>> {
        if let Some(Value::Float(value)) = args.get(index) {
            Ok(*value)
        } else {
            Err(self.error(ErrorKind::TypeMismatch, span))
        }
    }

    fn boolean(&self, args: &[Value], index: usize, span: Span<'error>) -> Result<bool, InterpretError<'error>> {
        if let Some(Value::Boolean(value)) = args.get(index) {
            Ok(*value)
        } else {
            Err(self.error(ErrorKind::TypeMismatch, span))
        }
    }

    fn invoke(&self, dynamic: &Dynamic, args: &[Value], span: Span<'error>) -> Result<Value, InterpretError<'error>> {
        unsafe {
            match dynamic.signature {
                Signature::VoidVoid => {
                    let function: extern "C" fn() = std::mem::transmute(dynamic.pointer);
                    function();
                    Ok(Value::Empty)
                }
                Signature::VoidInt64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let function: extern "C" fn(i64) = std::mem::transmute(dynamic.pointer);
                    function(a0);
                    Ok(Value::Empty)
                }
                Signature::VoidFloat64 => {
                    let a0 = self.float(args, 0, span)?;
                    let function: extern "C" fn(f64) = std::mem::transmute(dynamic.pointer);
                    function(a0);
                    Ok(Value::Empty)
                }
                Signature::VoidInt32 => {
                    let a0 = self.integer(args, 0, span)? as i32;
                    let function: extern "C" fn(i32) = std::mem::transmute(dynamic.pointer);
                    function(a0);
                    Ok(Value::Empty)
                }
                Signature::VoidPtr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let function: extern "C" fn(*mut u8) = std::mem::transmute(dynamic.pointer);
                    function(a0 as *mut u8);
                    Ok(Value::Empty)
                }
                Signature::VoidUint8 => {
                    let a0 = self.integer(args, 0, span)? as u8;
                    let function: extern "C" fn(u8) = std::mem::transmute(dynamic.pointer);
                    function(a0);
                    Ok(Value::Empty)
                }
                Signature::Int64Int64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let function: extern "C" fn(i64) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0)))
                }
                Signature::Int64Int64PtrInt64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let a1 = self.pointer(args, 1, span)?;
                    let a2 = self.integer(args, 2, span)?;
                    let function: extern "C" fn(i64, *mut u8, i64) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0, a1 as *mut u8, a2)))
                }
                Signature::PtrUint64 => {
                    let a0 = self.integer(args, 0, span)? as u64;
                    let function: extern "C" fn(u64) -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function(a0) as usize))
                }
                Signature::PtrPtr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let function: extern "C" fn(*mut u8) -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function(a0 as *mut u8) as usize))
                }
                Signature::PtrInt64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let function: extern "C" fn(i64) -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function(a0) as usize))
                }
                Signature::Int64Ptr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let function: extern "C" fn(*mut u8) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0 as *mut u8)))
                }
                Signature::Uint64Int64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let function: extern "C" fn(i64) -> u64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0) as i64))
                }
                Signature::Uint8Int64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let function: extern "C" fn(i64) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0) as i64))
                }
                Signature::Int32Uint8 => {
                    let a0 = self.integer(args, 0, span)? as u8;
                    let function: extern "C" fn(u8) -> i32 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0) as i64))
                }
                Signature::Uint8Int32 => {
                    let a0 = self.integer(args, 0, span)? as i32;
                    let function: extern "C" fn(i32) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0) as i64))
                }
                Signature::Int64Int32 => {
                    let a0 = self.integer(args, 0, span)? as i32;
                    let function: extern "C" fn(i32) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0)))
                }
                Signature::PtrPtrUint64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let function: extern "C" fn(*mut u8, u64) -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function(a0 as *mut u8, a1) as usize))
                }
                Signature::PtrPtrUint64Int64Int64Int64Int64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let a2 = self.integer(args, 2, span)?;
                    let a3 = self.integer(args, 3, span)?;
                    let a4 = self.integer(args, 4, span)?;
                    let a5 = self.integer(args, 5, span)?;
                    let function: extern "C" fn(*mut u8, u64, i64, i64, i64, i64) -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function(a0 as *mut u8, a1, a2, a3, a4, a5) as usize))
                }
                Signature::Int64PtrUint64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let function: extern "C" fn(*mut u8, u64) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0 as *mut u8, a1)))
                }
                Signature::Int64Int64PtrUint64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let a1 = self.pointer(args, 1, span)?;
                    let a2 = self.integer(args, 2, span)? as u64;
                    let function: extern "C" fn(i64, *mut u8, u64) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0, a1 as *mut u8, a2)))
                }
                Signature::Uint64Int64PtrUint64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let a1 = self.pointer(args, 1, span)?;
                    let a2 = self.integer(args, 2, span)? as u64;
                    let function: extern "C" fn(i64, *mut u8, u64) -> u64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0, a1 as *mut u8, a2) as i64))
                }
                Signature::Int64PtrInt64Int64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)?;
                    let a2 = self.integer(args, 2, span)?;
                    let function: extern "C" fn(*mut u8, i64, i64) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0 as *mut u8, a1, a2)))
                }
                Signature::Int64Int64Int64Int64 => {
                    let a0 = self.integer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)?;
                    let a2 = self.integer(args, 2, span)?;
                    let function: extern "C" fn(i64, i64, i64) -> i64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0, a1, a2)))
                }
                Signature::Uint64Ptr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let function: extern "C" fn(*mut u8) -> u64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0 as *mut u8) as i64))
                }
                Signature::Uint8PtrUint64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let function: extern "C" fn(*mut u8, u64) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Integer(function(a0 as *mut u8, a1) as i64))
                }
                Signature::BoolUint8 => {
                    let a0 = self.integer(args, 0, span)? as u8;
                    let function: extern "C" fn(u8) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Boolean(function(a0) != 0))
                }
                Signature::PtrPtrUint64Uint64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let a2 = self.integer(args, 2, span)? as u64;
                    let function: extern "C" fn(*mut u8, u64, u64) -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function(a0 as *mut u8, a1, a2) as usize))
                }
                Signature::Float64Ptr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let function: extern "C" fn(*mut u8) -> f64 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Float(function(a0 as *mut u8)))
                }
                Signature::PtrVoid => {
                    let function: extern "C" fn() -> *mut u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Pointer(function() as usize))
                }
                Signature::BoolPtrPtr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.pointer(args, 1, span)?;
                    let function: extern "C" fn(*mut u8, *mut u8) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Boolean(function(a0 as *mut u8, a1 as *mut u8) != 0))
                }
                Signature::BoolPtrUint64Ptr => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let a2 = self.pointer(args, 2, span)?;
                    let function: extern "C" fn(*mut u8, u64, *mut u8) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Boolean(function(a0 as *mut u8, a1, a2 as *mut u8) != 0))
                }
                Signature::BoolPtrUint64 => {
                    let a0 = self.pointer(args, 0, span)?;
                    let a1 = self.integer(args, 1, span)? as u64;
                    let function: extern "C" fn(*mut u8, u64) -> u8 = std::mem::transmute(dynamic.pointer);
                    Ok(Value::Boolean(function(a0 as *mut u8, a1) != 0))
                }
            }
        }
    }

    fn native_call(&mut self, target: usize, count: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let function = self.natives.get(target).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?.clone();

        if self.stack.len() < count {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }

        let start = self.stack.len() - count;
        let arguments = &self.stack[start..];

        let result = match function {
            NativeFunction::Rust(function) => function(arguments, span)?,
            NativeFunction::Dynamic(dynamic) => self.invoke(&dynamic, arguments, span)?,
        };

        self.stack.truncate(start);
        self.stack.push(result);

        Ok(())
    }

    fn call(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if target >= self.code.len() {
            return Err(self.error(ErrorKind::OutOfBounds, span));
        }
        self.frames.push(self.pointer);
        self.pointer = target;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        self.pointer = self.frames.pop().ok_or_else(|| self.error(ErrorKind::InvalidFrame, span))?;
        Ok(())
    }

    fn make_sequence(&mut self, size: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if self.stack.len() < size {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }
        let start = self.stack.len() - size;
        let sequence = self.stack.drain(start..).collect();
        self.stack.push(Value::Sequence(sequence));
        Ok(())
    }

    fn index(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let position = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let target = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match (target, position) {
            (Value::Sequence(sequence), Value::Integer(index)) => {
                let index = index as usize;
                if index >= sequence.len() {
                    return Err(self.error(ErrorKind::OutOfBounds, span));
                }
                self.stack.push(sequence[index].clone());
            }
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }
        Ok(())
    }

    pub fn extract(mut self) -> Option<Value> {
        self.stack.pop()
    }
}

pub struct InterpretAction;

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
        let mut headers = Vec::new();

        for (&key, record) in session.records.iter() {
            if record.kind == InputKind::Source && record.module.is_some() {
                sources.push(key);
            } else if record.kind == InputKind::C {
                headers.push(record.location.to_string());
            }
        }
        sources.sort();

        let mut translator = translator::Translator::new();
        let mut all_analyses = Vec::new();

        for &key in &sources {
            let record = session.records.get(&key).unwrap();
            let location = record.location;
            let stem = Str::from(location.stem().unwrap().to_string());

            translator.current_module = stem;

            if let Some(analyses) = record.analyses.clone() {
                all_analyses.extend(analyses);
            }
        }

        let mut dynamic = Vec::new();

        if !headers.is_empty() {
            session.report_execute("compiling dynamic base");

            let base = session.base();
            let build = base.join("build");
            _ = create_dir_all(&build);

            let extension = if cfg!(target_os = "windows") {
                "dll"
            } else if cfg!(target_os = "macos") {
                "dylib"
            } else {
                "so"
            };

            let library = build.join(format!("lib_base.{}", extension));

            let mut command = Command::new("clang");
            command.arg("-shared").arg("-fPIC").arg("-o").arg(library.to_str().unwrap());

            for header in headers {
                command.arg(header);
            }

            let status = command.status().expect("failed to compile dynamic library");

            if status.success() {
                if let Some(instance) = Library::load(library.to_str().unwrap()) {
                    let mappings = [
                        // ... (Keep all your existing Signature mappings here)
                        ("print_integer", Signature::VoidInt64),
                        ("print_newline", Signature::VoidVoid),
                        // ...
                    ];

                    for (name, signature) in mappings {
                        if let Some(pointer) = instance.symbol(name) {
                            dynamic.push(NativeFunction::Dynamic(Dynamic {
                                pointer,
                                signature,
                            }));

                            // FIX: use dynamic.len() - 1 because Vec is 0-indexed
                            translator.native(name, dynamic.len() - 1);
                        }
                    }

                    std::mem::forget(instance);
                } else {
                    panic!("failed to load compiled dynamic library: {}", library.to_str().unwrap());
                }
            } else {
                panic!("clang failed to compile dynamic library.");
            }
        }

        let code = translator.compile(all_analyses);
        let mut machine = Machine::new(code, 1024, dynamic);

        if let Err(error) = machine.run() {
            session.errors.push(CompileError::Interpret(error.clone()));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("interpreting", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }

        ()
    }
}
