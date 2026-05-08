// src/emitter/interpreter/compiler.rs

use {
    crate::{
        analyzer::{Analysis, AnalysisKind, Target},
        data::{BindingKind, Str},
        emitter::{
            interpreter::{op::Op, value::Value},
            BitwiseError, ControlFlowError, DataStructureError, ErrorKind, FunctionError,
            VariableError,
        },
        resolver::{Type, TypeKind},
        tracker::Span,
    },
    crate::emitter::interpreter::error::InterpretError,
};

pub struct Chunk<'a> {
    pub ops: Vec<Op<'a>>,
}

impl<'a> Chunk<'a> {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    fn emit(&mut self, op: Op<'a>) -> usize {
        self.ops.push(op);
        self.ops.len() - 1
    }

    fn patch_jump(&mut self, at: usize, target: usize) {
        match &mut self.ops[at] {
            Op::Jump(dest) | Op::JumpIf(dest) | Op::JumpIfNot(dest) => *dest = target,
            _ => {}
        }
    }

    fn here(&self) -> usize {
        self.ops.len()
    }
}

pub struct Compiler<'a> {
    locals: Vec<Str<'a>>,
    depth: usize,
    loop_starts: Vec<usize>,
    loop_exits: Vec<Vec<usize>>,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            depth: 0,
            loop_starts: Vec::new(),
            loop_exits: Vec::new(),
        }
    }

    fn local(&self, name: &Str<'a>) -> Option<usize> {
        self.locals.iter().rposition(|n| n == name)
    }

    pub fn define_local(&mut self, name: Str<'a>) -> usize {
        self.locals.push(name);
        self.locals.len() - 1
    }

    fn sizeof_type(&self, typing: &Type<'a>) -> usize {
        match &typing.kind {
            TypeKind::Integer { size, .. } => *size / 8,
            TypeKind::Float { size } => *size / 8,
            TypeKind::Boolean => 1,
            TypeKind::Character => 4,
            TypeKind::String | TypeKind::Pointer { .. } => 8,
            TypeKind::Array { member, size } => self.sizeof_type(member) * *size,
            TypeKind::Tuple { members } => members.iter().map(|m| self.sizeof_type(m)).sum(),
            _ => 0,
        }
    }

    fn value_type<'b>(&self, typing: &'b Type<'a>) -> &'b Type<'a> {
        match &typing.kind {
            TypeKind::Binding(binding) => binding
                .value
                .as_deref()
                .or(binding.annotation.as_deref())
                .unwrap_or(typing),
            _ => typing,
        }
    }

    pub fn compile(
        &mut self,
        analyses: &[Analysis<'a>],
        chunk: &mut Chunk<'a>,
    ) -> Result<(), InterpretError<'a>> {
        for analysis in analyses {
            self.compile_one(analysis, chunk)?;
        }
        Ok(())
    }

    pub fn compile_one(
        &mut self,
        analysis: &Analysis<'a>,
        chunk: &mut Chunk<'a>,
    ) -> Result<(), InterpretError<'a>> {
        let span = analysis.span;

        match &analysis.kind {
            AnalysisKind::Integer { value, .. } => {
                chunk.emit(Op::Integer(*value as i64));
            }
            AnalysisKind::Float { value, .. } => {
                chunk.emit(Op::Float(value.0));
            }
            AnalysisKind::Boolean { value } => {
                chunk.emit(Op::Boolean(*value));
            }
            AnalysisKind::Character { value } => {
                chunk.emit(Op::Character(*value));
            }
            AnalysisKind::String { value } => {
                chunk.emit(Op::String(*value));
            }

            AnalysisKind::Negate(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Op::Negate);
            }
            AnalysisKind::LogicalNot(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Op::Not);
            }
            AnalysisKind::BitwiseNot(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Op::BitwiseNot);
            }
            AnalysisKind::AddressOf(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Op::AddressOf);
            }
            AnalysisKind::Dereference(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Op::Deref);
            }

            AnalysisKind::Add(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Add);
            }
            AnalysisKind::Subtract(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Subtract);
            }
            AnalysisKind::Multiply(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Multiply);
            }
            AnalysisKind::Divide(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Divide);
            }
            AnalysisKind::Modulus(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Modulus);
            }
            AnalysisKind::LogicalAnd(l, r) => {
                self.compile_one(l, chunk)?;
                let short = chunk.emit(Op::JumpIfNot(0));
                chunk.emit(Op::Pop);
                self.compile_one(r, chunk)?;
                let end = chunk.here();
                chunk.patch_jump(short, end);
            }
            AnalysisKind::LogicalOr(l, r) => {
                self.compile_one(l, chunk)?;
                let short = chunk.emit(Op::JumpIf(0));
                chunk.emit(Op::Pop);
                self.compile_one(r, chunk)?;
                let end = chunk.here();
                chunk.patch_jump(short, end);
            }
            AnalysisKind::LogicalXOr(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Xor);
            }
            AnalysisKind::BitwiseAnd(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::BitwiseAnd);
            }
            AnalysisKind::BitwiseOr(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::BitwiseOr);
            }
            AnalysisKind::BitwiseXOr(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::BitwiseXor);
            }
            AnalysisKind::ShiftLeft(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::ShiftLeft);
            }
            AnalysisKind::ShiftRight(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::ShiftRight);
            }

            AnalysisKind::Equal(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Equal);
            }
            AnalysisKind::NotEqual(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::NotEqual);
            }
            AnalysisKind::Less(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Less);
            }
            AnalysisKind::LessOrEqual(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::LessOrEqual);
            }
            AnalysisKind::Greater(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::Greater);
            }
            AnalysisKind::GreaterOrEqual(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Op::GreaterOrEqual);
            }

            AnalysisKind::Array(items) => {
                let count = items.len();
                for item in items {
                    self.compile_one(item, chunk)?;
                }
                chunk.emit(Op::MakeArray(count));
            }
            AnalysisKind::Tuple(items) => {
                let count = items.len();
                for item in items {
                    self.compile_one(item, chunk)?;
                }
                chunk.emit(Op::MakeTuple(count));
            }

            AnalysisKind::SizeOf(typing) => {
                let size = self.sizeof_type(typing);
                chunk.emit(Op::SizeOf(size));
            }

            AnalysisKind::Usage(name) => {
                if let Some(slot) = self.local(name) {
                    chunk.emit(Op::Load(slot));
                } else {
                    chunk.emit(Op::LoadGlobal(*name));
                }
            }
            AnalysisKind::Symbol(target) => {
                if let Some(slot) = self.local(&target.name) {
                    chunk.emit(Op::Load(slot));
                } else {
                    chunk.emit(Op::LoadGlobal(target.name));
                }
            }

            AnalysisKind::Assign(name, value) => {
                self.compile_one(value, chunk)?;
                if let Some(slot) = self.local(name) {
                    chunk.emit(Op::Store(slot));
                    chunk.emit(Op::Load(slot));
                } else {
                    chunk.emit(Op::StoreGlobal(*name));
                    chunk.emit(Op::LoadGlobal(*name));
                }
            }
            AnalysisKind::Write(target, value) => {
                self.compile_one(value, chunk)?;
                if let Some(slot) = self.local(&target.name) {
                    chunk.emit(Op::Store(slot));
                    chunk.emit(Op::Load(slot));
                } else {
                    chunk.emit(Op::StoreGlobal(target.name));
                    chunk.emit(Op::LoadGlobal(target.name));
                }
            }
            AnalysisKind::Store(target, value) => {
                self.compile_one(value, chunk)?;
                self.compile_one(target, chunk)?;
                chunk.emit(Op::SetIndex);
            }

            AnalysisKind::Binding(binding) => {
                let name = match &binding.target.kind {
                    AnalysisKind::Usage(n) => *n,
                    AnalysisKind::Symbol(t) => t.name,
                    _ => {
                        chunk.emit(Op::Void);
                        return Ok(());
                    }
                };

                if let Some(expr) = &binding.value {
                    self.compile_one(expr, chunk)?;
                } else {
                    return Err(InterpretError::new(
                        ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                            name: name.to_string(),
                        }),
                        span,
                    ));
                }

                if matches!(binding.kind, BindingKind::Static) || self.depth == 0 {
                    chunk.emit(Op::DefineGlobal(name));
                    chunk.emit(Op::LoadGlobal(name));
                } else {
                    let slot = self.define_local(name);
                    chunk.emit(Op::Store(slot));
                    chunk.emit(Op::Load(slot));
                }
            }

            AnalysisKind::Block(analyses) => {
                self.depth += 1;
                let base = self.locals.len();
                chunk.emit(Op::EnterBlock);

                let mut last_void = true;
                for inner in analyses {
                    self.compile_one(inner, chunk)?;
                    last_void = false;
                }

                let popped = self.locals.len() - base;
                self.locals.truncate(base);
                self.depth -= 1;

                chunk.emit(Op::LeaveBlock);

                if last_void {
                    chunk.emit(Op::Void);
                }
            }

            AnalysisKind::Conditional(condition, then, otherwise) => {
                self.compile_one(condition, chunk)?;
                let to_else = chunk.emit(Op::JumpIfNot(0));

                self.compile_one(then, chunk)?;
                let to_end = chunk.emit(Op::Jump(0));

                let else_start = chunk.here();
                chunk.patch_jump(to_else, else_start);

                if let Some(branch) = otherwise {
                    self.compile_one(branch, chunk)?;
                } else {
                    chunk.emit(Op::Void);
                }

                let end = chunk.here();
                chunk.patch_jump(to_end, end);
            }

            AnalysisKind::While(condition, body) => {
                let loop_start = chunk.here();
                self.loop_starts.push(loop_start);
                self.loop_exits.push(Vec::new());

                self.compile_one(condition, chunk)?;
                let exit_jump = chunk.emit(Op::JumpIfNot(0));

                self.compile_one(body, chunk)?;
                chunk.emit(Op::Pop);
                chunk.emit(Op::Jump(loop_start));

                let exit = chunk.here();
                chunk.patch_jump(exit_jump, exit);

                let exits = self.loop_exits.pop().unwrap_or_default();
                for at in exits {
                    chunk.patch_jump(at, exit);
                }
                self.loop_starts.pop();

                chunk.emit(Op::Integer(0));
            }

            AnalysisKind::Return(value) => {
                if let Some(v) = value {
                    self.compile_one(v, chunk)?;
                } else {
                    chunk.emit(Op::Void);
                }
                chunk.emit(Op::ReturnSignal);
            }
            AnalysisKind::Break(value) => {
                if let Some(v) = value {
                    self.compile_one(v, chunk)?;
                } else {
                    chunk.emit(Op::Void);
                }
                chunk.emit(Op::BreakSignal);
                let at = chunk.emit(Op::Jump(0));
                if let Some(exits) = self.loop_exits.last_mut() {
                    exits.push(at);
                }
            }
            AnalysisKind::Continue(value) => {
                if let Some(v) = value {
                    self.compile_one(v, chunk)?;
                } else {
                    chunk.emit(Op::Void);
                }
                chunk.emit(Op::ContinueSignal);
                let top = self.loop_starts.last().copied().unwrap_or(0);
                chunk.emit(Op::Jump(top));
            }

            AnalysisKind::Call(target, args) => {
                for arg in args {
                    self.compile_one(arg, chunk)?;
                }
                chunk.emit(Op::Call(target.name, args.len()));
            }
            AnalysisKind::Invoke(invoke) => {
                let name = match &invoke.target.typing.kind {
                    TypeKind::Function(f) if !f.target.is_empty() => f.target,
                    _ => {
                        return Err(InterpretError::new(
                            ErrorKind::Function(FunctionError::Undefined {
                                name: String::from("unknown"),
                            }),
                            span,
                        ))
                    }
                };
                for arg in &invoke.members {
                    self.compile_one(arg, chunk)?;
                }
                chunk.emit(Op::Call(name, invoke.members.len()));
            }

            AnalysisKind::Function(function) => {
                chunk.emit(Op::DefineGlobal(function.target));
                chunk.emit(Op::Void);
            }
            AnalysisKind::Structure(_) | AnalysisKind::Union(_) => {
                chunk.emit(Op::Void);
            }
            AnalysisKind::Module(name, inner) => {
                for analysis in inner {
                    self.compile_one(analysis, chunk)?;
                    chunk.emit(Op::Pop);
                }
                chunk.emit(Op::Void);
            }

            AnalysisKind::Constructor(constructor) => {
                let name = constructor.target;
                let count = constructor.members.len();
                for member in &constructor.members {
                    self.compile_one(member, chunk)?;
                }
                chunk.emit(Op::MakeStruct(name, count));
            }
            AnalysisKind::Pack(target, values) => {
                let mut sorted = values.clone();
                sorted.sort_by_key(|(i, _)| *i);
                let count = sorted.len();
                for (_, analysis) in &sorted {
                    self.compile_one(analysis, chunk)?;
                }
                chunk.emit(Op::MakeStruct(target.name, count));
            }
            AnalysisKind::Composite(composite) => {
                let name = composite.target.name;
                let count = composite.members.len();
                for member in &composite.members {
                    self.compile_one(member, chunk)?;
                }
                chunk.emit(Op::MakeStruct(name, count));
            }

            AnalysisKind::Access(target, member) => {
                let typing = self.value_type(&target.typing).clone();
                let field = match &member.kind {
                    AnalysisKind::Usage(n) => *n,
                    _ => {
                        return Err(InterpretError::new(
                            ErrorKind::DataStructure(
                                DataStructureError::InvalidMemberAccessExpression,
                            ),
                            span,
                        ))
                    }
                };
                let index = self.field_index(&typing, &field).unwrap_or(0);
                self.compile_one(target, chunk)?;
                chunk.emit(Op::GetField(index));
            }
            AnalysisKind::Slot(target, index) => {
                self.compile_one(target, chunk)?;
                chunk.emit(Op::GetField(*index));
            }
            AnalysisKind::Index(index) => {
                self.compile_one(&index.target, chunk)?;
                if !index.members.is_empty() {
                    self.compile_one(&index.members[0], chunk)?;
                    chunk.emit(Op::GetIndex);
                }
            }

            _ => {
                chunk.emit(Op::Void);
            }
        }

        Ok(())
    }

    fn field_index(&self, typing: &Type<'a>, name: &Str<'a>) -> Option<usize> {
        match &typing.kind {
            TypeKind::Structure(agg) | TypeKind::Union(agg) => {
                agg.members.iter().position(|m| {
                    match &m.kind {
                        TypeKind::Binding(b) => b.target == *name,
                        _ => false,
                    }
                })
            }
            _ => None,
        }
    }
}