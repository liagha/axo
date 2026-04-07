mod error;
mod translator;

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        combinator::{Action, Operation, Operator},
        data::{memory::Arc, CString, Interface, Str},
        internal::{
            hash::Map,
            platform::Lock,
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
    stack: Vec<Value>,
    frames: Vec<usize>,
    memory: Vec<Value>,
    code: Vec<Instruction<'error>>,
    foreign: Vec<Foreign<'error>>,
    bindings: Map<String, usize>,
    entities: Map<String, Entity>,
    pub modules: Map<Str<'error>, Vec<Analysis<'error>>>,
    pub current_module: Str<'error>,
    calls: Vec<(usize, String, String)>,
    loops: Vec<(usize, Vec<usize>)>,
    memory_top: usize,
    pointer: usize,
    running: bool,
}

impl<'error> Machine<'error> {
    pub fn new(capacity: usize, foreign: Vec<Foreign<'error>>) -> Self {
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

        let mut vm = Machine::new(1024, Vec::new());

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
                                        (ffi_clone.prep_cif_var.unwrap())(
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