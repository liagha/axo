use crate::{
    analyzer::{Analysis, AnalysisKind},
    data::{
        Invoke, Str,
        memory::take,
    },
    interpreter::{error::ErrorKind, Address, Entity, Index, Instruction, Interpreter, Opcode, Value},
    tracker::Span,
};

impl<'error> Interpreter<'error> {
    pub fn native(&mut self, name: &str, index: Index) {
        self.insert_entity(Str::from(name.to_string()), Entity::Foreign(index));
    }

    pub fn address(&self, name: &Str<'error>) -> Option<Address> {
        match self.raw_entity(name) {
            Some(Entity::Function(Some(address))) => Some(address),
            _ => None,
        }
    }

    fn emit(&mut self, opcode: Opcode, span: Span<'error>) {
        self.code.push(Instruction { opcode, span });
    }

    fn patch(&mut self, index: Address, opcode: Opcode) {
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

        let entry = Str::from("main".to_string());

        if self.address(&entry).is_some() {
            if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
                let place = self.code.len();
                self.emit(Opcode::Call(0), span);
                self.calls.push((place, entry));
            }
        }

        if let Some(span) = self.code.last().map(|instruction| instruction.span.clone()) {
            self.emit(Opcode::Halt, span);
        }

        let calls = take(&mut self.calls);
        for (index, target) in calls {
            if let Some(address) = self.address(&target) {
                self.patch(index, Opcode::Call(address));
            } else {
                self.patch(index, Opcode::Trap(ErrorKind::MissingSymbol));
            }
        }
    }

    pub fn extend(&mut self, module: Str<'error>, analyses: Vec<Analysis<'error>>) -> Address {
        let start = self.code.len();
        let previous = self.current_module;

        self.current_module = module;
        self.declare(analyses.clone());
        self.generate(analyses);
        self.current_module = previous;

        let calls = take(&mut self.calls);
        for (index, target) in calls {
            if let Some(address) = self.address(&target) {
                self.patch(index, Opcode::Call(address));
            } else {
                self.patch(index, Opcode::Trap(ErrorKind::MissingSymbol));
            }
        }

        start
    }

    fn declare(&mut self, analyses: Vec<Analysis<'error>>) {
        for analysis in analyses {
            match analysis.kind {
                AnalysisKind::Structure(structure) => {
                    let items = Self::members(structure.members);
                    self.insert_entity(
                        structure.target,
                        Entity::Structure {
                            identity: analysis.typing.identity,
                            members: items,
                        },
                    );
                }
                AnalysisKind::Union(union) => {
                    let items = Self::members(union.members);
                    self.insert_entity(
                        union.target,
                        Entity::Union {
                            identity: analysis.typing.identity,
                            members: items,
                        },
                    );
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
        for analysis in analyses {
            match &analysis.kind {
                AnalysisKind::Function(function) => {
                    if !matches!(function.interface, crate::data::Interface::C) {
                        self.define(function.clone(), analysis.span.clone());
                    }
                }
                AnalysisKind::Module(stem, inner) => {
                    self.scope(stem.clone(), Self::generate, inner.clone());
                }
                _ => self.walk(analysis),
            }
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

        let entities = self.entities.clone();
        let memory = self.memory_top;

        let mut members = function.members.clone();
        members.reverse();

        for member in members {
            match &member.kind {
                AnalysisKind::Usage(target) => {
                    let address = self.memory_top;
                    self.memory_top += 1;
                    self.insert_entity(*target, Entity::Variable { address, typing: member.typing.clone() });
                    self.emit(Opcode::Store(address), span.clone());
                }
                AnalysisKind::Binding(binding) => {
                    if let AnalysisKind::Usage(target) = &binding.target.kind {
                        let address = self.memory_top;
                        self.memory_top += 1;
                        self.insert_entity(*target, Entity::Variable { address, typing: member.typing.clone() });
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
        self.entities = entities;
        self.memory_top = memory;
        self.patch(bypass, Opcode::Jump(self.code.len()));
    }

    fn aggregate(&self, typing: &crate::resolver::Type<'error>) -> Option<&Entity<'error>> {
        let mut current = typing;
        while let crate::resolver::TypeKind::Pointer { target } = &current.kind {
            current = target;
        }

        if let crate::resolver::TypeKind::Structure(aggregate)
        | crate::resolver::TypeKind::Union(aggregate) = &current.kind
        {
            if let Some(entity @ (Entity::Structure { .. } | Entity::Union { .. })) =
                self.get_entity(&aggregate.target)
            {
                return Some(entity);
            }
        }

        for (name, entity) in &self.entities {
            match entity {
                Entity::Structure { identity, .. } | Entity::Union { identity, .. }
                if *identity == current.identity =>
                    {
                        return Some(entity);
                    }
                _ => {}
            }
        }

        None
    }

    fn position(&self, typing: &crate::resolver::Type<'error>, field: &str) -> Option<Index> {
        match self.aggregate(typing) {
            Some(Entity::Structure { members, .. }) | Some(Entity::Union { members, .. }) => {
                members.iter().position(|item| item == field)
            }
            _ => None,
        }
    }

    fn raw_entity(&self, name: &Str<'error>) -> Option<Entity<'error>> {
        self.get_entity(name)
            .cloned()
            .or_else(|| self.has_module(name).then_some(Entity::Module))
    }

    fn symbol(&self, analysis: &Analysis<'error>) -> Option<Str<'error>> {
        match &analysis.kind {
            AnalysisKind::Usage(name) => Some(*name),
            AnalysisKind::Access(target, member) if self.namespace(target) => match &member.kind {
                AnalysisKind::Usage(name) => Some(*name),
                AnalysisKind::Access(_, _) => self.symbol(member),
                _ => None,
            },
            _ => None,
        }
    }

    fn entity(&self, analysis: &Analysis<'error>) -> Option<Entity<'error>> {
        self.symbol(analysis)
            .and_then(|name| self.raw_entity(&name))
            .filter(|entity| !matches!(entity, Entity::Variable { .. }))
    }

    fn namespace(&self, analysis: &Analysis<'error>) -> bool {
        matches!(
            self.entity(analysis),
            Some(Entity::Module | Entity::Structure { .. } | Entity::Union { .. })
        )
    }

    fn field(&self, target: &Analysis<'error>, field: &Str<'error>) -> Option<Index> {
        let typing = match &target.kind {
            AnalysisKind::Usage(name) => self
                .variable(name)
                .map(|(_, typing)| typing)
                .unwrap_or(&target.typing),
            _ => &target.typing,
        };

        self.position(typing, field.as_str().unwrap_or_default())
    }

    fn variable(&self, name: &Str<'error>) -> Option<(Address, &crate::resolver::Type<'error>)> {
        match self.get_entity(name) {
            Some(Entity::Variable { address, typing }) => Some((*address, typing)),
            _ => None,
        }
    }

    fn alias(&mut self, target: Str<'error>, value: &Analysis<'error>) -> bool {
        if let Some(entity) = self.entity(value) {
            self.insert_entity(target, entity);
            true
        } else {
            false
        }
    }

    fn invoke(&mut self, invoke: Invoke<Box<Analysis<'error>>, Analysis<'error>>, span: Span<'error>) {
        let count = invoke.members.len();
        for member in invoke.members {
            self.walk(member);
        }

        match self.entity(invoke.target.as_ref()) {
            Some(Entity::Foreign(index)) => self.emit(Opcode::CallForeign(index, count), span),
            Some(Entity::Function(Some(address))) => self.emit(Opcode::Call(address), span),
            Some(Entity::Function(None)) => {
                let place = self.code.len();
                self.emit(Opcode::Call(0), span);
                if let Some(target) = self.symbol(invoke.target.as_ref()) {
                    self.calls.push((place, target));
                } else {
                    self.patch(place, Opcode::Trap(ErrorKind::MissingSymbol));
                }
            }
            None => self.emit(Opcode::Trap(ErrorKind::MissingSymbol), span),
            _ => self.emit(Opcode::Trap(ErrorKind::InvalidCall), span),
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
                    if let Some((address, _)) = self.variable(&name) {
                        self.emit(Opcode::Load(address), span);
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
            let index = self.field(&target, &name);

            self.walk(*target);

            if let Some(index) = index {
                self.emit(Opcode::ExtractField(index), span);
            } else {
                self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span);
            }
        } else {
            self.walk(*target);
            self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span);
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
                    let mut init = false;

                    if let Some(value) = binding.value {
                        if self.alias(target, &value) {
                            return;
                        }

                        self.walk(*value);
                        init = true;
                    }

                    let address = self.memory_top;
                    self.memory_top += 1;

                    self.insert_entity(
                        target,
                        Entity::Variable {
                            address,
                            typing: binding.annotation,
                        },
                    );

                    if init {
                        self.emit(Opcode::Store(address), span);
                    }
                }
            }
            AnalysisKind::Usage(name) => {
                if let Some((address, _)) = self.variable(&name) {
                    self.emit(Opcode::Load(address), span);
                } else {
                    self.emit(Opcode::Trap(ErrorKind::MissingSymbol), span);
                }
            }
            AnalysisKind::Assign(name, value) => {
                self.walk(*value);
                if let Some((address, _)) = self.variable(&name) {
                    self.emit(Opcode::Store(address), span);
                } else {
                    self.emit(Opcode::Trap(ErrorKind::InvalidStore), span);
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
                        if let (Some((address, _)), Some(index)) = (self.variable(variable), self.position(&left.typing, field.as_str().unwrap_or_default())) {
                            self.walk(*value);
                            self.emit(Opcode::StoreField(address, index), span.clone());
                            valid = true;
                        }
                    }
                }

                if !valid {
                    self.emit(Opcode::Trap(ErrorKind::InvalidStore), span);
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
            _ => self.emit(Opcode::Trap(ErrorKind::InvalidAccess), span),
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

    fn members(analyses: Vec<Analysis>) -> Vec<Str> {
        analyses.iter().filter_map(Self::extract_name).collect()
    }
}
