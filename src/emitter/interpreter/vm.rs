// src/emitter/interpreter/vm.rs

use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::{Function, Str},
        emitter::{
            interpreter::{
                compiler::{Chunk, Compiler},
                error::InterpretError,
                foreign::Foreign,
                op::Op,
                value::Value,
            },
            BitwiseError, ControlFlowError, DataStructureError, ErrorKind, FunctionError,
            VariableError,
        },
        internal::hash::Map,
        resolver::{Type, TypeKind},
        tracker::Span,
    },
};

type AxoFn<'a> = Function<
Str<'a>,
Analysis<'a>,
Option<Box<Analysis<'a>>>,
Option<Type<'a>>,
>;

enum Signal<'a> {
    Return(Value<'a>),
    Break(Value<'a>),
    Continue(Value<'a>),
}

pub struct Vm<'a> {
    stack: Vec<Value<'a>>,
    globals: Map<Str<'a>, Value<'a>>,
    functions: Map<Str<'a>, AxoFn<'a>>,
    foreigns: Map<Str<'a>, Foreign<'a>>,
    frames: Vec<usize>,
    signal: Option<Signal<'a>>,
}

impl<'a> Vm<'a> {
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
                break Value::Void;
            }

            let op = chunk.ops[ip].clone();
            ip += 1;

            match op {
                Op::Void => self.stack.push(Value::Void),
                Op::Integer(n) => self.stack.push(Value::Integer(n)),
                Op::Float(f) => self.stack.push(Value::Float(f)),
                Op::Boolean(b) => self.stack.push(Value::Boolean(b)),
                Op::Character(c) => self.stack.push(Value::Character(c)),
                Op::String(s) => self.stack.push(Value::String(s)),

                Op::Pop => {
                    self.stack.pop();
                }
                Op::Dup => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    self.stack.push(top);
                }

                Op::Load(slot) => {
                    let base = *self.frames.last().unwrap_or(&0);
                    let value = self.stack.get(base + slot).cloned().unwrap_or(Value::Void);
                    self.stack.push(value);
                }
                Op::Store(slot) => {
                    let base = *self.frames.last().unwrap_or(&0);
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    let target = base + slot;
                    while self.stack.len() <= target {
                        self.stack.push(Value::Void);
                    }
                    self.stack[target] = value;
                }
                Op::LoadGlobal(name) => {
                    let value = self.globals.get(&name).cloned().unwrap_or(Value::Void);
                    self.stack.push(value);
                }
                Op::StoreGlobal(name) => {
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    self.globals.insert(name, value);
                }
                Op::DefineGlobal(name) => {
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    self.globals.insert(name, value);
                }

                Op::EnterBlock => {}
                Op::LeaveBlock => {}

                Op::Negate => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let result = match v {
                        Value::Integer(n) => Value::Integer(n.wrapping_neg()),
                        Value::Float(f) => Value::Float(-f),
                        _ => return Err(self.err(ErrorKind::Negate, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Not => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let result = match v {
                        Value::Boolean(b) => Value::Boolean(!b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::BitwiseNot => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let result = match v {
                        Value::Integer(n) => Value::Integer(!n),
                        _ => return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("not"),
                            }),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }
                Op::AddressOf => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.stack.push(Value::Pointer(Box::new(v)));
                }
                Op::Deref => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    let inner = match v {
                        Value::Pointer(inner) => *inner,
                        _ => return Err(self.err(
                            ErrorKind::Variable(VariableError::DereferenceNonPointer),
                            Span::void(),
                        )),
                    };
                    self.stack.push(inner);
                }

                Op::Add => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_add(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Subtract => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_sub(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Multiply => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_mul(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Divide => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b == 0 { panic!("division by zero"); }
                            if b == -1 && a == i64::MIN { panic!("integer overflow"); }
                            Value::Integer(a / b)
                        }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Modulus => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b == 0 { panic!("modulus by zero"); }
                            if b == -1 && a == i64::MIN { panic!("integer overflow"); }
                            Value::Integer(a % b)
                        }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::And => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a && b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Or => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a || b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Xor => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a ^ b),
                        _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::BitwiseAnd => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a & b),
                        _ => return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("and"),
                            }),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }
                Op::BitwiseOr => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a | b),
                        _ => return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("or"),
                            }),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }
                Op::BitwiseXor => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a ^ b),
                        _ => return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("xor"),
                            }),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }
                Op::ShiftLeft => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b < 0 || b >= 64 { panic!("shift out of range"); }
                            Value::Integer(a << b)
                        }
                        _ => return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("shift"),
                            }),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }
                Op::ShiftRight => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b < 0 || b >= 64 { panic!("shift out of range"); }
                            Value::Integer(a >> b)
                        }
                        _ => return Err(self.err(
                            ErrorKind::Bitwise(BitwiseError::InvalidOperandType {
                                instruction: String::from("shift"),
                            }),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }

                Op::Equal => {
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
                Op::NotEqual => {
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
                Op::Less => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a < b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a < b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::LessOrEqual => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a <= b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a <= b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::Greater => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a > b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a > b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }
                Op::GreaterOrEqual => {
                    let r = self.stack.pop().unwrap_or(Value::Void);
                    let l = self.stack.pop().unwrap_or(Value::Void);
                    let result = match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(a >= b),
                        (Value::Float(a), Value::Float(b)) => Value::Boolean(a >= b),
                        _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                    };
                    self.stack.push(result);
                }

                Op::MakeArray(count) => {
                    let start = self.stack.len().saturating_sub(count);
                    let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Array(items));
                }
                Op::MakeTuple(count) => {
                    let start = self.stack.len().saturating_sub(count);
                    let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Tuple(items));
                }
                Op::MakeStruct(name, count) => {
                    let start = self.stack.len().saturating_sub(count);
                    let fields: Vec<Value<'a>> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Structure(name, fields));
                }
                Op::MakeUnion(name) => {
                    let value = self.stack.pop().unwrap_or(Value::Void);
                    self.stack.push(Value::Union(name, Box::new(value)));
                }

                Op::GetField(index) => {
                    let value = self.stack.pop().unwrap_or(Value::Void);
                    let result = match value {
                        Value::Structure(_, fields) => {
                            fields.into_iter().nth(index).unwrap_or(Value::Void)
                        }
                        Value::Tuple(fields) => {
                            fields.into_iter().nth(index).unwrap_or(Value::Void)
                        }
                        Value::Array(items) => {
                            items.into_iter().nth(index).unwrap_or(Value::Void)
                        }
                        _ => Value::Void,
                    };
                    self.stack.push(result);
                }
                Op::GetIndex => {
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
                        _ => return Err(self.err(
                            ErrorKind::DataStructure(DataStructureError::NotIndexable),
                            Span::void(),
                        )),
                    };
                    self.stack.push(result);
                }
                Op::SetIndex => {
                    let target = self.stack.pop().unwrap_or(Value::Void);
                    let value = self.stack.pop().unwrap_or(Value::Void);
                    self.stack.push(value);
                }

                Op::SizeOf(size) => {
                    self.stack.push(Value::Integer(size as i64));
                }

                Op::Jump(dest) => {
                    ip = dest;
                }
                Op::JumpIf(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if top.is_truthy() {
                        ip = dest;
                    }
                }
                Op::JumpIfNot(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if !top.is_truthy() {
                        ip = dest;
                    }
                }

                Op::ReturnSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Return(v.clone()));
                    break v;
                }
                Op::BreakSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Break(v.clone()));
                    break v;
                }
                Op::ContinueSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Continue(v.clone()));
                    break v;
                }

                Op::Call(name, arity) => {
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

                Op::CallForeign(name, arity) => {
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

                Op::Return => {
                    break self.stack.pop().unwrap_or(Value::Void);
                }
            }
        };

        self.frames.pop();
        let frame_base = self.frames.last().copied().unwrap_or(0);
        self.stack.truncate(frame_base.max(self.stack.len().saturating_sub(0)));

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

        for (param, value) in function.members.iter().zip(args.iter()) {
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
                break Value::Void;
            }

            let op = chunk.ops[ip].clone();
            ip += 1;

            match &op {
                Op::Load(slot) => {
                    let value = self.stack.get(frame_base + slot).cloned().unwrap_or(Value::Void);
                    self.stack.push(value);
                }
                Op::Store(slot) => {
                    let value = self.stack.last().cloned().unwrap_or(Value::Void);
                    let target = frame_base + slot;
                    while self.stack.len() <= target {
                        self.stack.push(Value::Void);
                    }
                    self.stack[target] = value;
                }
                Op::Jump(dest) => { ip = *dest; }
                Op::JumpIf(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if top.is_truthy() { ip = *dest; }
                }
                Op::JumpIfNot(dest) => {
                    let top = self.stack.last().cloned().unwrap_or(Value::Void);
                    if !top.is_truthy() { ip = *dest; }
                }
                Op::ReturnSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Return(v.clone()));
                    break v;
                }
                Op::BreakSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Break(v.clone()));
                    break v;
                }
                Op::ContinueSignal => {
                    let v = self.stack.pop().unwrap_or(Value::Void);
                    self.signal = Some(Signal::Continue(v.clone()));
                    break v;
                }
                Op::Call(name, arity) => {
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

    fn dispatch(&mut self, op: Op<'a>) -> Result<(), InterpretError<'a>> {
        match op {
            Op::Void => self.stack.push(Value::Void),
            Op::Integer(n) => self.stack.push(Value::Integer(n)),
            Op::Float(f) => self.stack.push(Value::Float(f)),
            Op::Boolean(b) => self.stack.push(Value::Boolean(b)),
            Op::Character(c) => self.stack.push(Value::Character(c)),
            Op::String(s) => self.stack.push(Value::String(s)),
            Op::Pop => { self.stack.pop(); }
            Op::Dup => {
                let top = self.stack.last().cloned().unwrap_or(Value::Void);
                self.stack.push(top);
            }
            Op::LoadGlobal(name) => {
                let value = self.globals.get(&name).cloned().unwrap_or(Value::Void);
                self.stack.push(value);
            }
            Op::StoreGlobal(name) => {
                let value = self.stack.last().cloned().unwrap_or(Value::Void);
                self.globals.insert(name, value);
            }
            Op::DefineGlobal(name) => {
                let value = self.stack.last().cloned().unwrap_or(Value::Void);
                self.globals.insert(name, value);
            }
            Op::EnterBlock | Op::LeaveBlock => {}
            Op::SizeOf(size) => self.stack.push(Value::Integer(size as i64)),
            Op::MakeArray(count) => {
                let start = self.stack.len().saturating_sub(count);
                let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                self.stack.push(Value::Array(items));
            }
            Op::MakeTuple(count) => {
                let start = self.stack.len().saturating_sub(count);
                let items: Vec<Value<'a>> = self.stack.drain(start..).collect();
                self.stack.push(Value::Tuple(items));
            }
            Op::MakeStruct(name, count) => {
                let start = self.stack.len().saturating_sub(count);
                let fields: Vec<Value<'a>> = self.stack.drain(start..).collect();
                self.stack.push(Value::Structure(name, fields));
            }
            Op::MakeUnion(name) => {
                let value = self.stack.pop().unwrap_or(Value::Void);
                self.stack.push(Value::Union(name, Box::new(value)));
            }
            Op::GetField(index) => {
                let value = self.stack.pop().unwrap_or(Value::Void);
                let result = match value {
                    Value::Structure(_, fields) => fields.into_iter().nth(index).unwrap_or(Value::Void),
                    Value::Tuple(fields) => fields.into_iter().nth(index).unwrap_or(Value::Void),
                    Value::Array(items) => items.into_iter().nth(index).unwrap_or(Value::Void),
                    _ => Value::Void,
                };
                self.stack.push(result);
            }
            Op::GetIndex => {
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
                        if i < 0 || i as usize >= items.len() { panic!("index out of bounds"); }
                        items.into_iter().nth(i as usize).unwrap_or(Value::Void)
                    }
                    Value::Tuple(fields) => fields.into_iter().nth(i as usize).unwrap_or(Value::Void),
                    Value::Pointer(inner) => *inner,
                    _ => return Err(self.err(
                        ErrorKind::DataStructure(DataStructureError::NotIndexable),
                        Span::void(),
                    )),
                };
                self.stack.push(result);
            }
            Op::SetIndex => {
                let _target = self.stack.pop();
                let value = self.stack.pop().unwrap_or(Value::Void);
                self.stack.push(value);
            }
            Op::Negate => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                let result = match v {
                    Value::Integer(n) => Value::Integer(n.wrapping_neg()),
                    Value::Float(f) => Value::Float(-f),
                    _ => return Err(self.err(ErrorKind::Negate, Span::void())),
                };
                self.stack.push(result);
            }
            Op::Not => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                match v {
                    Value::Boolean(b) => self.stack.push(Value::Boolean(!b)),
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Op::BitwiseNot => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                match v {
                    Value::Integer(n) => self.stack.push(Value::Integer(!n)),
                    _ => return Err(self.err(
                        ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: String::from("not") }),
                        Span::void(),
                    )),
                }
            }
            Op::AddressOf => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                self.stack.push(Value::Pointer(Box::new(v)));
            }
            Op::Deref => {
                let v = self.stack.pop().unwrap_or(Value::Void);
                match v {
                    Value::Pointer(inner) => self.stack.push(*inner),
                    _ => return Err(self.err(
                        ErrorKind::Variable(VariableError::DereferenceNonPointer),
                        Span::void(),
                    )),
                }
            }
            Op::Add => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_add(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Op::Subtract => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_sub(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Op::Multiply => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_mul(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Op::Divide => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b == 0 { panic!("division by zero"); }
                        if b == -1 && a == i64::MIN { panic!("integer overflow"); }
                        Value::Integer(a / b)
                    }
                    (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Op::Modulus => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                let result = match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b == 0 { panic!("modulus by zero"); }
                        if b == -1 && a == i64::MIN { panic!("integer overflow"); }
                        Value::Integer(a % b)
                    }
                    (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                };
                self.stack.push(result);
            }
            Op::And => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.stack.push(Value::Boolean(a && b)),
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Op::Or => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.stack.push(Value::Boolean(a || b)),
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Op::Xor => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.stack.push(Value::Boolean(a ^ b)),
                    _ => return Err(self.err(ErrorKind::Boolean, Span::void())),
                }
            }
            Op::BitwiseAnd => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a & b)),
                    _ => return Err(self.err(
                        ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: String::from("and") }),
                        Span::void(),
                    )),
                }
            }
            Op::BitwiseOr => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a | b)),
                    _ => return Err(self.err(
                        ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: String::from("or") }),
                        Span::void(),
                    )),
                }
            }
            Op::BitwiseXor => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a ^ b)),
                    _ => return Err(self.err(
                        ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: String::from("xor") }),
                        Span::void(),
                    )),
                }
            }
            Op::ShiftLeft => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b < 0 || b >= 64 { panic!("shift out of range"); }
                        self.stack.push(Value::Integer(a << b));
                    }
                    _ => return Err(self.err(
                        ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: String::from("shift") }),
                        Span::void(),
                    )),
                }
            }
            Op::ShiftRight => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b < 0 || b >= 64 { panic!("shift out of range"); }
                        self.stack.push(Value::Integer(a >> b));
                    }
                    _ => return Err(self.err(
                        ErrorKind::Bitwise(BitwiseError::InvalidOperandType { instruction: String::from("shift") }),
                        Span::void(),
                    )),
                }
            }
            Op::Equal => {
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
            Op::NotEqual => {
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
            Op::Less => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a < b)),
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a < b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            Op::LessOrEqual => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a <= b)),
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a <= b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            Op::Greater => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a > b)),
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Boolean(a > b)),
                    _ => return Err(self.err(ErrorKind::Normalize, Span::void())),
                }
            }
            Op::GreaterOrEqual => {
                let r = self.stack.pop().unwrap_or(Value::Void);
                let l = self.stack.pop().unwrap_or(Value::Void);
                match (l, r) {
                    (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a >= b)),
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