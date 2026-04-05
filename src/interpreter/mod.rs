#![allow(unused)]

mod error;

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

pub struct Translator<'error> {
    pub code: Vec<Instruction<'error>>,
    memory: usize,
    bindings: Map<String, usize>,
    natives: Map<String, usize>,
    loops: Vec<(usize, Vec<usize>)>,
    functions: Map<String, usize>,
    calls: Vec<(usize, String)>,
}

impl<'error> Translator<'error> {
    pub fn new() -> Self {
        let mut natives = Map::new();
        natives.insert("print".to_string(), 0);

        Self {
            code: Vec::new(),
            memory: 0,
            bindings: Map::new(),
            natives,
            loops: Vec::new(),
            functions: Map::new(),
            calls: Vec::new(),
        }
    }

    pub fn native(&mut self, identifier: &str, index: usize) {
        self.natives.insert(identifier.to_string(), index);
    }

    fn emit(&mut self, opcode: Opcode, span: Span<'error>) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch(&mut self, position: usize, opcode: Opcode) {
        self.code[position].opcode = opcode;
    }

    pub fn compile(mut self, nodes: Vec<Analysis<'error>>) -> Vec<Instruction<'error>> {
        for node in nodes {
            self.walk(node);
        }

        if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
            self.emit(Opcode::Halt, span);
        }

        for (position, target) in self.calls {
            if let Some(address) = self.functions.get(&target) {
                self.code[position].opcode = Opcode::Call(*address);
            }
        }

        self.code
    }

    pub fn walk(&mut self, node: Analysis<'error>) {
        let span = node.span;
        match node.kind {
            AnalysisKind::Integer { value, .. } => {
                self.emit(Opcode::Push(Value::Integer(value as i64)), span);
            }
            AnalysisKind::Float { value, .. } => {
                self.emit(Opcode::Push(Value::Float(f64::from(value))), span);
            }
            AnalysisKind::Boolean { value } => {
                self.emit(Opcode::Push(Value::Boolean(value)), span);
            }
            AnalysisKind::Character { value } => {
                self.emit(Opcode::Push(Value::Character(value as char)), span);
            }
            AnalysisKind::String { value } => {
                self.emit(Opcode::Push(Value::Text(value.to_string())), span);
            }
            AnalysisKind::Array(elements) => {
                let size = elements.len();
                for element in elements {
                    self.walk(element);
                }
                self.emit(Opcode::MakeSequence(size), span);
            }
            AnalysisKind::Tuple(elements) => {
                let size = elements.len();
                for element in elements {
                    self.walk(element);
                }
                self.emit(Opcode::MakeSequence(size), span);
            }
            AnalysisKind::Negate(value) => {
                self.walk(*value);
                self.emit(Opcode::Negate, span);
            }
            AnalysisKind::Add(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Add, span);
            }
            AnalysisKind::Subtract(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Subtract, span);
            }
            AnalysisKind::Multiply(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Multiply, span);
            }
            AnalysisKind::Divide(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Divide, span);
            }
            AnalysisKind::Modulus(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Modulus, span);
            }
            AnalysisKind::LogicalAnd(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::LogicalAnd, span);
            }
            AnalysisKind::LogicalOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::LogicalOr, span);
            }
            AnalysisKind::LogicalNot(operand) => {
                self.walk(*operand);
                self.emit(Opcode::LogicalNot, span);
            }
            AnalysisKind::LogicalXOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::LogicalXor, span);
            }
            AnalysisKind::BitwiseAnd(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::BitwiseAnd, span);
            }
            AnalysisKind::BitwiseOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::BitwiseOr, span);
            }
            AnalysisKind::BitwiseNot(operand) => {
                self.walk(*operand);
                self.emit(Opcode::BitwiseNot, span);
            }
            AnalysisKind::BitwiseXOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::BitwiseXor, span);
            }
            AnalysisKind::ShiftLeft(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::ShiftLeft, span);
            }
            AnalysisKind::ShiftRight(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::ShiftRight, span);
            }
            AnalysisKind::Equal(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Equal, span);
            }
            AnalysisKind::NotEqual(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::NotEqual, span);
            }
            AnalysisKind::Less(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Less, span);
            }
            AnalysisKind::LessOrEqual(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::LessEqual, span);
            }
            AnalysisKind::Greater(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::Greater, span);
            }
            AnalysisKind::GreaterOrEqual(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::GreaterEqual, span);
            }
            AnalysisKind::Index(index) => {
                self.walk(*index.target);
                for member in index.members {
                    self.walk(member);
                    self.emit(Opcode::Index, span);
                }
            }
            AnalysisKind::Invoke(invoke) => {
                let count = invoke.members.len();
                for member in invoke.members {
                    self.walk(member);
                }
                let target = invoke.target.to_string();
                if let Some(position) = self.natives.get(&target) {
                    self.emit(Opcode::NativeCall(*position, count), span);
                } else if let Some(address) = self.functions.get(&target) {
                    self.emit(Opcode::Call(*address), span);
                } else {
                    let position = self.code.len();
                    self.emit(Opcode::Call(0), span);
                    self.calls.push((position, target));
                }
            }
            AnalysisKind::Block(statements) => {
                for statement in statements {
                    self.walk(statement);
                }
            }
            AnalysisKind::Conditional(condition, truthy, falsy) => {
                self.walk(*condition);
                let position = self.code.len();
                self.emit(Opcode::JumpFalse(0), span);
                self.walk(*truthy);

                if let Some(alternative) = falsy {
                    let bypass = self.code.len();
                    self.emit(Opcode::Jump(0), span);
                    self.patch(position, Opcode::JumpFalse(self.code.len()));
                    self.walk(*alternative);
                    self.patch(bypass, Opcode::Jump(self.code.len()));
                } else {
                    self.patch(position, Opcode::JumpFalse(self.code.len()));
                }
            }
            AnalysisKind::While(condition, body) => {
                let start = self.code.len();
                self.walk(*condition);
                let position = self.code.len();
                self.emit(Opcode::JumpFalse(0), span);

                self.loops.push((start, Vec::new()));
                self.walk(*body);
                self.emit(Opcode::Jump(start), span);

                let (_, breaks) = self.loops.pop().unwrap();
                let end = self.code.len();
                self.patch(position, Opcode::JumpFalse(end));

                for index in breaks {
                    self.patch(index, Opcode::Jump(end));
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                let length = self.loops.len();
                if length > 0 {
                    let index = self.code.len();
                    self.emit(Opcode::Jump(0), span);
                    self.loops[length - 1].1.push(index);
                }
            }
            AnalysisKind::Continue(_) => {
                if let Some(state) = self.loops.last() {
                    self.emit(Opcode::Jump(state.0), span);
                }
            }
            AnalysisKind::Binding(binding) => {
                if let Some(value) = binding.value {
                    if let AnalysisKind::Usage(target) = binding.target.kind {
                        self.walk(*value);
                        let address = self.memory;
                        self.memory += 1;
                        self.bindings.insert(target.to_string(), address);
                        self.emit(Opcode::Store(address), span);
                    }
                }
            }
            AnalysisKind::Usage(identifier) => {
                let target = identifier.to_string();
                if let Some(address) = self.bindings.get(&target) {
                    self.emit(Opcode::Load(*address), span);
                }
            }
            AnalysisKind::Assign(identifier, value) => {
                self.walk(*value);
                let target = identifier.to_string();
                if let Some(address) = self.bindings.get(&target) {
                    self.emit(Opcode::Store(*address), span);
                }
            }
            AnalysisKind::Function(function) => {
                let bypass = self.code.len();
                self.emit(Opcode::Jump(0), span);

                let address = self.code.len();
                self.functions.insert(function.target.to_string(), address);

                if let Some(body) = function.body {
                    self.walk(*body);
                }

                self.emit(Opcode::Return, span);

                let end = self.code.len();
                self.patch(bypass, Opcode::Jump(end));
            }
            AnalysisKind::Return(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                self.emit(Opcode::Return, span);
            }
            _ => {}
        }
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

        let mut translator = Translator::new();

        for &key in &sources {
            if let Some(analyses) = session.records.get(&key).unwrap().analyses.clone() {
                for analysis in analyses {
                    translator.walk(analysis);
                }
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
                        ("string_pointer", Signature::PtrPtr),
                        ("integer_pointer", Signature::PtrInt64),
                        ("pointer_integer", Signature::Int64Ptr),
                        ("integer_uint64", Signature::Uint64Int64),
                        ("integer_uint8", Signature::Uint8Int64),
                        ("uint8_character", Signature::Int32Uint8),
                        ("character_uint8", Signature::Uint8Int32),
                        ("character_integer", Signature::Int64Int32),
                        ("print_integer", Signature::VoidInt64),
                        ("print_float", Signature::VoidFloat64),
                        ("print_boolean", Signature::VoidInt32),
                        ("print_string", Signature::VoidPtr),
                        ("print_character", Signature::VoidUint8),
                        ("print_newline", Signature::VoidVoid),
                        ("print_hexadecimal", Signature::VoidInt64),
                        ("print_pointer", Signature::VoidPtr),
                        ("allocate_memory", Signature::PtrUint64),
                        ("free_memory", Signature::VoidPtr),
                        ("reallocate_memory", Signature::PtrPtrUint64),
                        ("memory_map", Signature::PtrPtrUint64Int64Int64Int64Int64),
                        ("memory_unmap", Signature::Int64PtrUint64),
                        ("file_write", Signature::Int64Int64PtrUint64),
                        ("file_read", Signature::Uint64Int64PtrUint64),
                        ("file_open", Signature::Int64PtrInt64Int64),
                        ("file_close", Signature::Int64Int64),
                        ("file_unlink", Signature::Int64Ptr),
                        ("file_seek", Signature::Int64Int64Int64Int64),
                        ("process_exit", Signature::VoidInt64),
                        ("string_length", Signature::Uint64Ptr),
                        ("character_at", Signature::Uint8PtrUint64),
                        ("is_whitespace", Signature::BoolUint8),
                        ("is_digit", Signature::BoolUint8),
                        ("string_substring", Signature::PtrPtrUint64Uint64),
                        ("parse_float", Signature::Float64Ptr),
                        ("get_input", Signature::PtrPtr),
                        ("vector_create", Signature::PtrVoid),
                        ("vector_count", Signature::Uint64Ptr),
                        ("vector_push", Signature::BoolPtrPtr),
                        ("vector_set", Signature::BoolPtrUint64Ptr),
                        ("vector_get", Signature::PtrPtrUint64),
                        ("vector_delete", Signature::BoolPtrUint64),
                        ("vector_free", Signature::VoidPtr),
                    ];

                    for (name, signature) in mappings {
                        if let Some(pointer) = instance.symbol(name) {
                            dynamic.push(NativeFunction::Dynamic(Dynamic {
                                pointer,
                                signature,
                            }));

                            translator.native(name, dynamic.len());
                        }
                    }

                    std::mem::forget(instance);
                } else {
                    panic!("failed to load compiled dynamic library: {}", library.to_string());
                }
            } else {
                panic!("clang failed to compile dynamic library.");
            }
        }

        let mut machine = Machine::new(translator.code, 1024, dynamic);

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
