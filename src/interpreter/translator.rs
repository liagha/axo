// src/interpreter/translator.rs
use {
    crate::{
        analyzer::{Analysis, AnalysisKind},
        data::Str,
        interpreter::{Entity, Instruction, Machine, Opcode, Value},
        tracker::Span,
    },
};

impl<'error> Machine<'error> {
    pub fn native(&mut self, identifier: &str, index: usize) {
        self.entities
            .insert(identifier.to_string(), Entity::Foreign(index));

        if let Some((prefix, suffix)) = identifier.split_once('_') {
            self.entities
                .insert(format!("{}.{}", prefix, suffix), Entity::Foreign(index));
            self.entities.insert(
                format!("{}.{}", prefix, identifier),
                Entity::Foreign(index),
            );
        }
    }

    pub fn address(&self, identifier: &str) -> Option<usize> {
        match self.entities.get(identifier) {
            Some(Entity::Function(Some(address))) => Some(*address),
            _ if !identifier.contains('.') => self.entities.iter().find_map(|(name, entity)| {
                if name.ends_with(&format!(".{}", identifier)) {
                    match entity {
                        Entity::Function(Some(address)) => Some(*address),
                        _ => None,
                    }
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    fn emit(&mut self, opcode: Opcode, span: Span<'error>) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch(&mut self, position: usize, opcode: Opcode) {
        self.code[position].opcode = opcode;
    }

    fn namespace(&self, target: &str) -> String {
        if target.contains('.') {
            target.to_string()
        } else if let Some(prefix) = self.current_module.as_str() {
            if prefix.is_empty() {
                target.to_string()
            } else {
                format!("{}.{}", prefix, target)
            }
        } else {
            target.to_string()
        }
    }

    fn scoped_module(
        &mut self,
        stem: Str<'error>,
        action: fn(&mut Self, Vec<Analysis<'error>>),
        analyses: Vec<Analysis<'error>>,
    ) {
        let previous = self.current_module;
        self.current_module = Str::from(self.namespace(&stem.to_string()));
        action(self, analyses);
        self.current_module = previous;
    }

    pub fn compile(&mut self) {
        let mut modules: Vec<_> = self.modules.keys().cloned().collect();
        modules.sort();

        for module in &modules {
            self.entities
                .insert(module.to_string(), Entity::Module);
        }

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
        for (position, identifier, target) in calls {
            if let Some(address) = self.address(&identifier).or_else(|| self.address(&target)) {
                self.patch(position, Opcode::Call(address));
            } else {
                self.patch(position, Opcode::Trap);
            }
        }
    }

    fn declare(&mut self, analyses: Vec<Analysis<'error>>) {
        for analysis in analyses {
            match analysis.kind {
                AnalysisKind::Structure(structure) => {
                    let identifier = self.namespace(&structure.target.to_string());
                    let members = Self::member_names(structure.members);
                    self.entities.insert(identifier, Entity::Structure(members));
                }
                AnalysisKind::Union(union) => {
                    let identifier = self.namespace(&union.target.to_string());
                    let members = Self::member_names(union.members);
                    self.entities.insert(identifier, Entity::Union(members));
                }
                AnalysisKind::Function(function) => {
                    if !matches!(function.interface, crate::data::Interface::C) {
                        let identifier = self.namespace(&function.target.to_string());
                        self.entities.insert(identifier, Entity::Function(None));
                    }
                }
                AnalysisKind::Module(stem, analyses) => {
                    let identifier = self.namespace(&stem.to_string());
                    self.entities.insert(identifier, Entity::Module);
                    self.scoped_module(stem.clone(), Self::declare, analyses);
                }
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
                    self.define_function(function.clone(), analysis.span.clone());
                }
                AnalysisKind::Module(stem, inner) => {
                    self.scoped_module(stem.clone(), Self::generate, inner.clone());
                }
                AnalysisKind::Binding(_) => self.walk(analysis),
                _ => {}
            }
        }

        if let Some((function, span)) = entry {
            self.define_function(function, span);
        }
    }

    fn define_function(
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
        let identifier = self.namespace(&function.target.to_string());
        self.entities
            .insert(identifier, Entity::Function(Some(address)));

        let saved_bindings = self.bindings.clone();
        let saved_memory = self.memory_top;

        let mut members = function.members.clone();
        members.reverse();

        for member in members {
            if let AnalysisKind::Usage(target) = member.kind {
                let address = self.memory_top;
                self.memory_top += 1;
                self.bindings.insert(target.to_string(), address);
                self.emit(Opcode::Store(address), span.clone());
            }
        }

        if let Some(body) = function.body {
            self.walk(*body);
        }

        self.emit(Opcode::Return, span.clone());
        self.bindings = saved_bindings;
        self.memory_top = saved_memory;
        self.patch(bypass, Opcode::Jump(self.code.len()));
    }

    fn walk(&mut self, node: Analysis<'error>) {
        let span = node.span;
        match node.kind {
            AnalysisKind::Integer { value, .. } => {
                self.emit(Opcode::Push(Value::Integer(value as i64)), span)
            }
            AnalysisKind::Float { value, .. } => {
                self.emit(Opcode::Push(Value::Float(f64::from(value))), span)
            }
            AnalysisKind::Boolean { value } => self.emit(Opcode::Push(Value::Boolean(value)), span),
            AnalysisKind::Character { value } => {
                self.emit(Opcode::Push(Value::Character(value as char)), span)
            }
            AnalysisKind::String { value } => {
                self.emit(Opcode::Push(Value::Text(value.to_string())), span)
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
            AnalysisKind::Invoke(invoke) => {
                let count = invoke.members.len();
                for member in invoke.members {
                    self.walk(member);
                }

                let target = invoke.target.to_string();
                let identifier = self.namespace(&target);

                match self
                    .entities
                    .get(&identifier)
                    .or_else(|| self.entities.get(&target))
                {
                    Some(Entity::Foreign(position)) => {
                        self.emit(Opcode::CallForeign(*position, count), span);
                    }
                    Some(Entity::Function(Some(address))) => {
                        self.emit(Opcode::Call(*address), span);
                    }
                    Some(Entity::Function(None)) | None => {
                        let position = self.code.len();
                        self.emit(Opcode::Call(0), span);
                        self.calls.push((position, identifier, target));
                    }
                    _ => self.emit(Opcode::Trap, span),
                }
            }
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

                    for position in breaks {
                        self.patch(position, Opcode::Jump(end));
                    }
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                let position = self.code.len();
                if self.loops.is_empty() {
                    self.emit(Opcode::Trap, span);
                } else {
                    self.emit(Opcode::Jump(0), span);
                    if let Some(state) = self.loops.last_mut() {
                        state.1.push(position);
                    }
                }
            }
            AnalysisKind::Continue(_) => {
                let target = self.loops.last().map(|state| state.0);
                if let Some(position) = target {
                    self.emit(Opcode::Jump(position), span);
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Binding(binding) => {
                if let Some(value) = binding.value {
                    if let AnalysisKind::Usage(target) = binding.target.kind {
                        self.walk(*value);
                        let address = self.memory_top;
                        self.memory_top += 1;
                        self.bindings.insert(target.to_string(), address);
                        self.emit(Opcode::Store(address), span);
                    }
                }
            }
            AnalysisKind::Usage(identifier) => {
                let target = identifier.to_string();
                if let Some(&address) = self.bindings.get(&target) {
                    self.emit(Opcode::Load(address), span);
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Assign(identifier, value) => {
                self.walk(*value);
                let target = identifier.to_string();
                if let Some(&address) = self.bindings.get(&target) {
                    self.emit(Opcode::Store(address), span);
                } else {
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Function(function) => self.define_function(function, span),
            AnalysisKind::Return(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                self.emit(Opcode::Return, span);
            }
            AnalysisKind::Module(stem, analyses) => {
                self.scoped_module(stem.clone(), Self::generate, analyses);
            }
            AnalysisKind::Access(left, right) => {
                if let AnalysisKind::Invoke(invoke) = right.kind {
                    let count = invoke.members.len();
                    for member in invoke.members {
                        self.walk(member);
                    }

                    if let AnalysisKind::Usage(name) = left.kind {
                        let target = format!("{}.{}", name, invoke.target);
                        let identifier = self.namespace(&target);

                        match self
                            .entities
                            .get(&identifier)
                            .or_else(|| self.entities.get(&target))
                            .or_else(|| self.entities.get(&invoke.target.to_string()))
                        {
                            Some(Entity::Foreign(position)) => {
                                self.emit(Opcode::CallForeign(*position, count), span);
                            }
                            Some(Entity::Function(Some(address))) => {
                                self.emit(Opcode::Call(*address), span);
                            }
                            Some(Entity::Function(None)) | None => {
                                let position = self.code.len();
                                self.emit(Opcode::Call(0), span);
                                self.calls.push((position, identifier, target));
                            }
                            _ => self.emit(Opcode::Trap, span),
                        }
                    } else {
                        self.emit(Opcode::Trap, span);
                    }
                } else if let AnalysisKind::Usage(field_name) = right.kind {
                    self.walk(*left);

                    let target_field = field_name.to_string();
                    let mut found_index = None;

                    let mut possible_indices = Vec::new();
                    for entity in self.entities.values() {
                        if let Entity::Structure(members) = entity {
                            if let Some(index) = members.iter().position(|m| m == &target_field) {
                                possible_indices.push(index);
                            }
                        }
                    }

                    if !possible_indices.is_empty() {
                        possible_indices.sort();
                        found_index = Some(possible_indices[0]);
                    }

                    if let Some(index) = found_index {
                        self.emit(Opcode::ExtractField(index), span);
                    } else {
                        self.emit(Opcode::Trap, span);
                    }
                } else {
                    self.walk(*left);
                    self.emit(Opcode::Trap, span);
                }
            }
            AnalysisKind::Store(target, value) => {
                self.walk(*value);
                self.walk(*target);
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

use crate::format::{Show, Stencil};

impl Machine<'_> {
    fn extract_name(analysis: &Analysis<'_>) -> Option<String> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => Some(name.to_string()),
            AnalysisKind::Assign(name, _) => Some(name.to_string()),
            AnalysisKind::Binding(binding) => Self::extract_name(binding.target.as_ref()),
            _ => None,
        }
    }

    fn member_names(analyses: Vec<Analysis<'_>>) -> Vec<String> {
        analyses
            .iter()
            .filter_map(|analysis| Self::extract_name(analysis))
            .collect()
    }
}
