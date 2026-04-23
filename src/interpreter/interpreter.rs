use {
    crate::{
        analyzer::Analysis,
        data::{
            memory::{Arc, RefCell},
            CString, Identity, Scale, Str,
        },
        internal::hash::Map,
        interpreter::{error::ErrorKind, InterpretError},
        reporter::Error,
        resolver::Type,
        tracker::Span,
    },
};

pub type Native<'a> = fn(&[Value], Span) -> Result<Value, InterpretError<'a>>;
pub type Address = usize;
pub type Index = usize;
pub type Tag = usize;

thread_local! {
    static FOREIGN_TEXT: RefCell<Vec<CString>> = RefCell::new(Vec::new());
}

#[derive(Clone)]
pub enum Foreign<'a> {
    Rust(Native<'a>),
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
    Structure(Identity, Vec<Value>),
    Variant(Tag, Box<Value>),
    Pointer(Address),
    Empty,
}

#[derive(Clone, Debug)]
pub enum Opcode {
    Mark,
    Restore(Scale),
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
    Jump(Address),
    JumpTrue(Address),
    JumpFalse(Address),
    Load(Address),
    Store(Address),
    StoreField(Address, Index),
    Call(Address),
    CallForeign(Index, Scale),
    Return,
    Halt,
    MakeSequence(Scale),
    MakeStructure(Identity, Scale),
    ExtractField(Index),
    Index,
    Trap(ErrorKind),
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub opcode: Opcode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Slot<'a> {
    pub address: Address,
    pub typing: Type<'a>,
}

#[derive(Clone, Debug)]
pub enum Call {
    Foreign(Index),
    Local(Option<Address>),
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub pointer: Address,
    pub start: Address,
    pub stack: Address,
    pub locals: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct CompilationUnit<'a> {
    pub stem: Str<'a>,
    pub analyses: Vec<Analysis<'a>>,
}

pub struct Interpreter<'a> {
    pub stack: Vec<Value>,
    pub marks: Vec<Address>,
    pub frames: Vec<Frame>,
    pub memory: Vec<Value>,
    pub code: Vec<Instruction>,
    pub foreign: Vec<Foreign<'a>>,
    pub slots: Map<Str<'a>, Slot<'a>>,
    pub calls: Map<Identity, Vec<(Type<'a>, Call)>>,
    pub values: Map<Str<'a>, Value>,
    pub function_frames: Map<Address, (Address, Scale, Scale)>,
    pub modules: Map<Str<'a>, ()>,
    pub units: Vec<CompilationUnit<'a>>,
    pub current_module: Str<'a>,
    pub pending: Vec<(Address, Identity, Type<'a>)>,
    pub loops: Vec<(Address, Vec<Address>)>,
    pub memory_top: Address,
    pub pointer: Address,
    pub running: bool,
}

impl<'a> Interpreter<'a> {
    pub fn new(capacity: Scale) -> Self {
        Self {
            stack: Vec::new(),
            marks: Vec::new(),
            frames: Vec::new(),
            memory: vec![Value::Empty; capacity],
            code: Vec::new(),
            foreign: Vec::new(),
            slots: Map::new(),
            calls: Map::new(),
            values: Map::new(),
            function_frames: Map::new(),
            modules: Map::new(),
            units: Vec::new(),
            current_module: Str::default(),
            pending: Vec::new(),
            loops: Vec::new(),
            memory_top: 0,
            pointer: 0,
            running: false,
        }
    }

    fn error(&self, kind: ErrorKind, span: Span) -> InterpretError<'a> {
        Error::new(kind, span)
    }

    fn current(&self) -> Span {
        self.code[self.pointer.saturating_sub(1)].span
    }

    pub fn slot(&self, name: &Str<'a>) -> Option<&Slot<'a>> {
        self.slots.get(name)
    }

    pub fn bind_slot(&mut self, name: Str<'a>, slot: Slot<'a>) {
        self.slots.insert(name, slot);
    }

    pub fn bind_value(&mut self, name: Str<'a>, value: Value) {
        self.values.insert(name, value);
    }

    pub fn has_module(&self, name: &Str<'a>) -> bool {
        self.modules.contains_key(name)
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.marks.clear();
        self.frames.clear();
        self.memory.fill(Value::Empty);
        self.code.clear();
        self.slots.clear();
        self.values.clear();
        self.function_frames.clear();
        self.modules.clear();
        self.units.clear();
        self.pending.clear();
        self.loops.clear();
        self.memory_top = 0;
        self.pointer = 0;
        self.running = false;
        self.calls.retain(|_, items| {
            items.retain(|(_, call)| matches!(call, Call::Foreign(_)));
            !items.is_empty()
        });
    }

    pub fn register_call(&mut self, identity: Identity, typing: Type<'a>, call: Call) {
        self.calls.entry(identity).or_default().push((typing, call));
    }

    pub fn set_call(&mut self, identity: Identity, typing: &Type<'a>, address: Address) {
        if let Some(items) = self.calls.get_mut(&identity) {
            for (item, call) in items {
                if item == typing {
                    *call = Call::Local(Some(address));
                    return;
                }
            }
        }
    }

    pub fn routine(&self, identity: Identity, typing: &Type<'a>) -> Option<Call> {
        let items = self.calls.get(&identity)?;

        items
            .iter()
            .find(|(item, _)| item == typing)
            .or_else(|| (items.len() == 1).then_some(&items[0]))
            .map(|(_, call)| call.clone())
    }

    pub fn run(&mut self) -> Result<(), InterpretError<'a>> {
        if self.frames.is_empty() {
            self.frames.push(Frame {
                pointer: self.code.len(),
                start: 0,
                stack: 0,
                locals: Vec::new(),
            });
        }

        self.running = true;
        while self.running && self.pointer < self.code.len() {
            self.step()?;
        }
        Ok(())
    }

    pub fn begin(&mut self, start: Address) {
        self.pointer = start;
        self.stack.clear();
        self.marks.clear();
        self.frames.clear();
    }

    fn step(&mut self) -> Result<(), InterpretError<'a>> {
        let instruction = self.code[self.pointer].clone();
        self.pointer += 1;

        match instruction.opcode {
            Opcode::Mark => self.marks.push(self.stack.len()),
            Opcode::Restore(count) => self.restore(count),
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
            Opcode::StoreField(address, field) => self.store_field(address, field)?,
            Opcode::Call(target) => self.call(target)?,
            Opcode::CallForeign(target, count) => self.call_foreign(target, count)?,
            Opcode::Return => self.finish()?,
            Opcode::Halt => self.running = false,
            Opcode::MakeSequence(size) => self.make_sequence(size)?,
            Opcode::MakeStructure(identity, size) => self.make_structure(identity, size)?,
            Opcode::ExtractField(field) => self.extract_field(field)?,
            Opcode::Index => self.index()?,
            Opcode::Trap(kind) => return Err(self.error(kind, instruction.span)),
        }

        Ok(())
    }

    fn restore(&mut self, count: Scale) {
        let Some(mark) = self.marks.pop() else {
            return;
        };

        let available = self.stack.len().saturating_sub(mark);
        let keep = count.min(available);

        if keep == 0 {
            self.stack.truncate(mark);
            return;
        }

        let start = self.stack.len() - keep;
        let values: Vec<_> = self.stack.drain(start..).collect();
        self.stack.truncate(mark);
        self.stack.extend(values);
    }

    fn pop(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        Ok(())
    }

    fn add(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left + right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
            (Value::Float(left), Value::Integer(right)) => Value::Float(left + right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 + right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn subtract(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left - right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
            (Value::Float(left), Value::Integer(right)) => Value::Float(left - right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 - right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn multiply(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left * right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
            (Value::Float(left), Value::Integer(right)) => Value::Float(left * right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 * right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn divide(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                if right == 0 {
                    return Err(self.error(ErrorKind::DivisionByZero, span));
                }
                Value::Integer(left / right)
            }
            (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
            (Value::Float(left), Value::Integer(right)) => Value::Float(left / right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 / right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn modulus(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                if right == 0 {
                    return Err(self.error(ErrorKind::DivisionByZero, span));
                }
                Value::Integer(left % right)
            }
            (Value::Float(left), Value::Float(right)) => Value::Float(left % right),
            (Value::Float(left), Value::Integer(right)) => Value::Float(left % right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 % right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn negate(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Integer(value) => Value::Integer(-value),
            Value::Float(value) => Value::Float(-value),
            _ => return Err(self.error(ErrorKind::InvalidUnary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn equal(&mut self) -> Result<(), InterpretError<'a>> {
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

    fn not_equal(&mut self) -> Result<(), InterpretError<'a>> {
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

    fn less(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left < right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left < right),
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left < right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) < right),
            _ => return Err(self.error(ErrorKind::InvalidCompare, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn greater(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left > right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left > right),
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left > right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) > right),
            _ => return Err(self.error(ErrorKind::InvalidCompare, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn less_equal(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left <= right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left <= right),
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left <= right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) <= right),
            _ => return Err(self.error(ErrorKind::InvalidCompare, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn greater_equal(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left >= right),
            (Value::Float(left), Value::Float(right)) => Value::Boolean(left >= right),
            (Value::Float(left), Value::Integer(right)) => Value::Boolean(left >= right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Boolean((left as f64) >= right),
            _ => return Err(self.error(ErrorKind::InvalidCompare, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logic_and(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left && right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logic_or(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left || right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logic_not(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Boolean(value) => Value::Boolean(!value),
            _ => return Err(self.error(ErrorKind::InvalidUnary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logic_xor(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(left ^ right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bit_and(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left & right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bit_or(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left | right),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bit_not(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Integer(value) => Value::Integer(!value),
            _ => return Err(self.error(ErrorKind::InvalidBinary, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bit_xor(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left ^ right),
            _ => return Err(self.error(ErrorKind::InvalidCondition, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn shift_left(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left << right),
            _ => return Err(self.error(ErrorKind::InvalidCondition, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn shift_right(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left >> right),
            _ => return Err(self.error(ErrorKind::InvalidStore, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn jump(&mut self, target: Address) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if target > self.code.len() {
            return Err(self.error(ErrorKind::OutOfBounds, span));
        }
        self.pointer = target;
        Ok(())
    }

    fn jump_true(&mut self, target: Address) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let condition = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match condition {
            Value::Boolean(true) => self.jump(target)?,
            Value::Boolean(false) => {}
            _ => return Err(self.error(ErrorKind::InvalidCondition, span)),
        }

        Ok(())
    }

    fn jump_false(&mut self, target: Address) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let condition = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match condition {
            Value::Boolean(false) => self.jump(target)?,
            Value::Boolean(true) => {}
            _ => return Err(self.error(ErrorKind::InvalidCondition, span)),
        }

        Ok(())
    }

    fn load(&mut self, address: Address) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.memory[address].clone();
        self.stack.push(value);
        Ok(())
    }

    fn store(&mut self, address: Address) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.memory[address] = value.clone();
        self.stack.push(value);
        Ok(())
    }

    fn store_field(&mut self, address: Address, field: Index) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match &mut self.memory[address] {
            Value::Structure(_, fields) => {
                if field >= fields.len() {
                    return Err(self.error(ErrorKind::OutOfBounds, span));
                }
                fields[field] = value.clone();
            }
            Value::Sequence(items) => {
                if field >= items.len() {
                    return Err(self.error(ErrorKind::OutOfBounds, span));
                }
                items[field] = value.clone();
            }
            _ => return Err(self.error(ErrorKind::InvalidStore, span)),
        }

        self.stack.push(value);
        Ok(())
    }

    fn extract_field(&mut self, field: Index) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let target = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match target {
            Value::Structure(_, fields) => {
                let value = fields.get(field).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?.clone();
                self.stack.push(value);
            }
            Value::Sequence(items) => {
                let value = items.get(field).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?.clone();
                self.stack.push(value);
            }
            _ => return Err(self.error(ErrorKind::InvalidAccess, span)),
        }
        Ok(())
    }

    fn call_foreign(&mut self, target: Index, count: Scale) -> Result<(), InterpretError<'a>> {
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

        FOREIGN_TEXT.with(|strings| strings.borrow_mut().clear());

        let result = result?;

        self.stack.truncate(start);
        self.stack.push(result);

        Ok(())
    }

    fn call(&mut self, target: Address) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if target >= self.code.len() {
            return Err(self.error(ErrorKind::OutOfBounds, span));
        }
        let (start, size, count) = self.function_frames.get(&target).copied().unwrap_or((0, 0, 0));
        let end = start + size;
        let stack = self.stack.len().saturating_sub(count);

        if end > self.memory.len() {
            self.memory.resize(end, Value::Empty);
        }

        let locals = self.memory[start..end].to_vec();
        for slot in &mut self.memory[start..end] {
            *slot = Value::Empty;
        }

        self.frames.push(Frame {
            pointer: self.pointer,
            start,
            stack,
            locals,
        });
        self.pointer = target;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();

        if self.frames.len() == 1 {
            self.frames.pop();
            self.running = false;
            return Ok(());
        }

        let frame = self.frames.pop().ok_or_else(|| self.error(ErrorKind::InvalidFrame, span))?;
        let value = self.stack.pop().unwrap_or(Value::Empty);
        let end = frame.start + frame.locals.len();
        self.memory[frame.start..end].clone_from_slice(&frame.locals);
        self.stack.truncate(frame.stack);
        self.stack.push(value);
        self.pointer = frame.pointer;
        Ok(())
    }

    fn make_sequence(&mut self, size: Scale) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if self.stack.len() < size {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }
        let start = self.stack.len() - size;
        let sequence = self.stack.drain(start..).collect();
        self.stack.push(Value::Sequence(sequence));
        Ok(())
    }

    fn make_structure(&mut self, identity: Identity, size: Scale) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        if self.stack.len() < size {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }
        let start = self.stack.len() - size;
        let fields = self.stack.drain(start..).collect();
        self.stack.push(Value::Structure(identity, fields));
        Ok(())
    }

    fn index(&mut self) -> Result<(), InterpretError<'a>> {
        let span = self.current();
        let position = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let target = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match (target, position) {
            (Value::Sequence(sequence), Value::Integer(index)) => {
                let index = index as Index;
                if index >= sequence.len() {
                    return Err(self.error(ErrorKind::OutOfBounds, span));
                }
                self.stack.push(sequence[index].clone());
            }
            _ => return Err(self.error(ErrorKind::InvalidIndex, span)),
        }
        Ok(())
    }

    pub fn extract(&mut self) -> Option<Value> {
        self.stack.pop()
    }
}

impl<'a> Default for Interpreter<'a> {
    fn default() -> Self {
        Interpreter::new(1024)
    }
}
