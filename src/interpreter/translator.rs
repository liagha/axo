use crate::{
    analyzer::{Analysis, AnalysisKind},
    data::{
        Function,
        Invoke,
        Str,
        memory::take,
    },
    interpreter::{Call, Instruction, Interpreter, Opcode, Slot, Value, error::ErrorKind},
    resolver::{Type, TypeKind},
    tracker::Span,
};

fn value_type<'a>(typing: &Type<'a>) -> Type<'a> {
    match &typing.kind {
        TypeKind::Binding(binding) => binding
            .value
            .as_deref()
            .cloned()
            .or_else(|| binding.annotation.as_deref().cloned())
            .unwrap_or_else(|| Type::from(TypeKind::Unknown)),
        _ => typing.clone(),
    }
}

fn member_name<'a>(typing: &Type<'a>) -> Option<Str<'a>> {
    match &typing.kind {
        TypeKind::Binding(binding) => Some(binding.target),
        TypeKind::Function(function) if !function.target.is_empty() => Some(function.target),
        TypeKind::Has(target) => member_name(target),
        _ => None,
    }
}

impl<'a> Interpreter<'a> {
    pub fn native(&mut self, typing: Type<'a>, index: usize) {
        self.register_call(typing.identity, typing, Call::Foreign(index));
    }

    fn emit(&mut self, opcode: Opcode, span: Span) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch(&mut self, index: usize, opcode: Opcode) {
        self.code[index].opcode = opcode;
    }

    fn scope(&mut self, stem: Str<'a>, action: fn(&mut Self, Vec<Analysis<'a>>), analyses: Vec<Analysis<'a>>) {
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
            self.current_module = *module;
            self.declare(analyses);
        }

        for module in &modules {
            let analyses = self.modules.get(module).cloned().unwrap_or_default();
            self.current_module = *module;
            self.generate(analyses);
        }

        let entry = self.calls.iter().find_map(|(&identity, items)| {
            items.first().and_then(|(typing, _)| match &typing.kind {
                TypeKind::Function(function) if function.target == Str::from("main".to_string()) => {
                    Some((identity, typing.clone()))
                }
                _ => None,
            })
        });

        if let Some((identity, typing)) = entry {
            if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
                let place = self.code.len();
                self.emit(Opcode::Call(0), span);
                self.pending.push((place, identity, typing));
            }
        }

        if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
            self.emit(Opcode::Halt, span);
        }

        self.finish_calls();
    }

    pub fn extend(&mut self, module: Str<'a>, analyses: Vec<Analysis<'a>>) -> usize {
        let start = self.code.len();
        let previous = self.current_module;

        self.current_module = module;
        self.declare(analyses.clone());
        self.generate(analyses);
        self.current_module = previous;
        self.finish_calls();

        start
    }

    fn finish_calls(&mut self) {
        let pending = take(&mut self.pending);

        for (index, identity, typing) in pending {
            match self.routine(identity, &typing) {
                Some(Call::Foreign(target)) => self.patch(index, Opcode::CallForeign(target, 0)),
                Some(Call::Local(Some(target))) => self.patch(index, Opcode::Call(target)),
                Some(Call::Local(None)) => self.pending.push((index, identity, typing)),
                None => self.patch(index, Opcode::Trap(ErrorKind::MissingSymbol)),
            }
        }
    }

    fn declare(&mut self, analyses: Vec<Analysis<'a>>) {
        for analysis in analyses {
            match analysis.kind {
                AnalysisKind::Function(function) => {
                    if !matches!(function.interface, crate::data::Interface::C) {
                        self.register_call(analysis.typing.identity, analysis.typing, Call::Local(None));
                    }
                }
                AnalysisKind::Structure(structure) => {
                    self.scope(structure.target, Self::declare, structure.members);
                }
                AnalysisKind::Union(union) => {
                    self.scope(union.target, Self::declare, union.members);
                }
                AnalysisKind::Module(stem, inner) => self.scope(stem, Self::declare, inner),
                _ => {}
            }
        }
    }

    fn generate(&mut self, analyses: Vec<Analysis<'a>>) {
        for analysis in analyses {
            match analysis.kind {
                AnalysisKind::Function(function) => {
                    if !matches!(function.interface, crate::data::Interface::C) {
                        self.define(function, analysis.typing, analysis.span.clone());
                    }
                }
                AnalysisKind::Structure(structure) => {
                    self.scope(structure.target, Self::generate, structure.members);
                }
                AnalysisKind::Union(union) => {
                    self.scope(union.target, Self::generate, union.members);
                }
                AnalysisKind::Module(stem, inner) => self.scope(stem, Self::generate, inner),
                _ => self.walk(analysis),
            }
        }
    }

    fn define(
        &mut self,
        function: Function<Str<'a>, Analysis<'a>, Option<Box<Analysis<'a>>>, Option<Type<'a>>>,
        typing: Type<'a>,
        span: Span,
    ) {
        let bypass = self.code.len();
        self.emit(Opcode::Jump(0), span.clone());

        let address = self.code.len();
        self.set_call(typing.identity, &typing, address);

        let slots = self.slots.clone();
        let memory = self.memory_top;

        let mut members = function.members.clone();
        members.reverse();

        for member in members {
            match &member.kind {
                AnalysisKind::Usage(target) => {
                    let address = self.memory_top;
                    self.memory_top += 1;
                    self.bind_slot(*target, Slot { address, typing: member.typing.clone() });
                    self.emit(Opcode::Store(address), span.clone());
                }
                AnalysisKind::Binding(binding) => {
                    if let AnalysisKind::Usage(target) = &binding.target.kind {
                        let address = self.memory_top;
                        self.memory_top += 1;
                        self.bind_slot(*target, Slot { address, typing: value_type(&member.typing) });
                        self.emit(Opcode::Store(address), span.clone());
                    }
                }
                _ => {}
            }
        }

        if let Some(body) = function.body {
            self.walk(*body);
        }

        self.function_frames.insert(address, (memory, self.memory_top - memory));
        self.emit(Opcode::Return, span.clone());
        self.slots = slots;
        self.memory_top = memory;
        self.patch(bypass, Opcode::Jump(self.code.len()));
    }

    fn member(&self, typing: &Type<'a>, field: &Str<'a>) -> bool {
        match &value_type(typing).kind {
            TypeKind::Pointer { target } => self.member(target, field),
            TypeKind::Has(target) => member_name(target).is_some_and(|name| &name == field),
            TypeKind::And(left, right) => self.member(left, field) || self.member(right, field),
            TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => aggregate.members.iter().any(|member| {
                member_name(member).is_some_and(|name| name == *field)
            }),
            _ => false,
        }
    }

    fn module(&self, analysis: &Analysis<'a>) -> bool {
        match &analysis.kind {
            AnalysisKind::Usage(name) => self.has_module(name),
            AnalysisKind::Access(left, _) => self.module(left),
            _ => matches!(analysis.typing.kind, TypeKind::Module(_)),
        }
    }

    fn invoke(&mut self, invoke: Invoke<Box<Analysis<'a>>, Analysis<'a>>, span: Span) {
        let count = invoke.members.len();
        let typing = invoke.target.typing.clone();

        for member in invoke.members {
            self.walk(member);
        }

        if typing.identity != 0 {
            match self.routine(typing.identity, &typing) {
                Some(Call::Foreign(index)) => self.emit(Opcode::CallForeign(index, count), span),
                Some(Call::Local(Some(address))) => self.emit(Opcode::Call(address), span),
                Some(Call::Local(None)) => {
                    let place = self.code.len();
                    self.emit(Opcode::Call(0), span);
                    self.pending.push((place, typing.identity, typing));
                }
                None => self.emit(Opcode::Trap(ErrorKind::MissingSymbol), span),
            }
        } else {
            self.emit(Opcode::Trap(ErrorKind::InvalidCall), span);
        }
    }

    fn access(&mut self, target: Box<Analysis<'a>>, member: Box<Analysis<'a>>, span: Span) {
        if self.module(&target) {
            match member.kind {
                AnalysisKind::Usage(name) => {
                    if let Some(slot) = self.slot(&name) {
                        self.emit(Opcode::Load(slot.address), span);
                    } else if let Some(value) = self.values.get(&name).cloned() {
                        self.emit(Opcode::Push(value), span);
                    } else {
                        self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span);
                    }
                }
                AnalysisKind::Invoke(invoke) => self.invoke(invoke, span),
                _ => self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span),
            }
            return;
        }

        if let AnalysisKind::Usage(name) = member.kind {
            if let Some(field) = self.field(&target.typing, &name) {
                self.walk(*target);
                self.emit(Opcode::ExtractField(field), span);
            } else {
                self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span);
            }
        } else {
            self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span);
        }
    }

    fn shape(&self, typing: &Type<'a>) -> Vec<Str<'a>> {
        match &value_type(typing).kind {
            TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => aggregate
                .members
                .iter()
                .filter_map(member_name)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn field(&self, typing: &Type<'a>, field: &Str<'a>) -> Option<usize> {
        self.shape(typing).iter().position(|name| name == field)
    }

    fn constructor(&mut self, typing: Type<'a>, span: Span, aggregate: crate::data::Aggregate<Str<'a>, Analysis<'a>>) {
        let identity = value_type(&typing).identity;
        let layout = self.shape(&typing);

        if layout.is_empty() {
            let size = aggregate.members.len();
            for member in aggregate.members {
                if let AnalysisKind::Assign(_, value) = member.kind {
                    self.walk(*value);
                } else {
                    self.walk(member);
                }
            }
            self.emit(Opcode::MakeStructure(identity, size), span);
            return;
        }

        let mut named = std::collections::BTreeMap::new();
        let mut values = Vec::new();

        for member in aggregate.members {
            match member.kind {
                AnalysisKind::Assign(name, value) => {
                    named.insert(name, *value);
                }
                _ => values.push(member),
            }
        }

        let size = layout.len();
        let mut values_iter = values.into_iter();

        for field in layout {
            if let Some(value) = named.remove(&field) {
                self.walk(value);
            } else if let Some(value) = values_iter.next() {
                self.walk(value);
            } else {
                self.emit(Opcode::Push(Value::Empty), span.clone());
            }
        }

        self.emit(Opcode::MakeStructure(identity, size), span);
    }

    fn walk(&mut self, node: Analysis<'a>) {
        let span = node.span.clone();
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
                self.emit(Opcode::MakeStructure(0, size), span);
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
                    self.emit(Opcode::Trap(ErrorKind::InvalidControl), span);
                }
            }
            AnalysisKind::Break(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                let place = self.code.len();
                if self.loops.is_empty() {
                    self.emit(Opcode::Trap(ErrorKind::InvalidControl), span);
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
                    self.emit(Opcode::Trap(ErrorKind::InvalidControl), span);
                }
            }
            AnalysisKind::Binding(binding) => {
                if let AnalysisKind::Usage(target) = binding.target.kind {
                    let address = self.memory_top;
                    self.memory_top += 1;
                    self.bind_slot(target, Slot { address, typing: value_type(&binding.annotation) });

                    if let Some(value) = binding.value {
                        self.walk(*value);
                        self.emit(Opcode::Store(address), span);
                    }
                }
            }
            AnalysisKind::Usage(name) => {
                if let Some(slot) = self.slot(&name) {
                    self.emit(Opcode::Load(slot.address), span);
                } else if let Some(value) = self.values.get(&name).cloned() {
                    self.emit(Opcode::Push(value), span);
                } else {
                    self.emit(Opcode::Trap(ErrorKind::MissingSymbol), span);
                }
            }
            AnalysisKind::Assign(name, value) => {
                self.walk(*value);
                if let Some(slot) = self.slot(&name) {
                    self.emit(Opcode::Store(slot.address), span);
                } else {
                    self.emit(Opcode::Trap(ErrorKind::InvalidStore), span);
                }
            }
            AnalysisKind::Function(function) => self.define(function, node.typing, span),
            AnalysisKind::Return(operand) => {
                if let Some(value) = operand {
                    self.walk(*value);
                }
                self.emit(Opcode::Return, span);
            }
            AnalysisKind::Module(stem, analyses) => self.scope(stem, Self::generate, analyses),
            AnalysisKind::Access(left, right) => self.access(left, right, span),
            AnalysisKind::Store(target, value) => {
                let valid = match &target.kind {
                    AnalysisKind::Access(left, right) => match (&left.kind, &right.kind) {
                        (AnalysisKind::Usage(name), AnalysisKind::Usage(field)) => {
                            if let Some((address, field)) = self
                                .slot(name)
                                .and_then(|slot| self.field(&left.typing, field).map(|field| (slot.address, field)))
                            {
                                self.walk(*value);
                                self.emit(Opcode::StoreField(address, field), span.clone());
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    },
                    _ => false,
                };

                if !valid {
                    self.emit(Opcode::Trap(ErrorKind::InvalidStore), span);
                }
            }
            AnalysisKind::Structure(_) | AnalysisKind::Union(_) => {}
            AnalysisKind::Constructor(aggregate) => self.constructor(node.typing, span, aggregate),
            _ => self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span),
        }
    }
}
