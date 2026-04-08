use crate::{
    analyzer::{Analysis, AnalysisKind},
    data::{Invoke, Str},
    interpreter::{Entity, Instruction, Interpreter, Opcode, Value},
    tracker::Span,
};

impl<'error> Interpreter<'error> {
    pub fn native(&mut self, name: &str, index: usize) {
        self.insert_entity(Str::from(name.to_string()), Entity::Foreign(index));
    }

    pub fn address(&self, name: &Str<'error>) -> Option<usize> {
        match self.get_entity(name) {
            Some(Entity::Function(Some(address))) => Some(*address),
            _ => None,
        }
    }

    fn emit(&mut self, opcode: Opcode, span: Span<'error>) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch(&mut self, index: usize, opcode: Opcode) {
        self.code[index].opcode = opcode;
    }

    fn scope(
        &mut self,
        stem: Str<'error>,
        action: fn(&mut Self, Vec<Analysis<'error>>),
        analyses: Vec<Analysis<'error>>,
    ) {
        let previous = self.current_module;
        self.current_module = stem;
        action(self, analyses);
        self.current_module = previous;
    }

    pub fn compile(&mut self) {
        let mut modules: Vec<_> = self.modules.keys().cloned().collect();
        modules.sort();

        for module in &modules {
            let analyses = self.modules.get(module).cloned().unwrap_or_default();
            self.current_module = module.clone();
            self.declare(analyses);
        }

        for module in &modules {
            let analyses = self.modules.get(module).cloned().unwrap_or_default();
            self.current_module = module.clone();
            self.generate(analyses);
        }

        if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
            self.emit(Opcode::Halt, span);
        }

        let calls = std::mem::take(&mut self.calls);
        for (index, target) in calls {
            if let Some(address) = self.address(&target) {
                self.patch(index, Opcode::Call(address));
            } else {
                self.patch(index, Opcode::Trap);
            }
        }
    }

    fn declare(&mut self, analyses: Vec<Analysis<'error>>) {
        for analysis in analyses {
            match analysis.kind {
                AnalysisKind::Structure(structure) => {
                    let items = Self::members(structure.members);
                    self.insert_entity(structure.target, Entity::Structure(analysis.typing.identity, items));
                }
                AnalysisKind::Union(union) => {
                    let items = Self::members(union.members);
                    self.insert_entity(union.target, Entity::Union(analysis.typing.identity, items));
                }
                AnalysisKind::Function(function) => {
                    if !matches!(function.interface, crate::data::Interface::C) {
                        self.insert_entity(function.target, Entity::Function(None));
                    }
                }
                AnalysisKind::Module(stem, inner) => self.scope(stem.clone(), Self::declare, inner),
                _ => {}
            }
        }
    }

    fn generate(&mut self, analyses: Vec<Analysis<'error>>) {
        let mut entry = None;

        for analysis in analyses {
            match &analysis.kind {
                AnalysisKind::Function(function) if function.entry => {
                    entry = Some((function.clone(), analysis.span.clone()));
                }
                AnalysisKind::Function(function) => {
                    self.define(function.clone(), analysis.span.clone());
                }
                AnalysisKind::Module(stem, inner) => {
                    self.scope(stem.clone(), Self::generate, inner.clone());
                }
                AnalysisKind::Binding(_) => self.walk(analysis),
                _ => {}
            }
        }

        if let Some((function, span)) = entry {
            self.define(function, span);
        }
    }

    fn define(
        &mut self,
        function: crate::data::Function<
            Str<'error>,
            Analysis<'error>,
            Option<Box<Analysis<'error>>>,
            Option<crate::resolver::Type<'error>>,
        >,
        span: Span<'error>,
    ) {
        let bypass = self.code.len();
        self.emit(Opcode::Jump(0), span.clone());

        let address = self.code.len();
        self.insert_entity(function.target, Entity::Function(Some(address)));

        let bindings = self.bindings.clone();
        let memory = self.memory_top;

        let mut members = function.members.clone();
        members.reverse();

        for member in members {
            match &member.kind {
                AnalysisKind::Usage(target) => {
                    let address = self.memory_top;
                    self.memory_top += 1;
                    self.bindings.insert(*target, address);
                    self.emit(Opcode::Store(address), span.clone());
                }
                AnalysisKind::Binding(binding) => {
                    if let AnalysisKind::Usage(target) = &binding.target.kind {
                        let address = self.memory_top;
                        self.memory_top += 1;
                        self.bindings.insert(*target, address);
                        self.emit(Opcode::Store(address), span.clone());
                    }
                }
                _ => {}
            }
        }

        if let Some(body) = function.body {
            self.walk(*body);
        }

        self.function_frames
            .insert(address, (memory, self.memory_top - memory));
        self.emit(Opcode::Return, span.clone());
        self.bindings = bindings;
        self.memory_top = memory;
        self.patch(bypass, Opcode::Jump(self.code.len()));
    }

    fn position(&self, typing: &crate::resolver::Type<'error>, field: &str) -> Option<usize> {
        let mut current = typing;
        while let crate::resolver::TypeKind::Pointer { target } = &current.kind {
            current = target;
        }

        for entity in self.entities.values() {
            if let Entity::Structure(id, members) | Entity::Union(id, members) = entity {
                if *id == current.identity {
                    return members.iter().position(|item| item == field);
                }
            }
        }

        None
    }

    fn namespace(&self, analysis: &Analysis<'error>) -> bool {
        if let AnalysisKind::Usage(name) = &analysis.kind {
            self.has_module(name)
                || matches!(
                    self.get_entity(name),
                    Some(Entity::Structure(..) | Entity::Union(..))
                )
        } else {
            false
        }
    }

    fn field(&self, target: &Analysis<'error>, field: &Str<'error>) -> Option<usize> {
        self.position(&target.typing, field.as_str().unwrap_or_default())
    }

    fn invoke(&mut self, invoke: Invoke<Str<'error>, Analysis<'error>>, span: Span<'error>) {
        let count = invoke.members.len();
        for member in invoke.members {
            self.walk(member);
        }

        match self.get_entity(&invoke.target) {
            Some(Entity::Foreign(index)) => self.emit(Opcode::CallForeign(*index, count), span),
            Some(Entity::Function(Some(address))) => self.emit(Opcode::Call(*address), span),
            Some(Entity::Function(None)) | None => {
                let place = self.code.len();
                self.emit(Opcode::Call(0), span);
                self.calls.push((place, invoke.target));
            }
            _ => self.emit(Opcode::Trap, span),
        }
    }

    fn access(
        &mut self,
        target: Box<Analysis<'error>>,
        member: Box<Analysis<'error>>,
        span: Span<'error>,
    ) {
        if self.namespace(&target) {
            match member.kind {
                AnalysisKind::Usage(name) => {
                    if let Some(&address) = self.bindings.get(&name) {
                        self.emit(Opcode::Load(address), span);
                    } else {
                        self.emit(Opcode::Trap, span);
                    }
                }
                AnalysisKind::Invoke(invoke) => self.invoke(invoke, span),
                _ => self.emit(Opcode::Trap, span),
            }
            return;
        }

        if let AnalysisKind::Usage(name) = member.kind {
            let index = self.field(&target, &name);

            self.walk(*target);

            if let Some(index) = index {
                self.emit(Opcode::ExtractField(index), span);
            } else {
                self.emit(Opcode::Trap, span);
            }
        } else {
            self.walk(*target);
            self.emit(Opcode::Trap, span);
        }
    }

    fn walk(&mut self, node: Analysis<'error>) {
        let span = node.span;
        match node.kind {
            AnalysisKind::Integer { value, .. } => self.emit(Opcode::Push(Value::Integer(value as i64)), span),
            AnalysisKind::Float { value, .. } => self.emit(Opcode::Push(Value::Float(f64::from(value))), span),
            AnalysisKind::Boolean { value } => self.emit(Opcode::Push(Value::Boolean(value)), span),
            AnalysisKind::Character { value } => self.emit(Opcode::Push(Value::Character(value as char)), span),
            AnalysisKind::String { value } => self.emit(Opcode::Push(Value::Text(value.to_string())), span),
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
                self.emit(Opcode::MakeStructure(size), span);
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
                self.emit(Opcode::LogicAnd, span);
            }
            AnalysisKind::LogicalOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::LogicOr, span);
            }
            AnalysisKind::LogicalNot(operand) => {
                self.walk(*operand);
                self.emit(Opcode::LogicNot, span);
            }
            AnalysisKind::LogicalXOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::LogicXor, span);
            }
            AnalysisKind::BitwiseAnd(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::BitAnd, span);
            }
            AnalysisKind::BitwiseOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::BitOr, span);
            }
            AnalysisKind::BitwiseNot(operand) => {
                self.walk(*operand);
                self.emit(Opcode::BitNot, span);
            }
            AnalysisKind::BitwiseXOr(left, right) => {
                self.walk(*left);
                self.walk(*right);
                self.emit(Opcode::BitXor, span);
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
                    self.emit(Opcode::Index, span.clone());
                }
            }
            AnalysisKind::Invoke(invoke) => self.invoke(invoke, span),
            AnalysisKind::Block(statements) => {
                for statement in statements {
                    self.walk(statement);
                }
            }
            AnalysisKind::Conditional(condition, truthy, falsy) => {
                self.walk(*condition);
                let check = self.code.len();
                self.emit(Opcode::JumpFalse(0), span.clone());
                self.walk(*truthy);

                if let Some(alternative) = falsy {
                    let bypass = self.code.len();
                    self.emit(Opcode::Jump(0), span.clone());
                    self.patch(check, Opcode::JumpFalse(self.code.len()));
                    self.walk(*alternative);
                    self.patch(bypass, Opcode::Jump(self.code.len()));
                } else {
                    self.patch(check, Opcode::JumpFalse(self.code.len()));
                }
            }
            AnalysisKind::While(condition, body) => {
                let start = self.code.len();
                self.walk(*condition);
                let check = self.code.len();
                self.emit(Opcode::JumpFalse(0), span.clone());

                self.loops.push((start, Vec::new()));
                self.walk(*body);
                self.emit(Opcode::Jump(start), span.clone());

                if let Some((_, breaks)) = self.loops.pop() {
                    let end = self.code.len();
                    self.patch(check, Opcode::JumpFalse(end));

                    for index in breaks {
                        self.patch(index, Opcode::Jump(end));
                    }
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                let place = self.code.len();
                if self.loops.is_empty() {
                    self.emit(Opcode::Trap, span);
                } else {
                    self.emit(Opcode::Jump(0), span);
                    if let Some(state) = self.loops.last_mut() {
                        state.1.push(place);
                    }
                }
            }
            AnalysisKind::Continue(_) => {
                if let Some(state) = self.loops.last() {
                    self.emit(Opcode::Jump(state.0), span);
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Binding(binding) => {
                if let (Some(value), AnalysisKind::Usage(target)) = (binding.value, binding.target.kind) {
                    self.walk(*value);
                    let address = self.memory_top;
                    self.memory_top += 1;
                    self.bindings.insert(target, address);
                    self.emit(Opcode::Store(address), span);
                }
            }
            AnalysisKind::Usage(name) => {
                if let Some(&address) = self.bindings.get(&name) {
                    self.emit(Opcode::Load(address), span);
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Assign(name, value) => {
                self.walk(*value);
                if let Some(&address) = self.bindings.get(&name) {
                    self.emit(Opcode::Store(address), span);
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Function(function) => self.define(function, span),
            AnalysisKind::Return(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                self.emit(Opcode::Return, span);
            }
            AnalysisKind::Module(stem, analyses) => self.scope(stem.clone(), Self::generate, analyses),
            AnalysisKind::Access(left, right) => self.access(left, right, span),
            AnalysisKind::Store(target, value) => {
                let mut valid = false;

                if let AnalysisKind::Access(left, right) = &target.kind {
                    if let (AnalysisKind::Usage(variable), AnalysisKind::Usage(field)) = (&left.kind, &right.kind) {
                        if let (Some(&address), Some(index)) = (self.bindings.get(variable), self.position(&left.typing, field.as_str().unwrap_or_default())) {
                            self.walk(*value);
                            self.emit(Opcode::StoreField(address, index), span.clone());
                            valid = true;
                        }
                    }
                }

                if !valid {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Structure(_) | AnalysisKind::Union(_) => {}
            AnalysisKind::Constructor(aggregate) => {
                let size = aggregate.members.len();
                for member in aggregate.members {
                    if let AnalysisKind::Assign(_, value) = member.kind {
                        self.walk(*value);
                    } else {
                        self.walk(member);
                    }
                }
                self.emit(Opcode::MakeStructure(size), span);
            }
            _ => self.emit(Opcode::Trap, span),
        }
    }
}

impl Interpreter<'_> {
    fn extract_name<'error>(analysis: &Analysis<'error>) -> Option<Str<'error>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => Some(*name),
            AnalysisKind::Assign(name, _) => Some(*name),
            AnalysisKind::Binding(binding) => Self::extract_name(binding.target.as_ref()),
            _ => None,
        }
    }

    fn members<'error>(analyses: Vec<Analysis<'error>>) -> Vec<Str<'error>> {
        analyses.iter().filter_map(Self::extract_name).collect()
    }
}
