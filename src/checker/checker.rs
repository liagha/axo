use crate::{
    checker::{Type, TypeKind, ErrorKind, CheckError},
    data::{
        Identity,
        Structure,
        memory::take,
    },
    internal::{
        hash::Map,
    },
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::Resolver,
    tracker::Span,
};

pub struct Checker<'check, 'source> {
    pub input: &'check mut Vec<Element<'source>>,
    pub resolver: &'check Resolver<'source>,
    pub environment: Map<Identity, Type<'source>>,
    pub errors: Vec<CheckError<'source>>,
    pub variables: Vec<Option<Type<'source>>>,
}

pub trait Checkable<'source> {
    fn check(&mut self, checker: &mut Checker<'_, 'source>);
}

impl<'check, 'source> Checker<'check, 'source> {
    pub fn new(input: &'check mut Vec<Element<'source>>, resolver: &'check Resolver<'source>) -> Self {
        Self {
            input,
            resolver,
            environment: Map::new(),
            errors: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn check(&mut self) {
        let mut elements = take(self.input);

        for element in &mut elements {
            element.check(self);
        }

        for element in &mut elements {
            self.concretize_element(element);
        }

        let environment = take(&mut self.environment);
        let mut resolved = Map::with_capacity(environment.len());

        for (identity, typ) in environment {
            resolved.insert(identity, self.concretize(&typ));
        }

        self.environment = resolved;
        *self.input = elements;
    }

    pub fn concretize_element(&mut self, element: &mut Element<'source>) {
        element.typ = self.concretize(&element.typ);

        match &mut element.kind {
            ElementKind::Literal(_) => {}
            ElementKind::Delimited(delimited) => {
                for member in &mut delimited.members {
                    self.concretize_element(member);
                }
            }
            ElementKind::Unary(unary) => {
                self.concretize_element(&mut unary.operand);
            }
            ElementKind::Binary(binary) => {
                self.concretize_element(&mut binary.left);
                self.concretize_element(&mut binary.right);
            }
            ElementKind::Index(index) => {
                self.concretize_element(&mut index.target);
                for member in &mut index.members {
                    self.concretize_element(member);
                }
            }
            ElementKind::Invoke(invoke) => {
                self.concretize_element(&mut invoke.target);
                for member in &mut invoke.members {
                    self.concretize_element(member);
                }
            }
            ElementKind::Construct(construct) => {
                for member in &mut construct.members {
                    self.concretize_element(member);
                }
            }
            ElementKind::Symbolize(symbol) => {
                self.concretize_symbol(symbol);
            }
        }
    }

    pub fn concretize_symbol(&mut self, symbol: &mut Symbol<'source>) {
        symbol.typ = self.concretize(&symbol.typ);

        match &mut symbol.kind {
            SymbolKind::Binding(binding) => {
                if let Some(value) = &mut binding.value {
                    self.concretize_element(value);
                }
            }
            SymbolKind::Structure(structure) | SymbolKind::Union(structure) => {
                for member in &mut structure.members {
                    self.concretize_symbol(member);
                }
            }
            SymbolKind::Function(function) => {
                for member in &mut function.members {
                    self.concretize_symbol(member);
                }
                if let Some(body) = &mut function.body {
                    self.concretize_element(body);
                }
            }
            SymbolKind::Module(_) => {}
        }
    }

    pub fn lookup(&mut self, identity: Identity, span: Span<'source>) -> Type<'source> {
        if let Some(typ) = self.environment.get(&identity) {
            return typ.clone();
        }

        if let Some(symbol) = self.resolver.scope.get_identity(identity) {
            let mut cloned = symbol.clone();

            let variable = self.fresh(span);
            self.environment.insert(identity, variable.clone());

            let scope = self.environment.clone();
            cloned.check(self);
            self.environment = scope;

            let unified = self.unify(span, &variable, &cloned.typ);
            self.environment.insert(identity, unified.clone());

            return unified;
        }

        self.fresh(span)
    }

    pub fn fresh(&mut self, span: Span<'source>) -> Type<'source> {
        let identity = self.variables.len();
        self.variables.push(None);
        Type::new(TypeKind::Variable(identity), span)
    }

    pub fn concretize(&mut self, typ: &Type<'source>) -> Type<'source> {
        match &typ.kind {
            TypeKind::Variable(identity) => {
                if let Some(resolved) = self.variables[*identity].clone() {
                    let deep = self.concretize(&resolved);
                    self.variables[*identity] = Some(deep.clone());
                    deep
                } else {
                    typ.clone()
                }
            }
            TypeKind::Pointer { target } => {
                Type::new(TypeKind::Pointer { target: Box::new(self.concretize(target)) }, typ.span)
            }
            TypeKind::Array { member, size } => {
                Type::new(TypeKind::Array { member: Box::new(self.concretize(member)), size: *size }, typ.span)
            }
            TypeKind::Tuple { members } => {
                let items = members.iter().map(|item| self.concretize(item)).collect();
                Type::new(TypeKind::Tuple { members: items }, typ.span)
            }
            TypeKind::Function(name, parameters, output) => {
                let arguments = parameters.iter().map(|parameter| self.concretize(parameter)).collect();
                let returnable = output.as_ref().map(|kind| Box::new(self.concretize(kind)));
                Type::new(TypeKind::Function(name.clone(), arguments, returnable), typ.span)
            }
            TypeKind::Structure(structure) => {
                let members = structure.members.iter().map(|member| self.concretize(member)).collect();
                Type::new(TypeKind::Structure(Structure::new(structure.target.clone(), members)), typ.span)
            }
            TypeKind::Union(structure) => {
                let members = structure.members.iter().map(|member| self.concretize(member)).collect();
                Type::new(TypeKind::Union(Structure::new(structure.target.clone(), members)), typ.span)
            }
            TypeKind::Constructor(structure) => {
                let members = structure.members.iter().map(|member| self.concretize(member)).collect();
                Type::new(TypeKind::Constructor(Structure::new(structure.target.clone(), members)), typ.span)
            }
            _ => typ.clone(),
        }
    }

    fn occurs(&mut self, identity: Identity, typ: &Type<'source>) -> bool {
        let concretized = self.concretize(typ);

        match concretized.kind {
            TypeKind::Variable(variable) => identity == variable,
            TypeKind::Pointer { ref target } => self.occurs(identity, target),
            TypeKind::Array { ref member, .. } => self.occurs(identity, member),
            TypeKind::Tuple { ref members } => members.iter().any(|member| self.occurs(identity, member)),
            TypeKind::Function(_, ref parameters, ref output) => {
                if parameters.iter().any(|parameter| self.occurs(identity, parameter)) {
                    return true;
                }
                if let Some(kind) = output {
                    return self.occurs(identity, kind);
                }
                false
            }
            TypeKind::Structure(ref structure) | TypeKind::Union(ref structure) | TypeKind::Constructor(ref structure) => {
                structure.members.iter().any(|member| self.occurs(identity, member))
            }
            _ => false,
        }
    }

    pub fn unify(&mut self, span: Span<'source>, left: &Type<'source>, right: &Type<'source>) -> Type<'source> {
        let left = self.concretize(left);
        let right = self.concretize(right);

        if left == right {
            return left;
        }

        match (left.kind.clone(), right.kind.clone()) {
            (TypeKind::Variable(identity), _) => {
                if self.occurs(identity, &right) {
                    self.errors.push(CheckError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                    return left;
                }

                self.variables[identity] = Some(right.clone());
                right
            }
            (_, TypeKind::Variable(identity)) => {
                if self.occurs(identity, &left) {
                    self.errors.push(CheckError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                    return left;
                }

                self.variables[identity] = Some(left.clone());
                left
            }
            (TypeKind::Pointer { target: source }, TypeKind::Pointer { target: destination }) => {
                let unified = self.unify(span, &source, &destination);
                Type::new(TypeKind::Pointer { target: Box::new(unified) }, left.span)
            }
            (TypeKind::Array { member: source, size: capacity }, TypeKind::Array { member: destination, size: limit }) if capacity == limit => {
                let unified = self.unify(span, &source, &destination);
                Type::new(TypeKind::Array { member: Box::new(unified), size: capacity }, left.span)
            }
            (TypeKind::Tuple { members: source }, TypeKind::Tuple { members: destination }) if source.len() == destination.len() => {
                let mut unified = Vec::with_capacity(source.len());
                for (source_item, destination_item) in source.iter().zip(destination.iter()) {
                    unified.push(self.unify(span, source_item, destination_item));
                }
                Type::new(TypeKind::Tuple { members: unified }, left.span)
            }
            (TypeKind::Integer { size: source, signed: source_signed }, TypeKind::Integer { size: destination, signed: destination_signed }) => {
                Type::new(TypeKind::Integer { size: source.max(destination), signed: source_signed || destination_signed }, left.span)
            }
            (TypeKind::Float { size: source }, TypeKind::Float { size: destination }) => {
                Type::new(TypeKind::Float { size: source.max(destination) }, left.span)
            }
            (TypeKind::Function(name, source, source_output), TypeKind::Function(_, destination, destination_output)) if source.len() == destination.len() => {
                let mut unified = Vec::with_capacity(source.len());

                for (source_item, destination_item) in source.iter().zip(destination.iter()) {
                    unified.push(self.unify(span, source_item, destination_item));
                }

                let output = match (source_output, destination_output) {
                    (Some(source_kind), Some(destination_kind)) => {
                        Some(Box::new(self.unify(span, &source_kind, &destination_kind)))
                    }
                    (Some(source_kind), None) => {
                        let void = Type::new(TypeKind::Void, span);
                        Some(Box::new(self.unify(span, &source_kind, &void)))
                    }
                    (None, Some(destination_kind)) => {
                        let void = Type::new(TypeKind::Void, span);
                        Some(Box::new(self.unify(span, &void, &destination_kind)))
                    }
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(name, unified, output), left.span)
            }
            _ => {
                self.errors.push(CheckError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                left
            }
        }
    }
}
