use {
    crate::emitter::interpreter::error::InterpretError,
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::{BindingKind, Str},
        emitter::{
            interpreter::instruction::Instruction, DataStructureError, ErrorKind, FunctionError,
            VariableError,
        },
        resolver::{Type, TypeKind},
    },
};

pub struct Chunk<'a> {
    pub ops: Vec<Instruction<'a>>,
}

impl<'a> Chunk<'a> {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    fn emit(&mut self, op: Instruction<'a>) -> usize {
        self.ops.push(op);
        self.ops.len() - 1
    }

    fn patch_jump(&mut self, at: usize, target: usize) {
        match &mut self.ops[at] {
            Instruction::Jump(dest) | Instruction::JumpIf(dest) | Instruction::JumpIfNot(dest) => {
                *dest = target
            }
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
                chunk.emit(Instruction::Integer(*value as i64));
            }
            AnalysisKind::Float { value, .. } => {
                chunk.emit(Instruction::Float(value.0));
            }
            AnalysisKind::Boolean { value } => {
                chunk.emit(Instruction::Boolean(*value));
            }
            AnalysisKind::Character { value } => {
                chunk.emit(Instruction::Character(*value));
            }
            AnalysisKind::String { value } => {
                chunk.emit(Instruction::String(*value));
            }

            AnalysisKind::Negate(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Instruction::Negate);
            }
            AnalysisKind::LogicalNot(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Instruction::Not);
            }
            AnalysisKind::BitwiseNot(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Instruction::BitwiseNot);
            }
            AnalysisKind::AddressOf(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Instruction::AddressOf);
            }
            AnalysisKind::Dereference(operand) => {
                self.compile_one(operand, chunk)?;
                chunk.emit(Instruction::Deref);
            }

            AnalysisKind::Add(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Add);
            }
            AnalysisKind::Subtract(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Subtract);
            }
            AnalysisKind::Multiply(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Multiply);
            }
            AnalysisKind::Divide(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Divide);
            }
            AnalysisKind::Modulus(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Modulus);
            }
            AnalysisKind::LogicalAnd(l, r) => {
                self.compile_one(l, chunk)?;
                let short = chunk.emit(Instruction::JumpIfNot(0));
                chunk.emit(Instruction::Pop);
                self.compile_one(r, chunk)?;
                let end = chunk.here();
                chunk.patch_jump(short, end);
            }
            AnalysisKind::LogicalOr(l, r) => {
                self.compile_one(l, chunk)?;
                let short = chunk.emit(Instruction::JumpIf(0));
                chunk.emit(Instruction::Pop);
                self.compile_one(r, chunk)?;
                let end = chunk.here();
                chunk.patch_jump(short, end);
            }
            AnalysisKind::LogicalXOr(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Xor);
            }
            AnalysisKind::BitwiseAnd(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::BitwiseAnd);
            }
            AnalysisKind::BitwiseOr(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::BitwiseOr);
            }
            AnalysisKind::BitwiseXOr(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::BitwiseXor);
            }
            AnalysisKind::ShiftLeft(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::ShiftLeft);
            }
            AnalysisKind::ShiftRight(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::ShiftRight);
            }

            AnalysisKind::Equal(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Equal);
            }
            AnalysisKind::NotEqual(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::NotEqual);
            }
            AnalysisKind::Less(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Less);
            }
            AnalysisKind::LessOrEqual(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::LessOrEqual);
            }
            AnalysisKind::Greater(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::Greater);
            }
            AnalysisKind::GreaterOrEqual(l, r) => {
                self.compile_one(l, chunk)?;
                self.compile_one(r, chunk)?;
                chunk.emit(Instruction::GreaterOrEqual);
            }

            AnalysisKind::Array(items) => {
                let count = items.len();
                for item in items {
                    self.compile_one(item, chunk)?;
                }
                chunk.emit(Instruction::MakeArray(count));
            }
            AnalysisKind::Tuple(items) => {
                let count = items.len();
                for item in items {
                    self.compile_one(item, chunk)?;
                }
                chunk.emit(Instruction::MakeTuple(count));
            }

            AnalysisKind::SizeOf(typing) => {
                let size = self.sizeof_type(typing);
                chunk.emit(Instruction::SizeOf(size));
            }

            AnalysisKind::Usage(_) | AnalysisKind::Symbol(_) => {
                let name = match &analysis.kind {
                    AnalysisKind::Usage(name) => *name,
                    AnalysisKind::Symbol(target) => target.name,
                    _ => unreachable!(),
                };
                if let Some(slot) = self.local(&name) {
                    chunk.emit(Instruction::Load(slot));
                } else {
                    chunk.emit(Instruction::LoadGlobal(name));
                }
            }

            AnalysisKind::Assign(_, value) | AnalysisKind::Write(_, value) => {
                let name = match &analysis.kind {
                    AnalysisKind::Assign(name, _) => *name,
                    AnalysisKind::Write(target, _) => target.name,
                    _ => unreachable!(),
                };
                self.compile_one(value, chunk)?;
                if let Some(slot) = self.local(&name) {
                    chunk.emit(Instruction::Store(slot));
                    chunk.emit(Instruction::Load(slot));
                } else {
                    chunk.emit(Instruction::StoreGlobal(name));
                    chunk.emit(Instruction::LoadGlobal(name));
                }
            }
            AnalysisKind::Store(target, value) => {
                self.compile_one(value, chunk)?;
                self.compile_one(target, chunk)?;
                chunk.emit(Instruction::SetIndex);
            }

            AnalysisKind::Binding(binding) => {
                let name = match &binding.target.kind {
                    AnalysisKind::Usage(n) => *n,
                    AnalysisKind::Symbol(t) => t.name,
                    _ => {
                        chunk.emit(Instruction::Void);
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
                    chunk.emit(Instruction::DefineGlobal(name));
                    chunk.emit(Instruction::LoadGlobal(name));
                } else {
                    let slot = self.define_local(name);
                    chunk.emit(Instruction::Store(slot));
                    chunk.emit(Instruction::Load(slot));
                }
            }

            AnalysisKind::Block(analyses) => {
                self.depth += 1;
                let base = self.locals.len();
                chunk.emit(Instruction::EnterBlock);

                let mut last_void = true;
                for inner in analyses {
                    self.compile_one(inner, chunk)?;
                    last_void = false;
                }

                self.locals.truncate(base);
                self.depth -= 1;

                chunk.emit(Instruction::LeaveBlock);

                if last_void {
                    chunk.emit(Instruction::Void);
                }
            }

            AnalysisKind::Conditional(condition, then, otherwise) => {
                self.compile_one(condition, chunk)?;
                let to_else = chunk.emit(Instruction::JumpIfNot(0));

                self.compile_one(then, chunk)?;
                let to_end = chunk.emit(Instruction::Jump(0));

                let else_start = chunk.here();
                chunk.patch_jump(to_else, else_start);

                if let Some(branch) = otherwise {
                    self.compile_one(branch, chunk)?;
                } else {
                    chunk.emit(Instruction::Void);
                }

                let end = chunk.here();
                chunk.patch_jump(to_end, end);
            }

            AnalysisKind::While(condition, body) => {
                let loop_start = chunk.here();
                self.loop_starts.push(loop_start);
                self.loop_exits.push(Vec::new());

                self.compile_one(condition, chunk)?;
                let exit_jump = chunk.emit(Instruction::JumpIfNot(0));

                self.compile_one(body, chunk)?;
                chunk.emit(Instruction::Pop);
                chunk.emit(Instruction::Jump(loop_start));

                let exit = chunk.here();
                chunk.patch_jump(exit_jump, exit);

                let exits = self.loop_exits.pop().unwrap_or_default();
                for at in exits {
                    chunk.patch_jump(at, exit);
                }
                self.loop_starts.pop();

                chunk.emit(Instruction::Integer(0));
            }

            AnalysisKind::Return(value) => {
                if let Some(v) = value {
                    self.compile_one(v, chunk)?;
                } else {
                    chunk.emit(Instruction::Void);
                }
                chunk.emit(Instruction::ReturnSignal);
            }
            AnalysisKind::Break(value) => {
                if let Some(v) = value {
                    self.compile_one(v, chunk)?;
                } else {
                    chunk.emit(Instruction::Void);
                }
                chunk.emit(Instruction::BreakSignal);
                let at = chunk.emit(Instruction::Jump(0));
                if let Some(exits) = self.loop_exits.last_mut() {
                    exits.push(at);
                }
            }
            AnalysisKind::Continue(value) => {
                if let Some(v) = value {
                    self.compile_one(v, chunk)?;
                } else {
                    chunk.emit(Instruction::Void);
                }
                chunk.emit(Instruction::ContinueSignal);
                let top = self.loop_starts.last().copied().unwrap_or(0);
                chunk.emit(Instruction::Jump(top));
            }

            AnalysisKind::Call(target, args) => {
                for arg in args {
                    self.compile_one(arg, chunk)?;
                }
                chunk.emit(Instruction::Call(target.name, args.len()));
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
                chunk.emit(Instruction::Call(name, invoke.members.len()));
            }

            AnalysisKind::Function(function) => {
                chunk.emit(Instruction::DefineGlobal(function.target));
                chunk.emit(Instruction::Void);
            }
            AnalysisKind::Structure(_) | AnalysisKind::Union(_) => {
                chunk.emit(Instruction::Void);
            }
            AnalysisKind::Module(_, inner) => {
                for analysis in inner {
                    self.compile_one(analysis, chunk)?;
                    chunk.emit(Instruction::Pop);
                }
                chunk.emit(Instruction::Void);
            }

            AnalysisKind::Constructor(constructor) => {
                let name = constructor.target;
                let count = constructor.members.len();
                for member in &constructor.members {
                    self.compile_one(member, chunk)?;
                }
                chunk.emit(Instruction::MakeStruct(name, count));
            }
            AnalysisKind::Pack(target, values) => {
                let mut sorted = values.clone();
                sorted.sort_by_key(|(i, _)| *i);
                let count = sorted.len();
                for (_, analysis) in &sorted {
                    self.compile_one(analysis, chunk)?;
                }
                chunk.emit(Instruction::MakeStruct(target.name, count));
            }
            AnalysisKind::Composite(composite) => {
                let name = composite.target.name;
                let count = composite.members.len();
                for member in &composite.members {
                    self.compile_one(member, chunk)?;
                }
                chunk.emit(Instruction::MakeStruct(name, count));
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
                chunk.emit(Instruction::GetField(index));
            }
            AnalysisKind::Slot(target, index) => {
                self.compile_one(target, chunk)?;
                chunk.emit(Instruction::GetField(*index));
            }
            AnalysisKind::Index(index) => {
                self.compile_one(&index.target, chunk)?;
                if !index.members.is_empty() {
                    self.compile_one(&index.members[0], chunk)?;
                    chunk.emit(Instruction::GetIndex);
                }
            }
        }

        Ok(())
    }

    fn field_index(&self, typing: &Type<'a>, name: &Str<'a>) -> Option<usize> {
        match &typing.kind {
            TypeKind::Structure(agg) | TypeKind::Union(agg) => {
                agg.members.iter().position(|m| match &m.kind {
                    TypeKind::Binding(b) => b.target == *name,
                    _ => false,
                })
            }
            _ => None,
        }
    }
}
