#![allow(unused)]

mod error;
mod translator;

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        combinator::{Action, Operation, Operator},
        data::{memory::Arc, Str, CString, Interface},
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
        let string = CString::new(path).ok()?;
        let handle = unsafe { sys::dlopen(string.as_ptr(), sys::RTLD_LAZY) };
        (!handle.is_null()).then_some(Self { handle })
    }

    pub fn symbol(&self, name: &str) -> Option<*mut sys::c_void> {
        let string = CString::new(name).ok()?;
        let pointer = unsafe { sys::dlsym(self.handle, string.as_ptr()) };
        (!pointer.is_null()).then_some(pointer)
    }
}

pub trait Cast: Sized {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind>;
}

pub trait Wrap {
    fn wrap(self) -> Value;
}

impl Cast for i64 {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Integer(v)) => Ok(*v),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Cast for i32 {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Integer(v)) => Ok(*v as i32),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Cast for u64 {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Integer(v)) => Ok(*v as u64),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Cast for u8 {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Integer(v)) => Ok(*v as u8),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Cast for f64 {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Float(v)) => Ok(*v),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Cast for bool {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Boolean(v)) => Ok(*v),
            Some(Value::Integer(v)) => Ok(*v != 0),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Cast for *mut u8 {
    fn cast(value: Option<&Value>) -> Result<Self, ErrorKind> {
        match value {
            Some(Value::Pointer(v)) => Ok(*v as *mut u8),
            _ => Err(ErrorKind::TypeMismatch),
        }
    }
}

impl Wrap for () {
    fn wrap(self) -> Value { Value::Empty }
}

impl Wrap for i64 {
    fn wrap(self) -> Value { Value::Integer(self) }
}

impl Wrap for i32 {
    fn wrap(self) -> Value { Value::Integer(self as i64) }
}

impl Wrap for u64 {
    fn wrap(self) -> Value { Value::Integer(self as i64) }
}

impl Wrap for u8 {
    fn wrap(self) -> Value { Value::Integer(self as i64) }
}

impl Wrap for f64 {
    fn wrap(self) -> Value { Value::Float(self) }
}

impl Wrap for bool {
    fn wrap(self) -> Value { Value::Boolean(self) }
}

impl Wrap for *mut u8 {
    fn wrap(self) -> Value { Value::Pointer(self as usize) }
}

macro_rules! bind {
    ($instance:expr, $store:expr, $translator:expr, $name:expr, $ret:ty $(, $arg:ty)*) => {
        if let Some(pointer) = $instance.symbol($name) {
            struct UnsafeWrapper<T>(T);
            unsafe impl<T> Send for UnsafeWrapper<T> {}
            unsafe impl<T> Sync for UnsafeWrapper<T> {}
            let safe_instance = UnsafeWrapper(instance);

            let execute = Arc::new(move |args: &[Value]| -> Result<Value, ErrorKind> {
                let instance = &safe_instance.0;
                let mut _index = 0;
                let function: extern "C" fn($($arg),*) -> $ret = unsafe { std::mem::transmute(pointer) };
                let result = function($(
                    {
                        let val = <$arg as Cast>::cast(args.get(_index))?;
                        _index += 1;
                        val
                    }
                ),*);
                Ok(result.wrap())
            });
            $store.push(Foreign::Dynamic(execute));
            $translator.native($name, $store.len() - 1);
        }
    };
}

#[derive(Clone)]
pub enum Foreign<'error> {
    Rust(Native<'error>),
    Dynamic(Arc<dyn Fn(&[Value]) -> Result<Value, ErrorKind> + Send + Sync>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    Text(String),
    Sequence(Vec<Value>),
    Structure(Vec<Value>),
    Variant(usize, Box<Value>),
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
    ForeignCall(usize, usize),
    Return,
    Halt,
    MakeSequence(usize),
    MakeStructure(usize),
    MakeVariant(usize),
    ExtractField(usize),
    ExtractVariant(usize),
    Index,
    Trap,
    CastInteger,
    CastFloat,
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
    foreign: Vec<Foreign<'error>>,
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
            Value::Structure(value) => std::print!("{:?}", value),
            Value::Variant(tag, value) => std::print!("{}: {:?}", tag, value),
            Value::Pointer(value) => std::print!("{:#x}", value),
            Value::Empty => std::print!("empty"),
        }
    }
    println!();
    Ok(Value::Empty)
}

impl<'error> Machine<'error> {
    pub fn new(code: Vec<Instruction<'error>>, capacity: usize, foreign: Vec<Foreign<'error>>) -> Self {
        let mut base = vec![Foreign::Rust(print)];
        base.extend(foreign);

        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            memory: vec![Value::Empty; capacity],
            code,
            foreign: base,
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
            Opcode::Pop => self.pop()?,
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
            Opcode::ForeignCall(target, count) => self.foreign_call(target, count)?,
            Opcode::Return => self.finish()?,
            Opcode::Halt => self.running = false,
            Opcode::MakeSequence(size) => self.make_sequence(size)?,
            Opcode::MakeStructure(size) => self.make_structure(size)?,
            Opcode::MakeVariant(tag) => self.make_variant(tag)?,
            Opcode::ExtractField(index) => self.extract_field(index)?,
            Opcode::ExtractVariant(tag) => self.extract_variant(tag)?,
            Opcode::Index => self.index()?,
            Opcode::Trap => return Err(self.error(ErrorKind::OutOfBounds, instruction.span)),
            Opcode::CastInteger => self.cast_integer()?,
            Opcode::CastFloat => self.cast_float()?,
        }

        Ok(())
    }

    fn pop(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
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

    fn foreign_call(&mut self, target: usize, count: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let routine = self.foreign.get(target).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?.clone();

        if self.stack.len() < count {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }

        let start = self.stack.len() - count;
        let arguments = &self.stack[start..];

        let result = match routine {
            Foreign::Rust(function) => function(arguments, span)?,
            Foreign::Dynamic(dynamic) => dynamic(arguments).map_err(|kind| self.error(kind, span))?,
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

    fn make_structure(&mut self, size: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if self.stack.len() < size {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }
        let start = self.stack.len() - size;
        let fields = self.stack.drain(start..).collect();
        self.stack.push(Value::Structure(fields));
        Ok(())
    }

    fn make_variant(&mut self, tag: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.stack.push(Value::Variant(tag, Box::new(value)));
        Ok(())
    }

    fn extract_field(&mut self, index: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let target = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match target {
            Value::Structure(fields) => {
                let value = fields.get(index).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?.clone();
                self.stack.push(value);
            }
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }
        Ok(())
    }

    fn extract_variant(&mut self, tag: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let target = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match target {
            Value::Variant(active, value) if active == tag => {
                self.stack.push(*value);
            }
            Value::Variant(..) => return Err(self.error(ErrorKind::TypeMismatch, span)),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }
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

    fn cast_integer(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let result = match value {
            Value::Float(v) => Value::Integer(v as i64),
            Value::Boolean(v) => Value::Integer(v as i64),
            Value::Character(v) => Value::Integer(v as i64),
            v @ Value::Integer(_) => v,
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };
        self.stack.push(result);
        Ok(())
    }

    fn cast_float(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let result = match value {
            Value::Integer(v) => Value::Float(v as f64),
            v @ Value::Float(_) => v,
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };
        self.stack.push(result);
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

        for (&key, record) in session.records.iter() {
            if record.kind == InputKind::Source && record.module.is_some() {
                sources.push(key);
            }
        }
        sources.sort();

        let mut translator = translator::Translator::new();
        let mut all = Vec::new();

        for &key in &sources {
            let record = session.records.get(&key).unwrap();
            let location = record.location;
            let stem = Str::from(location.stem().unwrap().to_string());

            translator.current_module = stem;

            if let Some(analyses) = record.analyses.clone() {
                all.extend(analyses);
            }
        }

        let mut dynamic = Vec::new();

        for analysis in &all {
            if let AnalysisKind::Function(function) = &analysis.kind {
                if matches!(function.interface, Interface::C) {
                    let name = function.target.as_str().unwrap_or_default();

                    let c_name = CString::new(name).unwrap();
                    let pointer = unsafe {
                        libc::dlsym(libc::RTLD_DEFAULT, c_name.as_ptr())
                    };

                    if !pointer.is_null() {
                        let arity = function.members.len();
                        let ptr_addr = pointer as usize;

                        let execute = Arc::new(move |args: &[Value]| -> Result<Value, ErrorKind> {
                            unsafe {
                                match arity {
                                    0 => {
                                        let func: extern "C" fn() -> i64 = std::mem::transmute(ptr_addr);
                                        Ok(Value::Integer(func()))
                                    }
                                    1 => {
                                        let func: extern "C" fn(i64) -> i64 = std::mem::transmute(ptr_addr);
                                        let arg0 = i64::cast(args.get(0))?;
                                        Ok(Value::Integer(func(arg0)))
                                    }
                                    2 => {
                                        let func: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(ptr_addr);
                                        let arg0 = i64::cast(args.get(0))?;
                                        let arg1 = i64::cast(args.get(1))?;
                                        Ok(Value::Integer(func(arg0, arg1)))
                                    }
                                    3 => {
                                        let func: extern "C" fn(i64, i64, i64) -> i64 = std::mem::transmute(ptr_addr);
                                        let arg0 = i64::cast(args.get(0))?;
                                        let arg1 = i64::cast(args.get(1))?;
                                        let arg2 = i64::cast(args.get(2))?;
                                        Ok(Value::Integer(func(arg0, arg1, arg2)))
                                    }
                                    4 => {
                                        let func: extern "C" fn(i64, i64, i64, i64) -> i64 = std::mem::transmute(ptr_addr);
                                        let arg0 = i64::cast(args.get(0))?;
                                        let arg1 = i64::cast(args.get(1))?;
                                        let arg2 = i64::cast(args.get(2))?;
                                        let arg3 = i64::cast(args.get(3))?;
                                        Ok(Value::Integer(func(arg0, arg1, arg2, arg3)))
                                    }
                                    5 => {
                                        let func: extern "C" fn(i64, i64, i64, i64, i64) -> i64 = std::mem::transmute(ptr_addr);
                                        let arg0 = i64::cast(args.get(0))?;
                                        let arg1 = i64::cast(args.get(1))?;
                                        let arg2 = i64::cast(args.get(2))?;
                                        let arg3 = i64::cast(args.get(3))?;
                                        let arg4 = i64::cast(args.get(4))?;
                                        Ok(Value::Integer(func(arg0, arg1, arg2, arg3, arg4)))
                                    }
                                    6 => {
                                        let func: extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64 = std::mem::transmute(ptr_addr);
                                        let arg0 = i64::cast(args.get(0))?;
                                        let arg1 = i64::cast(args.get(1))?;
                                        let arg2 = i64::cast(args.get(2))?;
                                        let arg3 = i64::cast(args.get(3))?;
                                        let arg4 = i64::cast(args.get(4))?;
                                        let arg5 = i64::cast(args.get(5))?;
                                        Ok(Value::Integer(func(arg0, arg1, arg2, arg3, arg4, arg5)))
                                    }
                                    _ => Err(ErrorKind::TypeMismatch),
                                }
                            }
                        });

                        dynamic.push(Foreign::Dynamic(execute));
                        translator.native(name, dynamic.len());
                    } else {
                        panic!("Could not resolve C function symbol: {}. Please ensure C libraries are dynamically loaded/linked into the interpreter executable.", name);
                    }
                }
            }
        }

        let code = translator.compile(all);
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
    }
}