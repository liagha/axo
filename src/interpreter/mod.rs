#![allow(unused)]

mod error;

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        interpreter::error::ErrorKind,
        internal::hash::Map,
        tracker::Span,
        reporter::Error,
    }
};

pub type InterpretError<'error> = Error<'error, ErrorKind>;

pub type Native<'error> = fn(&[Value], Span<'error>) -> Result<Value, InterpretError<'error>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    Text(String),
    Sequence(Vec<Value>),
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
    natives: Vec<Native<'error>>,
    pointer: usize,
    running: bool,
}

pub fn native_print<'error>(arguments: &[Value], _span: Span<'error>) -> Result<Value, InterpretError<'error>> {
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            print!(" ");
        }
        match argument {
            Value::Integer(value) => print!("{}", value),
            Value::Float(value) => print!("{}", value),
            Value::Boolean(value) => print!("{}", value),
            Value::Character(value) => print!("{}", value),
            Value::Text(value) => print!("{}", value),
            Value::Sequence(value) => print!("{:?}", value),
            Value::Empty => print!("empty"),
        }
    }

    println!();

    Ok(Value::Empty)
}

impl<'error> Machine<'error> {
    pub fn new(code: Vec<Instruction<'error>>, capacity: usize, natives: Vec<Native<'error>>) -> Self {
        let mut bundled_natives: Vec<Native<'error>> = vec![native_print];
        bundled_natives.extend(natives);

        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            memory: vec![Value::Empty; capacity],
            code,
            natives: bundled_natives,
            pointer: 0,
            running: false,
        }
    }

    fn error(&self, kind: ErrorKind, span: Span<'error>) -> InterpretError<'error> {
        Error::new(kind, span)
    }

    fn current_span(&self) -> Span<'error> {
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.stack.push(Value::Boolean(left == right));
        Ok(())
    }

    fn not_equal(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        let right = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        let left = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.stack.push(Value::Boolean(left != right));
        Ok(())
    }

    fn less(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Boolean(value) => Value::Boolean(!value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_xor(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        let result = match value {
            Value::Integer(value) => Value::Integer(!value),
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_xor(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
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
        let span = self.current_span();
        if target >= self.code.len() {
            return Err(self.error(ErrorKind::OutOfBounds, span));
        }
        self.pointer = target;
        Ok(())
    }

    fn jump_true(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        let condition = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match condition {
            Value::Boolean(true) => self.jump(target)?,
            Value::Boolean(false) => {}
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }

        Ok(())
    }

    fn jump_false(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        let condition = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;

        match condition {
            Value::Boolean(false) => self.jump(target)?,
            Value::Boolean(true) => {}
            _ => return Err(self.error(ErrorKind::TypeMismatch, span)),
        }

        Ok(())
    }

    fn load(&mut self, address: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.memory[address].clone();
        self.stack.push(value);
        Ok(())
    }

    fn store(&mut self, address: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        if address >= self.memory.len() {
            return Err(self.error(ErrorKind::MemoryAccessViolation, span));
        }
        let value = self.stack.pop().ok_or_else(|| self.error(ErrorKind::StackUnderflow, span))?;
        self.memory[address] = value;
        Ok(())
    }

    fn native_call(&mut self, target: usize, count: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        let function = self.natives.get(target).ok_or_else(|| self.error(ErrorKind::OutOfBounds, span))?;

        if self.stack.len() < count {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }

        let start = self.stack.len() - count;
        let arguments = &self.stack[start..];

        let result = function(arguments, span)?;

        self.stack.truncate(start);
        self.stack.push(result);

        Ok(())
    }

    fn call(&mut self, target: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        if target >= self.code.len() {
            return Err(self.error(ErrorKind::OutOfBounds, span));
        }
        self.frames.push(self.pointer);
        self.pointer = target;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        self.pointer = self.frames.pop().ok_or_else(|| self.error(ErrorKind::InvalidFrame, span))?;
        Ok(())
    }

    fn make_sequence(&mut self, size: usize) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
        if self.stack.len() < size {
            return Err(self.error(ErrorKind::StackUnderflow, span));
        }
        let start = self.stack.len() - size;
        let sequence = self.stack.drain(start..).collect();
        self.stack.push(Value::Sequence(sequence));
        Ok(())
    }

    fn index(&mut self) -> Result<(), InterpretError<'error>> {
        let span = self.current_span();
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

    pub fn define_native(&mut self, identifier: &str, index: usize) {
        self.natives.insert(identifier.to_string(), index);
    }

    fn emit(&mut self, opcode: Opcode, span: Span<'error>) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch_jump(&mut self, position: usize, opcode: Opcode) {
        self.code[position].opcode = opcode;
    }

    pub fn compile(mut self, nodes: Vec<Analysis<'error>>) -> Vec<Instruction<'error>> {
        for node in nodes {
            self.walk(node);
        }

        if let Some(last_span) = self.code.last().map(|inst| inst.span.clone()) {
            self.emit(Opcode::Halt, last_span);
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
                let patch = self.code.len();
                self.emit(Opcode::JumpFalse(0), span);
                self.walk(*truthy);

                if let Some(alternative) = falsy {
                    let bypass = self.code.len();
                    self.emit(Opcode::Jump(0), span);
                    self.patch_jump(patch, Opcode::JumpFalse(self.code.len()));
                    self.walk(*alternative);
                    self.patch_jump(bypass, Opcode::Jump(self.code.len()));
                } else {
                    self.patch_jump(patch, Opcode::JumpFalse(self.code.len()));
                }
            }
            AnalysisKind::While(condition, body) => {
                let start = self.code.len();
                self.walk(*condition);
                let patch = self.code.len();
                self.emit(Opcode::JumpFalse(0), span);

                self.loops.push((start, Vec::new()));
                self.walk(*body);
                self.emit(Opcode::Jump(start), span);

                let (_, breaks) = self.loops.pop().unwrap();
                let end = self.code.len();
                self.patch_jump(patch, Opcode::JumpFalse(end));

                for index in breaks {
                    self.patch_jump(index, Opcode::Jump(end));
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                let len = self.loops.len();
                if len > 0 {
                    let index = self.code.len();
                    self.emit(Opcode::Jump(0), span);
                    self.loops[len - 1].1.push(index);
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
                self.patch_jump(bypass, Opcode::Jump(end));
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