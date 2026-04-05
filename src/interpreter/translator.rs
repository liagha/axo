use crate::analyzer::{Analysis, AnalysisKind};
use crate::data::Str;
use crate::internal::hash::Map;
use crate::interpreter::{Instruction, Opcode, Value};
use crate::tracker::Span;

pub struct Translator<'error> {
    pub code: Vec<Instruction<'error>>,
    pub current_module: Str<'error>,
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
            current_module: Str::default(),
            memory: 0,
            bindings: Map::new(),
            natives,
            loops: Vec::new(),
            functions: Map::new(),
            calls: Vec::new(),
        }
    }

    pub fn native(&mut self, identifier: &str, index: usize) {
        self.natives.insert(identifier.to_string(), index);

        if let Some(prefix) = identifier.split('_').next() {
            self.natives.insert(format!("{}.{}", prefix, identifier), index);
        }
    }

    fn emit(&mut self, opcode: Opcode, span: Span<'error>) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch(&mut self, position: usize, opcode: Opcode) {
        self.code[position].opcode = opcode;
    }

    fn namespaced(&self, target: &str) -> String {
        if target.contains('.') {
            target.to_string()
        } else {
            format!("{}.{}", self.current_module.as_str().unwrap_or(""), target)
        }
    }

    pub fn compile(mut self, nodes: Vec<Analysis<'error>>) -> Vec<Instruction<'error>> {
        for node in nodes {
            self.walk(node);
        }

        if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
            self.emit(Opcode::Halt, span);
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
                let namespaced_target = self.namespaced(&target);

                println!("DEBUG INVOKE: target='{}', namespaced='{}'", target, namespaced_target);
                println!("DEBUG NATIVES MAP: {:?}", self.natives);

                if let Some(position) = self.natives.get(&namespaced_target).or_else(|| self.natives.get(&target)) {
                    self.emit(Opcode::NativeCall(*position, count), span);
                } else if let Some(address) = self.functions.get(&namespaced_target) {
                    self.emit(Opcode::Call(*address), span);
                } else {
                    let position = self.code.len();
                    self.emit(Opcode::Call(0), span);
                    self.calls.push((position, namespaced_target));
                }
            }            AnalysisKind::Block(statements) => {
                for statement in statements {
                    self.walk(statement);
                }
            }
            AnalysisKind::Conditional(condition, truthy, falsy) => {
                self.walk(*condition);
                let position = self.code.len();
                self.emit(Opcode::JumpFalse(0), span);
                self.walk(*truthy);

                if let Some(alternative) = falsy {
                    let bypass = self.code.len();
                    self.emit(Opcode::Jump(0), span);
                    self.patch(position, Opcode::JumpFalse(self.code.len()));
                    self.walk(*alternative);
                    self.patch(bypass, Opcode::Jump(self.code.len()));
                } else {
                    self.patch(position, Opcode::JumpFalse(self.code.len()));
                }
            }
            AnalysisKind::While(condition, body) => {
                let start = self.code.len();
                self.walk(*condition);
                let position = self.code.len();
                self.emit(Opcode::JumpFalse(0), span);

                self.loops.push((start, Vec::new()));
                self.walk(*body);
                self.emit(Opcode::Jump(start), span);

                let (_, breaks) = self.loops.pop().unwrap();
                let end = self.code.len();
                self.patch(position, Opcode::JumpFalse(end));

                for index in breaks {
                    self.patch(index, Opcode::Jump(end));
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                let length = self.loops.len();
                if length > 0 {
                    let index = self.code.len();
                    self.emit(Opcode::Jump(0), span);
                    self.loops[length - 1].1.push(index);
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
                self.patch(bypass, Opcode::Jump(end));
            }
            AnalysisKind::Return(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                self.emit(Opcode::Return, span);
            }
            AnalysisKind::Module(stem, analyses) => {
                let previous_module = self.current_module;
                self.current_module = stem;

                for analysis in analyses {
                    self.walk(analysis);
                }

                self.current_module = previous_module;
            }
            AnalysisKind::Access(lhs, rhs) => {
                if let AnalysisKind::Invoke(invoke) = rhs.kind {
                    let count = invoke.members.len();
                    for member in invoke.members {
                        self.walk(member);
                    }

                    let lhs_name = match lhs.kind {
                        AnalysisKind::Usage(name) => name,
                        _ => {
                            println!("WARNING: Complex access patterns not yet supported: {:?}", lhs.kind);
                            return;
                        }
                    };

                    let target = format!("{}.{}", lhs_name, invoke.target);

                    if let Some(position) = self.natives.get(&target) {
                        self.emit(Opcode::NativeCall(*position, count), span);
                    } else {
                        let position = self.code.len();
                        self.emit(Opcode::Call(0), span);
                        self.calls.push((position, target));
                    }
                }
            }
            _ => {
                println!("WARNING: Unhandled AST node: {:?}", node.kind);
            }
        }
    }
}

