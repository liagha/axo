#![allow(unused)]

use {
    crate::{
        analyzer::{Analysis},
        data::{memory::Arc, CString, Str},
        internal::{
            hash::Map,
        },
        interpreter::{
            InterpretError,
            error::ErrorKind
        },
        reporter::Error,
        tracker::Span,
    },
};

pub type Native<'error> = fn(&[Value], Span<'error>) -> Result<Value, InterpretError<'error>>;

#[cfg(unix)]
pub mod sys {
    pub use libc::{c_void, dlopen, dlsym, RTLD_LAZY};
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

thread_local! {
    static FFI_STRINGS: std::cell::RefCell<Vec<CString>> = std::cell::RefCell::new(Vec::new());
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
    LogicAnd,
    LogicOr,
    LogicNot,
    LogicXor,
    BitAnd,
    BitOr,
    BitNot,
    BitXor,
    ShiftLeft,
    ShiftRight,
    Jump(usize),
    JumpTrue(usize),
    JumpFalse(usize),
    Load(usize),
    Store(usize),
    StoreField(usize, usize),
    Call(usize),
    CallForeign(usize, usize),
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

#[derive(Clone, Debug)]
pub enum Entity {
    Foreign(usize),
    Function(Option<usize>),
    Structure(usize, Vec<String>),
    Union(usize, Vec<String>),
    Module,
}

pub struct Machine<'error> {
    pub stack: Vec<Value>,
    pub frames: Vec<usize>,
    pub memory: Vec<Value>,
    pub code: Vec<Instruction<'error>>,
    pub foreign: Vec<Foreign<'error>>,
    pub bindings: Map<String, usize>,
    pub entities: Map<String, Entity>,
    pub modules: Map<Str<'error>, Vec<Analysis<'error>>>,
    pub current_module: Str<'error>,
    pub calls: Vec<(usize, String, String)>,
    pub loops: Vec<(usize, Vec<usize>)>,
    pub memory_top: usize,
    pub pointer: usize,
    pub running: bool,
}

impl<'error> Machine<'error> {
    pub fn new(capacity: usize) -> Self {
        let mut machine = Self {
            stack: Vec::new(),
            frames: Vec::new(),
            memory: vec![Value::Empty; capacity],
            code: Vec::new(),
            foreign: Vec::new(),
            bindings: Map::new(),
            entities: Map::new(),
            modules: Map::new(),
            current_module: Str::default(),
            calls: Vec::new(),
            loops: Vec::new(),
            memory_top: 0,
            pointer: 0,
            running: false,
        };

        machine.native("print", 0);
        machine
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
            Opcode::LogicAnd => self.logic_and()?,
            Opcode::LogicOr => self.logic_or()?,
            Opcode::LogicNot => self.logic_not()?,
            Opcode::LogicXor => self.logic_xor()?,
            Opcode::BitAnd => self.bit_and()?,
            Opcode::BitOr => self.bit_or()?,
            Opcode::BitNot => self.bit_not()?,
            Opcode::BitXor => self.bit_xor()?,
            Opcode::ShiftLeft => self.shift_left()?,
            Opcode::ShiftRight => self.shift_right()?,
            Opcode::Jump(target) => self.jump(target)?,
            Opcode::JumpTrue(target) => self.jump_true(target)?,
            Opcode::JumpFalse(target) => self.jump_false(target)?,
            Opcode::Load(address) => self.load(address)?,
            Opcode::Store(address) => self.store(address)?,
            Opcode::StoreField(address, index) => self.store_field(address, index)?,
            Opcode::Call(target) => self.call(target)?,
            Opcode::CallForeign(target, count) => self.call_foreign(target, count)?,
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
            (Value::Float(left), Value::Integer(right)) => Value::Float(left + right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 + right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Float(left - right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 - right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Float(left * right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 * right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Float(left / right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 / right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Float(left % right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 % right),
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

        let is_equal = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left == right,
            (Value::Float(left), Value::Integer(right)) => left == (right as f64),
            (Value::Integer(left), Value::Float(right)) => (left as f64) == right,
            (left, right) => left == right,
        };

        self.stack.push(Value::Boolean(is_equal));
        Ok(())
    }

    fn not_equal(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let is_not_equal = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => left != right,
            (Value::Float(left), Value::Float(right)) => left != right,
            (Value::Float(left), Value::Integer(right)) => left != (right as f64),
            (Value::Integer(left), Value::Float(right)) => (left as f64) != right,
            (left, right) => left != right,
        };

        self.stack.push(Value::Boolean(is_not_equal));
        Ok(())
    }

    fn less(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left < right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left < right),
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left < right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) < right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left > right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) > right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left <= right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) <= right),
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
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left >= right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) >= right),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logic_and(&mut self) -> Result<(), InterpretError<'error>> {
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

    fn logic_or(&mut self) -> Result<(), InterpretError<'error>> {
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

    fn logic_not(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Boolean(value) => Value::Boolean(!value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logic_xor(&mut self) -> Result<(), InterpretError<'error>> {
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

    fn bit_and(&mut self) -> Result<(), InterpretError<'error>> {
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

    fn bit_or(&mut self) -> Result<(), InterpretError<'error>> {
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

    fn bit_not(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Integer(value) => Value::Integer(!value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bit_xor(&mut self) -> Result<(), InterpretError<'error>> {
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

    fn store_field(&mut self, address: usize, index: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        if let Value::Structure(fields) = &mut self.memory[address] {
            if index >= fields.len() {
                return Err(self.error(ErrorKind::OutOfBounds, span));
            }
            fields[index] = value;
        } else {
            return Err(self.error(ErrorKind::TypeMismatch, span));
        }
        Ok(())
    }

    fn call_foreign(&mut self, target: usize, count: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current();
        let routine = self.foreign.get(target).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?.clone();

        if self.stack.len() < count {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }

        let start = self.stack.len() - count;
        let inputs = &self.stack[start..];

        let result = match routine {
            Foreign::Rust(function) => function(inputs, span),
            Foreign::Dynamic(dynamic) => dynamic(inputs).map_err(|kind| self.error(kind, span)),
        };

        FFI_STRINGS.with(|strings| strings.borrow_mut().clear());

        let result = result?;

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
            Value::Variant(..) => {
                self.stack.push(Value::Empty);
            }
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
