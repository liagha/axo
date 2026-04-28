use crate::{
    analyzer::{Analysis, AnalysisKind, Target},
    data::{BindingKind, Function, Identity, Str},
    generator::{
        ControlFlowError, DataStructureError, ErrorKind, FunctionError, GenerateError,
        VariableError,
    },
    internal::{hash::Map, Artifact, RecordKind, Session},
    parser::SymbolKind,
    resolver::{Type, TypeKind},
    tracker::Span,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Sequence(Vec<Value>),
    Composite(Vec<Value>),
    Empty,
}

#[derive(Clone)]
struct Scope<'a> {
    values: Map<Identity, Value>,
    names: Map<Str<'a>, Identity>,
}

#[derive(Clone)]
struct Frame<'a> {
    scopes: Vec<Scope<'a>>,
}

#[derive(Clone)]
struct Routine<'a> {
    func: Function<Str<'a>, Analysis<'a>, Option<Box<Analysis<'a>>>, Option<Type<'a>>>,
}

#[derive(Clone)]
enum Flow {
    Value(Value),
    Return(Value),
    Break(Value),
    Continue,
}

pub struct Engine<'a> {
    next: Identity,
    globals: Scope<'a>,
    frames: Vec<Frame<'a>>,
    funcs: Map<Identity, Routine<'a>>,
    func_names: Map<Str<'a>, Identity>,
    names: Map<Str<'a>, Identity>,
}

impl Value {
    fn truth(&self) -> bool {
        match self {
            Self::Integer(value) => *value != 0,
            Self::Float(value) => *value != 0.0,
            Self::Boolean(value) => *value,
            Self::Character(value) => *value != '\0',
            Self::String(value) => !value.is_empty(),
            Self::Sequence(value) => !value.is_empty(),
            Self::Composite(value) => !value.is_empty(),
            Self::Empty => false,
        }
    }
}

impl<'a> Scope<'a> {
    fn new() -> Self {
        Self {
            values: Map::new(),
            names: Map::new(),
        }
    }
}

impl<'a> Frame<'a> {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }
}

impl<'a> Default for Engine<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Self {
            next: 1,
            globals: Scope::new(),
            frames: Vec::new(),
            funcs: Map::new(),
            func_names: Map::new(),
            names: Map::new(),
        }
    }

    pub fn reset(&mut self) {
        self.next = 1;
        self.globals = Scope::new();
        self.frames.clear();
        self.funcs.clear();
        self.func_names.clear();
        self.names.clear();
    }

    pub fn process(
        &mut self,
        session: &Session<'a>,
        keys: &[Identity],
    ) -> Result<(), GenerateError<'a>> {
        self.reset();
        self.collect(session);

        let mut keys = session.source_keys(keys);
        keys.sort();

        for key in keys {
            self.run_unit(session, key)?;
        }

        Ok(())
    }

    pub fn execute_line(
        &mut self,
        session: &Session<'a>,
        key: Identity,
    ) -> Result<Option<Value>, GenerateError<'a>> {
        self.reset();
        self.collect(session);

        let mut keys = session
            .records
            .iter()
            .filter_map(|(&id, record)| {
                if id != key && record.kind == RecordKind::Source && record.fetch(0).is_some() {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        keys.sort();

        for id in keys {
            self.run_unit(session, id)?;
        }

        let Some(record) = session.records.get(&key) else {
            return Ok(None);
        };
        let Some(Artifact::Analyses(analyses)) = record.fetch(3) else {
            return Ok(None);
        };

        self.run_analyses(analyses)
    }

    fn run_unit(&mut self, session: &Session<'a>, key: Identity) -> Result<(), GenerateError<'a>> {
        let Some(record) = session.records.get(&key) else {
            return Ok(());
        };
        let Some(Artifact::Analyses(analyses)) = record.fetch(3) else {
            return Ok(());
        };
        self.run_top(analyses)?;
        Ok(())
    }

    fn run_top(&mut self, analyses: &[Analysis<'a>]) -> Result<(), GenerateError<'a>> {
        for analysis in analyses {
            match self.eval(analysis)? {
                Flow::Value(_) => {}
                Flow::Return(_) => {
                    return Err(self.error(
                        ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                        analysis.span,
                    ));
                }
                Flow::Break(_) => {
                    return Err(self.error(
                        ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                        analysis.span,
                    ));
                }
                Flow::Continue => {
                    return Err(self.error(
                        ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                        analysis.span,
                    ));
                }
            }
        }

        Ok(())
    }

    fn run_analyses(
        &mut self,
        analyses: &[Analysis<'a>],
    ) -> Result<Option<Value>, GenerateError<'a>> {
        let mut last = Value::Empty;

        for analysis in analyses {
            match self.eval(analysis)? {
                Flow::Value(value) => last = value,
                Flow::Return(value) => last = value,
                Flow::Break(_) => {
                    return Err(self.error(
                        ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                        analysis.span,
                    ));
                }
                Flow::Continue => {
                    return Err(self.error(
                        ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                        analysis.span,
                    ));
                }
            }
        }

        Ok(Some(last))
    }

    fn collect(&mut self, session: &Session<'a>) {
        for symbol in session.resolver.registry.values() {
            if let Some(name) = symbol.target() {
                self.names.insert(name, symbol.identity);
                if matches!(symbol.kind, SymbolKind::Function(_)) {
                    self.func_names.insert(name, symbol.identity);
                }
            }
        }
    }

    fn error(&self, kind: ErrorKind<'a>, span: Span) -> GenerateError<'a> {
        GenerateError::new(kind, span)
    }

    fn name_text(&self, name: Str<'a>) -> String {
        name.as_str().unwrap_or_default().to_string()
    }

    fn target_text(&self, target: &Target<'a>) -> String {
        self.name_text(target.name)
    }

    fn bind(&mut self, target: &Target<'a>, value: Value) {
        let id = self.target_id(target);

        if let Some(frame) = self.frames.last_mut() {
            let scope = frame.scopes.last_mut().unwrap();
            scope.values.insert(id, value);
            if !target.name.is_empty() {
                scope.names.insert(target.name, id);
            }
        } else {
            self.globals.values.insert(id, value);
            if !target.name.is_empty() {
                self.globals.names.insert(target.name, id);
            }
        }
    }

    fn target_id(&mut self, target: &Target<'a>) -> Identity {
        if target.id != 0 {
            return target.id;
        }

        if !target.name.is_empty() {
            if let Some(id) = self.names.get(&target.name).copied() {
                return id;
            }
            if let Some(id) = self.globals.names.get(&target.name).copied() {
                return id;
            }
            for frame in self.frames.iter().rev() {
                for scope in frame.scopes.iter().rev() {
                    if let Some(id) = scope.names.get(&target.name).copied() {
                        return id;
                    }
                }
            }
        }

        let id = self.next;
        self.next += 1;
        if !target.name.is_empty() {
            self.names.insert(target.name, id);
        }
        id
    }

    fn target_key(&self, scope: &Scope<'a>, target: &Target<'a>) -> Option<Identity> {
        if target.id != 0 && scope.values.contains_key(&target.id) {
            return Some(target.id);
        }

        if !target.name.is_empty() {
            return scope.names.get(&target.name).copied();
        }

        None
    }

    fn read_target(&self, target: &Target<'a>, span: Span) -> Result<Value, GenerateError<'a>> {
        for frame in self.frames.iter().rev() {
            for scope in frame.scopes.iter().rev() {
                if let Some(id) = self.target_key(scope, target) {
                    return Ok(scope.values.get(&id).cloned().unwrap_or(Value::Empty));
                }
            }
        }

        if let Some(id) = self.target_key(&self.globals, target) {
            return Ok(self
                .globals
                .values
                .get(&id)
                .cloned()
                .unwrap_or(Value::Empty));
        }

        Err(self.error(
            ErrorKind::Variable(VariableError::Undefined {
                name: self.target_text(target),
            }),
            span,
        ))
    }

    fn write_target(
        &mut self,
        target: &Target<'a>,
        value: Value,
        span: Span,
    ) -> Result<Value, GenerateError<'a>> {
        for frame in self.frames.iter_mut().rev() {
            for scope in frame.scopes.iter_mut().rev() {
                let key = if target.id != 0 && scope.values.contains_key(&target.id) {
                    Some(target.id)
                } else if !target.name.is_empty() {
                    scope.names.get(&target.name).copied()
                } else {
                    None
                };

                if let Some(id) = key {
                    scope.values.insert(id, value.clone());
                    return Ok(value);
                }
            }
        }

        let key = if target.id != 0 && self.globals.values.contains_key(&target.id) {
            Some(target.id)
        } else if !target.name.is_empty() {
            self.globals.names.get(&target.name).copied()
        } else {
            None
        };

        if let Some(id) = key {
            self.globals.values.insert(id, value.clone());
            return Ok(value);
        }

        Err(self.error(
            ErrorKind::Variable(VariableError::Undefined {
                name: self.target_text(target),
            }),
            span,
        ))
    }

    fn eval(&mut self, analysis: &Analysis<'a>) -> Result<Flow, GenerateError<'a>> {
        match &analysis.kind {
            AnalysisKind::Integer { value, .. } => Ok(Flow::Value(Value::Integer(*value as i64))),
            AnalysisKind::Float { value, .. } => Ok(Flow::Value(Value::Float(value.0))),
            AnalysisKind::Boolean { value } => Ok(Flow::Value(Value::Boolean(*value))),
            AnalysisKind::Character { value } => Ok(Flow::Value(Value::Character(*value))),
            AnalysisKind::String { value } => Ok(Flow::Value(Value::String(
                value.as_str().unwrap_or_default().to_string(),
            ))),
            AnalysisKind::Array(values) | AnalysisKind::Tuple(values) => {
                let mut items = Vec::with_capacity(values.len());
                for value in values {
                    items.push(self.value(value)?);
                }
                Ok(Flow::Value(Value::Sequence(items)))
            }
            AnalysisKind::Negate(value) => self.negate(value, analysis.span),
            AnalysisKind::SizeOf(typing) => {
                Ok(Flow::Value(Value::Integer(self.size_of(typing) as i64)))
            }
            AnalysisKind::Add(left, right) => self.add(left, right, analysis.span),
            AnalysisKind::Subtract(left, right) => self.subtract(left, right, analysis.span),
            AnalysisKind::Multiply(left, right) => self.multiply(left, right, analysis.span),
            AnalysisKind::Divide(left, right) => self.divide(left, right, analysis.span),
            AnalysisKind::Modulus(left, right) => self.modulus(left, right, analysis.span),
            AnalysisKind::LogicalAnd(left, right) => self.logical_and(left, right),
            AnalysisKind::LogicalOr(left, right) => self.logical_or(left, right),
            AnalysisKind::LogicalNot(value) => self.logical_not(value, analysis.span),
            AnalysisKind::LogicalXOr(left, right) => self.logical_xor(left, right, analysis.span),
            AnalysisKind::BitwiseAnd(left, right) => self.bitwise_and(left, right, analysis.span),
            AnalysisKind::BitwiseOr(left, right) => self.bitwise_or(left, right, analysis.span),
            AnalysisKind::BitwiseNot(value) => self.bitwise_not(value, analysis.span),
            AnalysisKind::BitwiseXOr(left, right) => self.bitwise_xor(left, right, analysis.span),
            AnalysisKind::ShiftLeft(left, right) => self.shift_left(left, right, analysis.span),
            AnalysisKind::ShiftRight(left, right) => self.shift_right(left, right, analysis.span),
            AnalysisKind::AddressOf(_) | AnalysisKind::Dereference(_) => Err(self.error(
                ErrorKind::Variable(VariableError::DereferenceNonPointer),
                analysis.span,
            )),
            AnalysisKind::Equal(left, right) => self.equal(left, right),
            AnalysisKind::NotEqual(left, right) => self.not_equal(left, right),
            AnalysisKind::Less(left, right) => self.less(left, right, analysis.span),
            AnalysisKind::LessOrEqual(left, right) => {
                self.less_or_equal(left, right, analysis.span)
            }
            AnalysisKind::Greater(left, right) => self.greater(left, right, analysis.span),
            AnalysisKind::GreaterOrEqual(left, right) => {
                self.greater_or_equal(left, right, analysis.span)
            }
            AnalysisKind::Index(index) => self.index(index, analysis.span),
            AnalysisKind::Usage(name) => self.usage(*name, analysis.span),
            AnalysisKind::Symbol(target) => {
                Ok(Flow::Value(self.read_target(target, analysis.span)?))
            }
            AnalysisKind::Access(target, member) => self.access(target, member, analysis.span),
            AnalysisKind::Slot(target, slot) => self.slot(target, *slot, analysis.span),
            AnalysisKind::Constructor(value) => {
                self.constructor(&analysis.typing, value, analysis.span)
            }
            AnalysisKind::Pack(target, values) => {
                self.pack(&analysis.typing, target, values, analysis.span)
            }
            AnalysisKind::Assign(name, value) => self.assign(*name, value, analysis.span),
            AnalysisKind::Write(target, value) => self.write(target, value, analysis.span),
            AnalysisKind::Store(target, value) => self.store(target, value, analysis.span),
            AnalysisKind::Binding(binding) => self.binding(binding, analysis.span),
            AnalysisKind::Structure(_) | AnalysisKind::Union(_) | AnalysisKind::Composite(_) => {
                Ok(Flow::Value(Value::Empty))
            }
            AnalysisKind::Function(function) => self.function(function, analysis),
            AnalysisKind::Block(values) => self.block(values),
            AnalysisKind::Conditional(condition, truth, fall) => {
                self.conditional(condition, truth, fall.as_deref())
            }
            AnalysisKind::While(condition, body) => self.r#while(condition, body),
            AnalysisKind::Module(_, values) => self.block(values),
            AnalysisKind::Invoke(invoke) => self.invoke(invoke, analysis.span),
            AnalysisKind::Call(target, values) => self.call(target, values, analysis.span),
            AnalysisKind::Return(value) => Ok(Flow::Return(self.option(value.as_deref())?)),
            AnalysisKind::Break(value) => Ok(Flow::Break(self.option(value.as_deref())?)),
            AnalysisKind::Continue(_) => Ok(Flow::Continue),
        }
    }

    fn value(&mut self, analysis: &Analysis<'a>) -> Result<Value, GenerateError<'a>> {
        match self.eval(analysis)? {
            Flow::Value(value) | Flow::Return(value) | Flow::Break(value) => Ok(value),
            Flow::Continue => Ok(Value::Empty),
        }
    }

    fn option(&mut self, value: Option<&Analysis<'a>>) -> Result<Value, GenerateError<'a>> {
        match value {
            Some(value) => self.value(value),
            None => Ok(Value::Empty),
        }
    }

    fn function(
        &mut self,
        function: &Function<Str<'a>, Analysis<'a>, Option<Box<Analysis<'a>>>, Option<Type<'a>>>,
        analysis: &Analysis<'a>,
    ) -> Result<Flow, GenerateError<'a>> {
        let id = if analysis.typing.identity != 0 {
            analysis.typing.identity
        } else if let Some(id) = self.func_names.get(&function.target).copied() {
            id
        } else {
            self.target_id(&Target::new(0, function.target))
        };

        self.funcs.insert(
            id,
            Routine {
                func: function.clone(),
            },
        );
        self.func_names.insert(function.target, id);
        Ok(Flow::Value(Value::Empty))
    }

    fn usage(&mut self, name: Str<'a>, span: Span) -> Result<Flow, GenerateError<'a>> {
        let target = Target::new(self.names.get(&name).copied().unwrap_or_default(), name);
        Ok(Flow::Value(self.read_target(&target, span)?))
    }

    fn assign(
        &mut self,
        name: Str<'a>,
        value: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let target = Target::new(self.names.get(&name).copied().unwrap_or_default(), name);
        let value = self.value(value)?;
        Ok(Flow::Value(self.write_target(&target, value, span)?))
    }

    fn write(
        &mut self,
        target: &Target<'a>,
        value: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let value = self.value(value)?;
        Ok(Flow::Value(self.write_target(target, value, span)?))
    }

    fn binding(
        &mut self,
        binding: &crate::data::Binding<Box<Analysis<'a>>, Box<Analysis<'a>>, Type<'a>>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let AnalysisKind::Symbol(target) = &binding.target.kind else {
            return Err(self.error(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                span,
            ));
        };

        let value = match binding.kind {
            BindingKind::Static | BindingKind::Let => {
                if let Some(value) = binding.value.as_deref() {
                    self.value(value)?
                } else {
                    return Err(self.error(
                        ErrorKind::Variable(VariableError::BindingWithoutInitializer {
                            name: self.target_text(target),
                        }),
                        span,
                    ));
                }
            }
        };

        self.bind(target, value.clone());
        Ok(Flow::Value(value))
    }

    fn block(&mut self, values: &[Analysis<'a>]) -> Result<Flow, GenerateError<'a>> {
        if let Some(frame) = self.frames.last_mut() {
            frame.scopes.push(Scope::new());
        }

        let mut last = Value::Empty;

        for value in values {
            match self.eval(value)? {
                Flow::Value(value) => last = value,
                flow => {
                    if let Some(frame) = self.frames.last_mut() {
                        frame.scopes.pop();
                    }
                    return Ok(flow);
                }
            }
        }

        if let Some(frame) = self.frames.last_mut() {
            frame.scopes.pop();
        }

        Ok(Flow::Value(last))
    }

    fn conditional(
        &mut self,
        condition: &Analysis<'a>,
        truth: &Analysis<'a>,
        fall: Option<&Analysis<'a>>,
    ) -> Result<Flow, GenerateError<'a>> {
        if self.value(condition)?.truth() {
            self.eval(truth)
        } else if let Some(fall) = fall {
            self.eval(fall)
        } else {
            Ok(Flow::Value(Value::Empty))
        }
    }

    fn r#while(
        &mut self,
        condition: &Analysis<'a>,
        body: &Analysis<'a>,
    ) -> Result<Flow, GenerateError<'a>> {
        let mut last = Value::Empty;

        loop {
            if !self.value(condition)?.truth() {
                return Ok(Flow::Value(last));
            }

            match self.eval(body)? {
                Flow::Value(value) => last = value,
                Flow::Break(value) => return Ok(Flow::Value(value)),
                Flow::Continue => continue,
                Flow::Return(value) => return Ok(Flow::Return(value)),
            }
        }
    }

    fn call(
        &mut self,
        target: &Target<'a>,
        values: &[Analysis<'a>],
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let id = if target.id != 0 {
            target.id
        } else {
            self.func_names
                .get(&target.name)
                .copied()
                .unwrap_or_default()
        };

        let Some(routine) = self.funcs.get(&id).cloned() else {
            return Err(self.error(
                ErrorKind::Function(FunctionError::Undefined {
                    name: self.target_text(target),
                }),
                span,
            ));
        };

        let mut args = Vec::with_capacity(values.len());
        for value in values {
            args.push(self.value(value)?);
        }

        self.invoke_routine(routine, args, span)
    }

    fn invoke(
        &mut self,
        invoke: &crate::data::Invoke<Box<Analysis<'a>>, Analysis<'a>>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        match &invoke.target.kind {
            AnalysisKind::Symbol(target) => self.call(target, &invoke.members, span),
            AnalysisKind::Usage(name) => {
                let target = Target::new(
                    self.func_names.get(name).copied().unwrap_or_default(),
                    *name,
                );
                self.call(&target, &invoke.members, span)
            }
            _ => Err(self.error(
                ErrorKind::Function(FunctionError::Undefined {
                    name: String::new(),
                }),
                span,
            )),
        }
    }

    fn invoke_routine(
        &mut self,
        routine: Routine<'a>,
        args: Vec<Value>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        if !routine.func.variadic && args.len() != routine.func.members.len() {
            return Err(self.error(
                ErrorKind::Verification(format!(
                    "invalid argument count for {}",
                    self.name_text(routine.func.target)
                )),
                span,
            ));
        }

        self.frames.push(Frame::new());

        for (index, member) in routine.func.members.iter().enumerate() {
            let AnalysisKind::Binding(binding) = &member.kind else {
                continue;
            };
            let AnalysisKind::Symbol(target) = &binding.target.kind else {
                continue;
            };
            let value = args.get(index).cloned().unwrap_or(Value::Empty);
            self.bind(target, value);
        }

        let flow = if let Some(body) = routine.func.body.as_deref() {
            self.eval(body)?
        } else {
            Flow::Value(Value::Empty)
        };

        self.frames.pop();

        Ok(match flow {
            Flow::Return(value) => Flow::Value(value),
            Flow::Break(_) => {
                return Err(self.error(
                    ErrorKind::ControlFlow(ControlFlowError::BreakOutsideLoop),
                    span,
                ))
            }
            Flow::Continue => {
                return Err(self.error(
                    ErrorKind::ControlFlow(ControlFlowError::ContinueOutsideLoop),
                    span,
                ))
            }
            flow => flow,
        })
    }

    fn slot(
        &mut self,
        target: &Analysis<'a>,
        slot: usize,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let value = self.value(target)?;
        Ok(Flow::Value(self.slot_value(value, slot, span)?))
    }

    fn access(
        &mut self,
        target: &Analysis<'a>,
        member: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        if let AnalysisKind::Symbol(item) = &member.kind {
            if let Some(slot) = self.slot_of(&target.typing, item) {
                return self.slot(target, slot, span);
            }
        }

        Err(self.error(
            ErrorKind::DataStructure(DataStructureError::InvalidMemberAccessExpression),
            span,
        ))
    }

    fn slot_of(&self, typing: &Type<'a>, target: &Target<'a>) -> Option<usize> {
        match &typing.kind {
            TypeKind::Pointer { target: typing } => self.slot_of(typing, target),
            TypeKind::Structure(value) | TypeKind::Union(value) => {
                value.members.iter().position(|item| {
                    item.identity == target.id || self.name_of(item) == Some(target.name)
                })
            }
            _ => None,
        }
    }

    fn name_of(&self, typing: &Type<'a>) -> Option<Str<'a>> {
        match &typing.kind {
            TypeKind::Binding(value) => Some(value.target),
            TypeKind::Function(value) if !value.target.is_empty() => Some(value.target),
            TypeKind::Has(value) => self.name_of(value),
            _ => None,
        }
    }

    fn slot_value(
        &self,
        value: Value,
        slot: usize,
        span: Span,
    ) -> Result<Value, GenerateError<'a>> {
        match value {
            Value::Composite(values) | Value::Sequence(values) => {
                values.get(slot).cloned().ok_or_else(|| {
                    self.error(
                        ErrorKind::DataStructure(DataStructureError::UnknownField {
                            target: String::new(),
                            member: slot.to_string(),
                        }),
                        span,
                    )
                })
            }
            _ => Err(self.error(
                ErrorKind::DataStructure(DataStructureError::NotIndexable),
                span,
            )),
        }
    }

    fn store(
        &mut self,
        target: &Analysis<'a>,
        value: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let value = self.value(value)?;
        self.assign_target(target, value.clone(), span)?;
        Ok(Flow::Value(value))
    }

    fn assign_target(
        &mut self,
        target: &Analysis<'a>,
        value: Value,
        span: Span,
    ) -> Result<(), GenerateError<'a>> {
        match &target.kind {
            AnalysisKind::Symbol(target) => {
                self.write_target(target, value, span)?;
                Ok(())
            }
            AnalysisKind::Slot(target, slot) => self.assign_slot(target, *slot, value, span),
            AnalysisKind::Index(index) => {
                let member = index.members.first().ok_or_else(|| {
                    self.error(
                        ErrorKind::DataStructure(DataStructureError::IndexMissingArgument),
                        span,
                    )
                })?;
                let slot = self.index_value(member, span)?;
                self.assign_index(&index.target, slot, value, span)
            }
            _ => Err(self.error(
                ErrorKind::Variable(VariableError::InvalidAssignmentTarget),
                span,
            )),
        }
    }

    fn assign_slot(
        &mut self,
        target: &Analysis<'a>,
        slot: usize,
        value: Value,
        span: Span,
    ) -> Result<(), GenerateError<'a>> {
        match &target.kind {
            AnalysisKind::Symbol(target) => {
                let mut base = self.read_target(target, span)?;
                match &mut base {
                    Value::Composite(items) | Value::Sequence(items) => {
                        if slot >= items.len() {
                            items.resize(slot + 1, Value::Empty);
                        }
                        items[slot] = value;
                        self.write_target(target, base, span)?;
                        Ok(())
                    }
                    _ => Err(self.error(
                        ErrorKind::DataStructure(DataStructureError::NotIndexable),
                        span,
                    )),
                }
            }
            AnalysisKind::Slot(base, base_slot) => {
                let mut root = self.value(target)?;
                match &mut root {
                    Value::Composite(items) | Value::Sequence(items) => {
                        if *base_slot >= items.len() {
                            return Err(self.error(
                                ErrorKind::DataStructure(DataStructureError::NotIndexable),
                                span,
                            ));
                        }
                        match &mut items[*base_slot] {
                            Value::Composite(inner) | Value::Sequence(inner) => {
                                if slot >= inner.len() {
                                    inner.resize(slot + 1, Value::Empty);
                                }
                                inner[slot] = value;
                            }
                            _ => {
                                return Err(self.error(
                                    ErrorKind::DataStructure(DataStructureError::NotIndexable),
                                    span,
                                ))
                            }
                        }
                        self.assign_target(base, root, span)
                    }
                    _ => Err(self.error(
                        ErrorKind::DataStructure(DataStructureError::NotIndexable),
                        span,
                    )),
                }
            }
            _ => {
                let mut root = self.value(target)?;
                match &mut root {
                    Value::Composite(items) | Value::Sequence(items) => {
                        if slot >= items.len() {
                            items.resize(slot + 1, Value::Empty);
                        }
                        items[slot] = value;
                        self.assign_target(target, root, span)
                    }
                    _ => Err(self.error(
                        ErrorKind::DataStructure(DataStructureError::NotIndexable),
                        span,
                    )),
                }
            }
        }
    }

    fn assign_index(
        &mut self,
        target: &Analysis<'a>,
        index: usize,
        value: Value,
        span: Span,
    ) -> Result<(), GenerateError<'a>> {
        let mut root = self.value(target)?;
        match &mut root {
            Value::Composite(items) | Value::Sequence(items) => {
                if index >= items.len() {
                    items.resize(index + 1, Value::Empty);
                }
                items[index] = value;
                self.assign_target(target, root, span)
            }
            _ => Err(self.error(
                ErrorKind::DataStructure(DataStructureError::NotIndexable),
                span,
            )),
        }
    }

    fn constructor(
        &mut self,
        typing: &Type<'a>,
        value: &crate::data::Aggregate<Str<'a>, Analysis<'a>>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let target = Target::new(
            self.names.get(&value.target).copied().unwrap_or_default(),
            value.target,
        );
        let values = value
            .members
            .iter()
            .enumerate()
            .map(|(slot, value)| (slot, value.clone()))
            .collect::<Vec<_>>();
        self.pack(typing, &target, &values, span)
    }

    fn pack(
        &mut self,
        typing: &Type<'a>,
        _target: &Target<'a>,
        values: &[(usize, Analysis<'a>)],
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let size = match &typing.kind {
            TypeKind::Structure(value) | TypeKind::Union(value) => value.members.len(),
            _ => values.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0),
        };

        let mut items = vec![Value::Empty; size];
        for (slot, value) in values {
            if *slot >= items.len() {
                return Err(self.error(
                    ErrorKind::DataStructure(DataStructureError::TooManyInitializers {
                        target: String::new(),
                    }),
                    span,
                ));
            }
            items[*slot] = self.value(value)?;
        }

        Ok(Flow::Value(Value::Composite(items)))
    }

    fn index(
        &mut self,
        index: &crate::data::Index<Box<Analysis<'a>>, Analysis<'a>>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let value = self.value(&index.target)?;
        let slot = index.members.first().ok_or_else(|| {
            self.error(
                ErrorKind::DataStructure(DataStructureError::IndexMissingArgument),
                span,
            )
        })?;
        let slot = self.index_value(slot, span)?;
        Ok(Flow::Value(self.slot_value(value, slot, span)?))
    }

    fn index_value(
        &mut self,
        value: &Analysis<'a>,
        span: Span,
    ) -> Result<usize, GenerateError<'a>> {
        match self.value(value)? {
            Value::Integer(value) if value >= 0 => Ok(value as usize),
            _ => Err(self.error(
                ErrorKind::DataStructure(DataStructureError::ArrayIndexNotConstant),
                span,
            )),
        }
    }

    fn negate(&mut self, value: &Analysis<'a>, span: Span) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(match self.value(value)? {
            Value::Integer(value) => Value::Integer(-value),
            Value::Float(value) => Value::Float(-value),
            _ => return Err(self.error(ErrorKind::Negate, span)),
        }))
    }

    fn add(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.numeric(
            left,
            right,
            span,
            |left, right| left + right,
            |left, right| left + right,
        )
    }

    fn subtract(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.numeric(
            left,
            right,
            span,
            |left, right| left - right,
            |left, right| left - right,
        )
    }

    fn multiply(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.numeric(
            left,
            right,
            span,
            |left, right| left * right,
            |left, right| left * right,
        )
    }

    fn divide(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let right_value = self.value(right)?;
        match right_value {
            Value::Integer(0) | Value::Float(0.0) => {
                return Err(self.error(
                    ErrorKind::Verification("division by zero".to_string()),
                    span,
                ))
            }
            _ => {}
        }
        let left_value = self.value(left)?;
        self.numeric_pair(
            left_value,
            right_value,
            span,
            |left, right| left / right,
            |left, right| left / right,
        )
    }

    fn modulus(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        let left = self.value(left)?;
        let right = self.value(right)?;
        Ok(Flow::Value(match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                if right == 0 {
                    return Err(self.error(
                        ErrorKind::Verification("division by zero".to_string()),
                        span,
                    ));
                }
                Value::Integer(left % right)
            }
            (Value::Float(left), Value::Float(right)) => Value::Float(left % right),
            (Value::Float(left), Value::Integer(right)) => Value::Float(left % right as f64),
            (Value::Integer(left), Value::Float(right)) => Value::Float(left as f64 % right),
            _ => return Err(self.error(ErrorKind::Normalize, span)),
        }))
    }

    fn numeric(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
        ints: fn(i64, i64) -> i64,
        floats: fn(f64, f64) -> f64,
    ) -> Result<Flow, GenerateError<'a>> {
        let left = self.value(left)?;
        let right = self.value(right)?;
        self.numeric_pair(left, right, span, ints, floats)
    }

    fn numeric_pair(
        &self,
        left: Value,
        right: Value,
        span: Span,
        ints: fn(i64, i64) -> i64,
        floats: fn(f64, f64) -> f64,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(ints(left, right)),
            (Value::Float(left), Value::Float(right)) => Value::Float(floats(left, right)),
            (Value::Float(left), Value::Integer(right)) => Value::Float(floats(left, right as f64)),
            (Value::Integer(left), Value::Float(right)) => Value::Float(floats(left as f64, right)),
            _ => return Err(self.error(ErrorKind::Normalize, span)),
        }))
    }

    fn logical_and(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
    ) -> Result<Flow, GenerateError<'a>> {
        let left = self.value(left)?;
        if !left.truth() {
            return Ok(Flow::Value(Value::Boolean(false)));
        }
        Ok(Flow::Value(Value::Boolean(self.value(right)?.truth())))
    }

    fn logical_or(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
    ) -> Result<Flow, GenerateError<'a>> {
        let left = self.value(left)?;
        if left.truth() {
            return Ok(Flow::Value(Value::Boolean(true)));
        }
        Ok(Flow::Value(Value::Boolean(self.value(right)?.truth())))
    }

    fn logical_not(
        &mut self,
        value: &Analysis<'a>,
        _span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(Value::Boolean(!self.value(value)?.truth())))
    }

    fn logical_xor(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        _span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(Value::Boolean(
            self.value(left)?.truth() ^ self.value(right)?.truth(),
        )))
    }

    fn bitwise_and(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.integer_pair(left, right, span, |left, right| left & right)
    }

    fn bitwise_or(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.integer_pair(left, right, span, |left, right| left | right)
    }

    fn bitwise_not(&mut self, value: &Analysis<'a>, span: Span) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(match self.value(value)? {
            Value::Integer(value) => Value::Integer(!value),
            _ => return Err(self.error(ErrorKind::Normalize, span)),
        }))
    }

    fn bitwise_xor(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.integer_pair(left, right, span, |left, right| left ^ right)
    }

    fn shift_left(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.integer_pair(left, right, span, |left, right| left << right)
    }

    fn shift_right(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.integer_pair(left, right, span, |left, right| left >> right)
    }

    fn integer_pair(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
        apply: fn(i64, i64) -> i64,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(match (self.value(left)?, self.value(right)?) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(apply(left, right)),
            _ => return Err(self.error(ErrorKind::Normalize, span)),
        }))
    }

    fn equal(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(Value::Boolean(
            self.value(left)? == self.value(right)?,
        )))
    }

    fn not_equal(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(Value::Boolean(
            self.value(left)? != self.value(right)?,
        )))
    }

    fn less(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.compare(
            left,
            right,
            span,
            |left, right| left < right,
            |left, right| left < right,
        )
    }

    fn less_or_equal(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.compare(
            left,
            right,
            span,
            |left, right| left <= right,
            |left, right| left <= right,
        )
    }

    fn greater(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.compare(
            left,
            right,
            span,
            |left, right| left > right,
            |left, right| left > right,
        )
    }

    fn greater_or_equal(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
    ) -> Result<Flow, GenerateError<'a>> {
        self.compare(
            left,
            right,
            span,
            |left, right| left >= right,
            |left, right| left >= right,
        )
    }

    fn compare(
        &mut self,
        left: &Analysis<'a>,
        right: &Analysis<'a>,
        span: Span,
        ints: fn(i64, i64) -> bool,
        floats: fn(f64, f64) -> bool,
    ) -> Result<Flow, GenerateError<'a>> {
        Ok(Flow::Value(Value::Boolean(
            match (self.value(left)?, self.value(right)?) {
                (Value::Integer(left), Value::Integer(right)) => ints(left, right),
                (Value::Float(left), Value::Float(right)) => floats(left, right),
                (Value::Float(left), Value::Integer(right)) => floats(left, right as f64),
                (Value::Integer(left), Value::Float(right)) => floats(left as f64, right),
                _ => return Err(self.error(ErrorKind::Normalize, span)),
            },
        )))
    }

    fn size_of(&self, typing: &Type<'a>) -> usize {
        match &typing.kind {
            TypeKind::Integer { size, .. } => size / 8,
            TypeKind::Float { size } => size / 8,
            TypeKind::Boolean => 1,
            TypeKind::Character => 4,
            TypeKind::String => 16,
            TypeKind::Pointer { .. } => 8,
            TypeKind::Array { member, size } => self.size_of(member) * size,
            TypeKind::Tuple { members } => members.iter().map(|item| self.size_of(item)).sum(),
            TypeKind::Structure(value) => value.members.iter().map(|item| self.size_of(item)).sum(),
            TypeKind::Union(value) => value
                .members
                .iter()
                .map(|item| self.size_of(item))
                .max()
                .unwrap_or(0),
            TypeKind::Binding(value) => value
                .value
                .as_deref()
                .map(|value| self.size_of(value))
                .or_else(|| value.annotation.as_deref().map(|value| self.size_of(value)))
                .unwrap_or(0),
            TypeKind::Has(value) => self.size_of(value),
            _ => 0,
        }
    }
}
