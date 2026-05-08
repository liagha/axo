use crate::{
    analyzer::{Analysis, AnalysisKind},
    data::{Function, Str},
    emitter::{
        interpreter::{
            compiler::{Chunk, Compiler},
            error::InterpretError,
            instruction::Instruction,
            value::Value,
            Foreign,
        },
        BitwiseError, DataStructureError, ErrorKind, FunctionError, VariableError,
    },
    internal::hash::Map,
    resolver::Type,
    tracker::Span,
};

type AxoFn<'a> = Function<Str<'a>, Analysis<'a>, Option<Box<Analysis<'a>>>, Option<Type<'a>>>;

enum Signal<'a> {
    Return(Value<'a>),
    Break,
    Continue,
}

pub struct Machine<'a> {
    stack: Vec<Value<'a>>,
    globals: Map<Str<'a>, Value<'a>>,
    functions: Map<Str<'a>, AxoFn<'a>>,
    foreigns: Map<Str<'a>, Foreign<'a>>,
    frames: Vec<usize>,
    signal: Option<Signal<'a>>,
}

impl<'a> Machine<'a> {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: Map::default(),
            functions: Map::default(),
            foreigns: Map::default(),
            frames: Vec::new(),
            signal: None,
        }
    }

    pub fn register(&mut self, name: Str<'a>, foreign: Foreign<'a>) {
        self.foreigns.insert(name, foreign);
    }

    pub fn load(&mut self, analyses: &[Analysis<'a>]) -> Result<(), InterpretError<'a>> {
        for analysis in analyses {
            if let AnalysisKind::Function(f) = &analysis.kind {
                self.functions.insert(f.target, f.clone());
                self.globals.insert(f.target, Value::Function(f.target));
            }
        }
        Ok(())
    }

    pub fn run(&mut self, chunk: &Chunk<'a>) -> Result<Value<'a>, InterpretError<'a>> {
        let mut ip = 0;
        let frame_base = self.stack.len();
        self.frames.push(frame_base);

        let result = loop {
            if ip >= chunk.ops.len() {
                break self.stack.pop().unwrap_or(Value::Void);
            }

            let op = chunk.ops[ip].clone();
            ip += 1;

            match op {
                Instruction::Void => self.stack.push(Value::Void),
                Instruction::Integer(n) => self.stack.push(Value::Integer(n)),
                Instruction::Float(f) => self.stack.push(Value::Float(f)),
                Instruction::Boolean(b) => self.stack.push(Value::Boolean(b)),
                Instruction::Character(c) => self.stack.push(Value::Character(c)),
                Instruction::String(s) => self.stack.push(Value::String(s)),

                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Dup => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    self.stack.push(top);
                }

                Instruction::Load(slot) => {
                    let base = *self.frames.last().unwrap_or(&0);
                    let value = self.stack.get(base + slot).cloned().unwrap_or(Value::Void);
                    self.stack.push(value);
                }
                Instruction::Store(slot) => {
                    let base = *self.frames.last().unwrap_or(&0);
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    let target = base + slot;
                    while self.stack.len() <= target {
                        self.stack.push(Value::Void);
                    }
                    self.stack[target] = value;
                }
                Instruction::LoadGlobal(name) => {
                    let value = self.globals.get(&name).cloned().unwrap_or(Value::Void);
                    self.stack.push(value);
                }
                Instruction::StoreGlobal(name) => {
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    self.globals.insert(name, value);
                }
                Instruction::DefineGlobal(name) => {
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    self.globals.insert(name, value);
                }

                Instruction::EnterBlock => {}
                Instruction::LeaveBlock => {}

                Instruction::Negate => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let result = match v {
                        Value::Integer(n) => Value::Integer(n.wrapping_neg()),
                        Value::Float(f) => Value::Float(-f),
                        _ => return Err(self.err(ErrorKind::Negate, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Not => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let result = match v {
                        Value::Boolean(b) => Value::Boolean(!b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::BitwiseNot => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let result = match v {
                        Value::Integer(n) => Value::Integer(!n),
                        _ => {
                            return Err(self.err(
                                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                    instruction: String::from("not"),
                                }),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }
                Instruction::AddressOf => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.stack.push(Value::Pointer(Box::new(v)));
                }
                Instruction::Deref => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let inner = match v {
                        Value::Pointer(inner) => *inner,
                        _ => {
                            return Err(self.err(
                                ErrorKind::Variable(VariableError::DereferenceNonPointer),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(inner);
                }

                Instruction::Add => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_add(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Subtract => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_sub(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Multiply => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_mul(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Divide => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b == 0 {
                                panic!("division by zero");
                            }
                            if b == -1 && a == i64::MIN {
                                panic!("integer overflow");
                            }
                            Value::Integer(a / b)
                        }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Modulus => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b == 0 {
                                panic!("modulus by zero");
                            }
                            if b == -1 && a == i64::MIN {
                                panic!("integer overflow");
                            }
                            Value::Integer(a % b)
                        }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::And => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a && b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Or => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a || b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Xor => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a ^ b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::BitwiseAnd => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a & b),
                        _ => {
                            return Err(self.err(
                                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                    instruction: String::from("and"),
                                }),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }
                Instruction::BitwiseOr => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a | b),
                        _ => {
                            return Err(self.err(
                                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                    instruction: String::from("or"),
                                }),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }
                Instruction::BitwiseXor => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a ^ b),
                        _ => {
                            return Err(self.err(
                                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                    instruction: String::from("xor"),
                                }),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }
                Instruction::ShiftLeft => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b < 0 || b >= 64 {
                                panic!("shift out of range");
                            }
                            Value::Integer(a << b)
                        }
                        _ => {
                            return Err(self.err(
                                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                    instruction: String::from("shift"),
                                }),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }
                Instruction::ShiftRight => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b < 0 || b >= 64 {
                                panic!("shift out of range");
                            }
                            Value::Integer(a >> b)
                        }
                        _ => {
                            return Err(self.err(
                                ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                    instruction: String::from("shift"),
                                }),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                Instruction::Equal => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let lv = l.tag();
                    let rv = r.tag();
                    let result = match (lv, rv) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a == b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a == b),
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a == b),
                        (Value::String(a), Value::String(b)) => Value::Boolean(a == b),
                        (Value::Character(a), Value::Character(b)) => Value::Boolean(a == b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::NotEqual => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let lv = l.tag();
                    let rv = r.tag();
                    let result = match (lv, rv) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a != b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a != b),
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a != b),
                        (Value::String(a), Value::String(b)) => Value::Boolean(a != b),
                        (Value::Character(a), Value::Character(b)) => Value::Boolean(a != b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Less => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a < b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a < b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::LessOrEqual => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a <= b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a <= b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::Greater => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a > b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a > b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Instruction::GreaterOrEqual => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a >= b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a >= b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }

                Instruction::MakeArray(count) => {
                    let start = self.stack.len().saturating_sub(count);
                    let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Array(items));
                }
                Instruction::MakeTuple(count) => {
                    let start = self.stack.len().saturating_sub(count);
                    let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Tuple(items));
                }
                Instruction::MakeStruct(name, count) => {
                    let start = self.stack.len().saturating_sub(count);
                    let fields: Vec<Value<'a>> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Structure(name, fields));
                }
                Instruction::MakeUnion(name) => {
                    let value = self.stack.pop().unwrap_or(Value::Void);
                    self.stack.push(Value::Union(name, Box::new(value)));
                }

                Instruction::GetField(index) => {
                    let value = self.stack.pop().unwrap_or(Value::Void);
                    let result = match value {
                        Value::Structure(_, fields) => {
                            fields.into_iter().nth(index).unwrap_or(Value::Void)
                        }
                        Value::Tuple(fields) => {
                            fields.into_iter().nth(index).unwrap_or(Value::Void)
                        }
                        Value::Array(items) => items.into_iter().nth(index).unwrap_or(Value::Void),
                        _ => Value::Void,
                    };
                    self.stack.push(result);
                }
                Instruction::GetIndex => {
                    let idx = self.stack.pop().unwrap_or(Value::Void);
                    let base = self.stack.pop().unwrap_or(Value::Void);
                    let Value::Integer(i) = idx else {
                        return Err(self.err(
                            ErrorKind::DataStructure(DataStructureError::NotIndexable),
                            Span::void(),
                        ));
                    };
                    let result = match base {
                        Value::Array(items) => {
                            if i < 0 || i as usize >= items.len() {
                                panic!("array index out of bounds");
                            }
                            items.into_iter().nth(i as usize).unwrap_or(Value::Void)
                        }
                        Value::Tuple(fields) => {
                            fields.into_iter().nth(i as usize).unwrap_or(Value::Void)
                        }
                        Value::Pointer(inner) => *inner,
                        _ => {
                            return Err(self.err(
                                ErrorKind::DataStructure(DataStructureError::NotIndexable),
                                Span::void(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }
                Instruction::SetIndex => {
                    let _target = self.stack.pop().unwrap_or(Value::Void);
                    let value = self.stack.pop().unwrap_or(Value::Void);
                    self.stack.push(value);
                }

                Instruction::SizeOf(size) => {
                    self.stack.push(Value::Integer(size as i64));
                }

                Instruction::Jump(dest) => {
                    ip = dest;
                }
                Instruction::JumpIf(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if top.is_truthy() {
                        ip = dest;
                    }
                }
                Instruction::JumpIfNot(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if !top.is_truthy() {
                        ip = dest;
                    }
                }

                Instruction::ReturnSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Return(v.clone()));
                    break v;
                }
                Instruction::BreakSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Break);
                    break v;
                }
                Instruction::ContinueSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Continue);
                    break v;
                }

                Instruction::Call(name, arity) => {
                    let start = self.stack.len().saturating_sub(arity);
                    let args: Vec<Value<'a>> = self.stack.drain(start..).collect();

                    if let Some(foreign) = self.foreigns.get(&name).cloned() {
                        let result = foreign.call(&args);
                        self.stack.push(result);
                        continue;
                    }

                    let function = self.functions.get(&name).cloned();

                    if let Some(function) = function {
                        let result = self.call_function(&function, args)?;
                        self.stack.push(result);
                    } else {
                        return Err(self.err(
                            ErrorKind::Function(FunctionError::Undefined {
                                name: name.to_string(),
                            }),
                            Span::void(),
                        ));
                    }
                }

                Instruction::CallForeign(name, arity) => {
                    let start = self.stack.len().saturating_sub(arity);
                    let args: Vec<Value<'a>> = self.stack.drain(start..).collect();

                    if let Some(foreign) = self.foreigns.get(&name).cloned() {
                        let result = foreign.call(&args);
                        self.stack.push(result);
                    } else {
                        return Err(self.err(
                            ErrorKind::Function(FunctionError::Undefined {
                                name: name.to_string(),
                            }),
                            Span::void(),
                        ));
                    }
                }

                Instruction::Return => {
                    break self.stack.pop().unwrap_or(Value::Void);
                }
            }
        };

        self.frames.pop();
        let frame_base = self.frames.last().copied().unwrap_or(0);
        self.stack
            .truncate(frame_base.max(self.stack.len().saturating_sub(0)));

        Ok(result)
    }

    fn call_function(
        &mut self,
        function: &AxoFn<'a>,
        args: Vec<Value<'a>>,
    ) -> Result<Value<'a>, InterpretError<'a>> {
        let frame_base = self.stack.len();
        self.frames.push(frame_base);

        for value in &args {
            self.stack.push(value.clone());
        }

        let mut compiler = Compiler::new();

        for param in &function.members {
            if let AnalysisKind::Binding(binding) = &param.kind {
                let name = match &binding.target.kind {
                    AnalysisKind::Usage(n) => *n,
                    AnalysisKind::Symbol(t) => t.name,
                    _ => continue,
                };
                compiler.define_local(name);
            }
        }

        let result = if let Some(body) = &function.body {
            let mut chunk = Chunk::new();
            compiler.compile_one(body, &mut chunk)?;
            self.run_frame(&chunk, frame_base)?
        } else {
            Value::Void
        };

        let returned = match self.signal.take() {
            Some(Signal::Return(v)) => v,
            _ => result,
        };

        self.stack.truncate(frame_base);
        self.frames.pop();

        Ok(returned)
    }

    fn run_frame(
        &mut self,
        chunk: &Chunk<'a>,
        frame_base: usize,
    ) -> Result<Value<'a>, InterpretError<'a>> {
        let mut ip = 0;

        let result = loop {
            if ip >= chunk.ops.len() {
                break self.stack.pop().unwrap_or(Value::Void);
            }

            let op = chunk.ops[ip].clone();
            ip += 1;

            match &op {
                Instruction::Load(slot) => {
                    let value = self
                        .stack
                        .get(frame_base + slot)
                        .cloned()
                        .unwrap_or(Value::Void);
                    self.stack.push(value);
                }
                Instruction::Store(slot) => {
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    let target = frame_base + slot;
                    while self.stack.len() <= target {
                        self.stack.push(Value::Void);
                    }
                    self.stack[target] = value;
                }
                Instruction::Jump(dest) => {
                    ip = *dest;
                }
                Instruction::JumpIf(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if top.is_truthy() {
                        ip = *dest;
                    }
                }
                Instruction::JumpIfNot(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if !top.is_truthy() {
                        ip = *dest;
                    }
                }
                Instruction::ReturnSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Return(v.clone()));
                    break v;
                }
                Instruction::BreakSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Break);
                    break v;
                }
                Instruction::ContinueSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Continue);
                    break v;
                }
                Instruction::Call(name, arity) => {
                    let name = *name;
                    let arity = *arity;
                    let start = self.stack.len().saturating_sub(arity);
                    let args: Vec<Value<'a>> = self.stack.drain(start..).collect();

                    if let Some(foreign) = self.foreigns.get(&name).cloned() {
                        let result = foreign.call(&args);
                        self.stack.push(result);
                        continue;
                    }

                    let function = self.functions.get(&name).cloned();
                    if let Some(function) = function {
                        let result = self.call_function(&function, args)?;
                        self.stack.push(result);
                    } else {
                        return Err(self.err(
                            ErrorKind::Function(FunctionError::Undefined {
                                name: name.to_string(),
                            }),
                            Span::void(),
                        ));
                    }
                }
                other => {
                    self.dispatch(other.clone())?;
                }
            }
        };

        Ok(result)
    }

    fn dispatch(&mut self, op: Instruction<'a>) -> Result<(), InterpretError<'a>> {
        match op {
            Instruction::Void => self.stack.push(Value::Void),
            Instruction::Integer(n) => self.stack.push(Value::Integer(n)),
            Instruction::Float(f) => self.stack.push(Value::Float(f)),
            Instruction::Boolean(b) => self.stack.push(Value::Boolean(b)),
            Instruction::Character(c) => self.stack.push(Value::Character(c)),
            Instruction::String(s) => self.stack.push(Value::String(s)),
            Instruction::Pop => {
                self.stack.pop();
            }
            Instruction::Dup => {
                let top = self.stack.last().cloned().unwrap_or(Value::Void);
                self.stack.push(top);
            }
            Instruction::LoadGlobal(name) => {
                let value = self.globals.get(&name).cloned().unwrap_or(Value::Void);
                self.stack.push(value);
            }
            Instruction::StoreGlobal(name) => {
                let value = self.stack.last().cloned().unwrap_or(Value::Void);
                self.globals.insert(name, value);
            }
            Instruction::DefineGlobal(name) => {
                let value = self.stack.last().cloned().unwrap_or(Value::Void);
                self.globals.insert(name, value);
            }
            Instruction::EnterBlock | Instruction::LeaveBlock => {}
            Instruction::SizeOf(size) => self.stack.push(Value::Integer(size as i64)),
            Instruction::MakeArray(count) => {
                let start = self.stack.len().saturating_sub(count);
                let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                self.stack.push(Value::Array(items));
            }
            Instruction::MakeTuple(count) => {
                let start = self.stack.len().saturating_sub(count);
                let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                self.stack.push(Value::Tuple(items));
            }
            Instruction::MakeStruct(name, count) => {
                let start = self.stack.len().saturating_sub(count);
                let fields: Vec<Value<'a>> = self.stack.drain(start..).collect();
                self.stack.push(Value::Structure(name, fields));
            }
            Instruction::MakeUnion(name) => {
                let value = self.stack.pop().unwrap_or(Value::Void);
                self.stack.push(Value::Union(name, Box::new(value)));
            }
            Instruction::GetField(index) => {
                let value = self.stack.pop().unwrap_or(Value::Void);
                let result = match value {
                    Value::Structure(_, fields) => {
                        fields.into_iter().nth(index).unwrap_or(Value::Void)
                    }
                    Value::Tuple(fields) => fields.into_iter().nth(index).unwrap_or(Value::Void),
                    Value::Array(items) => items.into_iter().nth(index).unwrap_or(Value::Void),
                    _ => Value::Void,
                };
                self.stack.push(result);
            }
            Instruction::GetIndex => {
                let idx = self.stack.pop().unwrap_or(Value::Void);
                let base = self.stack.pop().unwrap_or(Value::Void);
                let Value::Integer(i) = idx else {
                    return Err(self.err(
                        ErrorKind::DataStructure(DataStructureError::NotIndexable),
                        Span::void(),
                    ));
                };
                let result = match base {
                    Value::Array(items) => {
                        if i < 0 || i as usize >= items.len() {
                            panic!("index out of bounds");
                        }
                        items.into_iter().nth(i as usize).unwrap_or(Value::Void)
                    }
                    Value::Tuple(fields) => {
                        fields.into_iter().nth(i as usize).unwrap_or(Value::Void)
                    }
                    Value::Pointer(inner) => *inner,
                    _ => {
                        return Err(self.err(
                            ErrorKind::DataStructure(DataStructureError::NotIndexable),
                            Span::void(),
                        ))
                    }
                };
                self.stack.push(result);
            }
            Instruction::SetIndex => {
                let _target = self.stack.pop();
                let value = self.stack.pop().unwrap_or(Value::Void);
                self.stack.push(value);
            }
            Instruction::Negate => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                let result = match v {
                    Value::Integer(n) => Value::Integer(n.wrapping_neg()),
                    Value::Float(f) => Value::Float(-f),
                    _ => return Err(self.err(ErrorKind::Negate, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::Not => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                match v {
                    Value::Boolean(b) => self.stack.push(Value::Boolean(!b)),
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Instruction::BitwiseNot => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                match v {
                    Value::Integer(n) => self.stack.push(Value::Integer(!n)),
                    _ => {
                        return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("not"),
                            }),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::AddressOf => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                self.stack.push(Value::Pointer(Box::new(v)));
            }
            Instruction::Deref => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                match v {
                    Value::Pointer(inner) => self.stack.push(*inner),
                    _ => {
                        return Err(self.err(
                            ErrorKind::Variable(VariableError::DereferenceNonPointer),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::Add => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_add(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::Subtract => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_sub(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::Multiply => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_mul(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::Divide => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b == 0 {
                            panic!("division by zero");
                        }
                        if b == -1 && a == i64::MIN {
                            panic!("integer overflow");
                        }
                        Value::Integer(a / b)
                    }
                    (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::Modulus => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b == 0 {
                            panic!("modulus by zero");
                        }
                        if b == -1 && a == i64::MIN {
                            panic!("integer overflow");
                        }
                        Value::Integer(a % b)
                    }
                    (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::And => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Boolean(a), Value::Boolean(b)) => {
                        self.stack.push(Value::Boolean(a && b))
                    }
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Instruction::Or => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Boolean(a), Value::Boolean(b)) => {
                        self.stack.push(Value::Boolean(a || b))
                    }
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Instruction::Xor => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Boolean(a), Value::Boolean(b)) => {
                        self.stack.push(Value::Boolean(a ^ b))
                    }
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Instruction::BitwiseAnd => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Integer(a & b))
                    }
                    _ => {
                        return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("and"),
                            }),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::BitwiseOr => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Integer(a | b))
                    }
                    _ => {
                        return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("or"),
                            }),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::BitwiseXor => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Integer(a ^ b))
                    }
                    _ => {
                        return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("xor"),
                            }),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::ShiftLeft => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b < 0 || b >= 64 {
                            panic!("shift out of range");
                        }
                        self.stack.push(Value::Integer(a << b));
                    }
                    _ => {
                        return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("shift"),
                            }),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::ShiftRight => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b < 0 || b >= 64 {
                            panic!("shift out of range");
                        }
                        self.stack.push(Value::Integer(a >> b));
                    }
                    _ => {
                        return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("shift"),
                            }),
                            Span::void(),
                        ))
                    }
                }
            }
            Instruction::Equal => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let lv = l.tag();
                let rv = r.tag();
                let result = match (lv, rv) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a == b),
                    (Value::Float(a), Value::Float(b)) => Value::Boolean(a == b),
                    (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a == b),
                    (Value::String(a), Value::String(b)) => Value::Boolean(a == b),
                    (Value::Character(a), Value::Character(b)) => Value::Boolean(a == b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::NotEqual => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let lv = l.tag();
                let rv = r.tag();
                let result = match (lv, rv) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a != b),
                    (Value::Float(a), Value::Float(b)) => Value::Boolean(a != b),
                    (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a != b),
                    (Value::String(a), Value::String(b)) => Value::Boolean(a != b),
                    (Value::Character(a), Value::Character(b)) => Value::Boolean(a != b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Instruction::Less => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Boolean(a < b))
                    }
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a < b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            Instruction::LessOrEqual => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Boolean(a <= b))
                    }
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a <= b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            Instruction::Greater => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Boolean(a > b))
                    }
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a > b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            Instruction::GreaterOrEqual => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Boolean(a >= b))
                    }
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a >= b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn err(&self, kind: ErrorKind<'a>, span: Span) -> InterpretError<'a> {
        InterpretError::new(kind, span)
    }
}
