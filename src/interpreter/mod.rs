use std::collections::HashMap;

use crate::{
    analyzer::{Analysis, AnalysisKind},
};

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
    Return,
    Halt,
}

#[derive(Debug)]
pub enum Error {
    Stack,
    Memory,
    Type,
    Bounds,
    Frame,
}

pub struct Machine {
    stack: Vec<Value>,
    frames: Vec<usize>,
    memory: Vec<Value>,
    code: Vec<Opcode>,
    pointer: usize,
    running: bool,
}

impl Machine {
    pub fn new(code: Vec<Opcode>, capacity: usize) -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            memory: vec![Value::Empty; capacity],
            code,
            pointer: 0,
            running: false,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.running = true;
        while self.running && self.pointer < self.code.len() {
            self.step()?;
        }
        Ok(())
    }

    fn step(&mut self) -> Result<(), Error> {
        let opcode = self.code[self.pointer].clone();
        self.pointer += 1;

        match opcode {
            Opcode::Push(value) => self.stack.push(value),
            Opcode::Pop => {
                self.stack.pop().ok_or(Error::Stack)?;
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
            Opcode::Return => self.finish()?,
            Opcode::Halt => self.running = false,
        }

        Ok(())
    }

    fn add(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn subtract(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn multiply(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn divide(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(Error::Bounds);
                }
                Value::Integer(a / b)
            }
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn modulus(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(Error::Bounds);
                }
                Value::Integer(a % b)
            }
            (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn negate(&mut self) -> Result<(), Error> {
        let value = self.stack.pop().ok_or(Error::Stack)?;

        let result = match value {
            Value::Integer(a) => Value::Integer(-a),
            Value::Float(a) => Value::Float(-a),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn equal(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;
        self.stack.push(Value::Boolean(left == right));
        Ok(())
    }

    fn not_equal(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;
        self.stack.push(Value::Boolean(left != right));
        Ok(())
    }

    fn less(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a < b),
            (Value::Float(a), Value::Float(b)) => Value::Boolean(a < b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn greater(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a > b),
            (Value::Float(a), Value::Float(b)) => Value::Boolean(a > b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn less_equal(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a <= b),
            (Value::Float(a), Value::Float(b)) => Value::Boolean(a <= b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn greater_equal(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a >= b),
            (Value::Float(a), Value::Float(b)) => Value::Boolean(a >= b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_and(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a && b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_or(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a || b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_not(&mut self) -> Result<(), Error> {
        let value = self.stack.pop().ok_or(Error::Stack)?;

        let result = match value {
            Value::Boolean(a) => Value::Boolean(!a),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn logical_xor(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a ^ b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_and(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a & b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_or(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a | b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_not(&mut self) -> Result<(), Error> {
        let value = self.stack.pop().ok_or(Error::Stack)?;

        let result = match value {
            Value::Integer(a) => Value::Integer(!a),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn bitwise_xor(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a ^ b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn shift_left(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a << b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn shift_right(&mut self) -> Result<(), Error> {
        let right = self.stack.pop().ok_or(Error::Stack)?;
        let left = self.stack.pop().ok_or(Error::Stack)?;

        let result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a >> b),
            _ => return Err(Error::Type),
        };

        self.stack.push(result);
        Ok(())
    }

    fn jump(&mut self, target: usize) -> Result<(), Error> {
        if target >= self.code.len() {
            return Err(Error::Bounds);
        }
        self.pointer = target;
        Ok(())
    }

    fn jump_true(&mut self, target: usize) -> Result<(), Error> {
        let condition = self.stack.pop().ok_or(Error::Stack)?;

        match condition {
            Value::Boolean(true) => self.jump(target)?,
            Value::Boolean(false) => {}
            _ => return Err(Error::Type),
        }

        Ok(())
    }

    fn jump_false(&mut self, target: usize) -> Result<(), Error> {
        let condition = self.stack.pop().ok_or(Error::Stack)?;

        match condition {
            Value::Boolean(false) => self.jump(target)?,
            Value::Boolean(true) => {}
            _ => return Err(Error::Type),
        }

        Ok(())
    }

    fn load(&mut self, address: usize) -> Result<(), Error> {
        if address >= self.memory.len() {
            return Err(Error::Memory);
        }
        let value = self.memory[address].clone();
        self.stack.push(value);
        Ok(())
    }

    fn store(&mut self, address: usize) -> Result<(), Error> {
        if address >= self.memory.len() {
            return Err(Error::Memory);
        }
        let value = self.stack.pop().ok_or(Error::Stack)?;
        self.memory[address] = value;
        Ok(())
    }

    fn call(&mut self, target: usize) -> Result<(), Error> {
        if target >= self.code.len() {
            return Err(Error::Bounds);
        }
        self.frames.push(self.pointer);
        self.pointer = target;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.pointer = self.frames.pop().ok_or(Error::Frame)?;
        Ok(())
    }

    pub fn extract(mut self) -> Option<Value> {
        self.stack.pop()
    }
}

pub struct Translator {
    code: Vec<Opcode>,
    memory: usize,
    bindings: HashMap<String, usize>,
    loops: Vec<(usize, Vec<usize>)>,
}

impl Translator {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            memory: 0,
            bindings: HashMap::new(),
            loops: Vec::new(),
        }
    }

    pub fn compile(mut self, nodes: Vec<Analysis>) -> Vec<Opcode> {
        for node in nodes {
            self.walk(node);
        }
        self.code.push(Opcode::Halt);
        self.code
    }

    fn walk(&mut self, node: Analysis) {
        match node.kind {
            AnalysisKind::Integer { value, .. } => {
                self.code.push(Opcode::Push(Value::Integer(value as i64)));
            }
            AnalysisKind::Float { value, .. } => {
                self.code.push(Opcode::Push(Value::Float(f64::from(value))));
            }
            AnalysisKind::Boolean { value } => {
                self.code.push(Opcode::Push(Value::Boolean(value)));
            }
            AnalysisKind::Character { value } => {
                self.code.push(Opcode::Push(Value::Character(value as char)));
            }
            AnalysisKind::String { value } => {
                self.code.push(Opcode::Push(Value::Text(value.to_string())));
            }
            AnalysisKind::Negate(value) => {
                self.walk(*value);
                self.code.push(Opcode::Negate);
            }
            AnalysisKind::Add(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Add);
            }
            AnalysisKind::Subtract(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Subtract);
            }
            AnalysisKind::Multiply(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Multiply);
            }
            AnalysisKind::Divide(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Divide);
            }
            AnalysisKind::Modulus(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Modulus);
            }
            AnalysisKind::LogicalAnd(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::LogicalAnd);
            }
            AnalysisKind::LogicalOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::LogicalOr);
            }
            AnalysisKind::LogicalNot(operand) => {
                self.walk(*operand);
                self.code.push(Opcode::LogicalNot);
            }
            AnalysisKind::LogicalXOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::LogicalXor);
            }
            AnalysisKind::BitwiseAnd(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::BitwiseAnd);
            }
            AnalysisKind::BitwiseOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::BitwiseOr);
            }
            AnalysisKind::BitwiseNot(operand) => {
                self.walk(*operand);
                self.code.push(Opcode::BitwiseNot);
            }
            AnalysisKind::BitwiseXOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::BitwiseXor);
            }
            AnalysisKind::ShiftLeft(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::ShiftLeft);
            }
            AnalysisKind::ShiftRight(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::ShiftRight);
            }
            AnalysisKind::Equal(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Equal);
            }
            AnalysisKind::NotEqual(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::NotEqual);
            }
            AnalysisKind::Less(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Less);
            }
            AnalysisKind::LessOrEqual(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::LessEqual);
            }
            AnalysisKind::Greater(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::Greater);
            }
            AnalysisKind::GreaterOrEqual(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.code.push(Opcode::GreaterEqual);
            }
            AnalysisKind::Block(statements) => {
                for statement in statements {
                    self.walk(statement);
                }
            }
            AnalysisKind::Conditional(condition, truthy, falsy) => {
                self.walk(*condition);
                let patch = self.code.len();
                self.code.push(Opcode::JumpFalse(0));
                self.walk(*truthy);

                if let Some(alternative) = falsy {
                    let bypass = self.code.len();
                    self.code.push(Opcode::Jump(0));
                    self.code[patch] = Opcode::JumpFalse(self.code.len());
                    self.walk(*alternative);
                    self.code[bypass] = Opcode::Jump(self.code.len());
                } else {
                    self.code[patch] = Opcode::JumpFalse(self.code.len());
                }
            }
            AnalysisKind::While(condition, body) => {
                let start = self.code.len();
                self.walk(*condition);
                let patch = self.code.len();
                self.code.push(Opcode::JumpFalse(0));

                self.loops.push((start, Vec::new()));
                self.walk(*body);
                self.code.push(Opcode::Jump(start));

                let (_, breaks) = self.loops.pop().unwrap();
                let end = self.code.len();
                self.code[patch] = Opcode::JumpFalse(end);

                for index in breaks {
                    self.code[index] = Opcode::Jump(end);
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                if let Some(state) = self.loops.last_mut() {
                    let index = self.code.len();
                    self.code.push(Opcode::Jump(0));
                    state.1.push(index);
                }
            }
            AnalysisKind::Continue(_) => {
                if let Some(state) = self.loops.last() {
                    self.code.push(Opcode::Jump(state.0));
                }
            }
            AnalysisKind::Binding(binding) => {
                if let Some(value) = binding.value {
                    if let AnalysisKind::Usage(target) = binding.target.kind {
                        self.walk(*value);
                        let address = self.memory;
                        self.memory += 1;
                        self.bindings.insert(target.to_string(), address);
                        self.code.push(Opcode::Store(address));
                    }
                }
            }
            AnalysisKind::Usage(identifier) => {
                let target = identifier.to_string();
                if let Some(address) = self.bindings.get(&target) {
                    self.code.push(Opcode::Load(*address));
                }
            }
            AnalysisKind::Assign(identifier, value) => {
                self.walk(*value);
                let target = identifier.to_string();
                if let Some(address) = self.bindings.get(&target) {
                    self.code.push(Opcode::Store(*address));
                }
            }
            AnalysisKind::Return(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                self.code.push(Opcode::Return);
            }
            _ => {}
        }
    }
}
