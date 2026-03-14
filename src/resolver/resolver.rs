use {
    super::{scope::Scope, ResolveError},
    crate::{
        data::{memory::replace},
        parser::{Element, Symbol},
    },
};
use crate::data::{Identity, Structure};
use crate::resolver::{ErrorKind, Type, TypeKind};
use crate::tracker::Span;

pub struct Resolver<'resolver> {
    pub scope: Scope<Symbol<'resolver>>,
    pub input: Vec<Element<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
    pub history: Vec<Vec<(Identity, Type<'resolver>)>>,
    pub variables: Vec<Option<Type<'resolver>>>,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            input: self.input.clone(),
            errors: self.errors.clone(),
            history: self.history.clone(),
            variables: self.variables.clone(),
        }
    }
}

pub trait Resolvable<'resolvable> {
    fn resolve(
        &mut self,
        resolver: &mut Resolver<'resolvable>,
    );
    fn reify(&mut self, resolver: &mut Resolver<'resolvable>);
}

impl<'resolver> Resolver<'resolver> {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            input: Vec::new(),
            errors: Vec::new(),
            history: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn set_input(&mut self, input: Vec<Element<'resolver>>) {
        self.input = input;
    }

    pub fn enter_scope(&mut self, scope: Scope<Symbol<'resolver>>) {
        let parent = replace(&mut self.scope, scope);
        self.scope.attach(parent);
    }

    pub fn add(&mut self, symbol: Symbol<'resolver>) {
        self.scope.add(symbol);
    }
    pub fn enter(&mut self) {
        self.history.push(Vec::new());
        let parent = replace(&mut self.scope, Scope::new());
        self.scope.attach(parent);
    }

    pub fn exit(&mut self) {
        self.history.pop();
        if let Some(parent) = self.scope.detach() {
            self.scope = parent;
        }
    }

    pub fn bind(&mut self, identity: Identity, typ: Type<'resolver>) {
        if let Some(frame) = self.history.last_mut() {
            frame.push((identity, typ));
        }
    }

    pub fn lookup(&mut self, identity: Identity, span: Span<'resolver>) -> Type<'resolver> {
        for frame in self.history.iter().rev() {
            for (id, typ) in frame.iter().rev() {
                if *id == identity {
                    return typ.clone();
                }
            }
        }
        if let Some(symbol) = self.scope.get_identity(identity) {
            return symbol.typ.clone();
        }
        self.fresh(span)
    }

    pub fn fresh(&mut self, span: Span<'resolver>) -> Type<'resolver> {
        let identity = self.variables.len();
        self.variables.push(None);
        Type::new(TypeKind::Variable(identity), span)
    }

    pub fn reify(&mut self, typ: &Type<'resolver>) -> Type<'resolver> {
        match &typ.kind {
            TypeKind::Variable(identity) => {
                if let Some(resolved) = self.variables[*identity].clone() {
                    let deep = self.reify(&resolved);
                    self.variables[*identity] = Some(deep.clone());
                    deep
                } else {
                    typ.clone()
                }
            }
            TypeKind::Pointer { target } => Type::new(TypeKind::Pointer { target: Box::new(self.reify(target)) }, typ.span),
            TypeKind::Array { member, size } => Type::new(TypeKind::Array { member: Box::new(self.reify(member)), size: *size }, typ.span),
            TypeKind::Tuple { members } => {
                let items = members.iter().map(|item| self.reify(item)).collect();
                Type::new(TypeKind::Tuple { members: items }, typ.span)
            }
            TypeKind::Function(name, parameters, output) => {
                let arguments = parameters.iter().map(|parameter| self.reify(parameter)).collect();
                let returnable = output.as_ref().map(|kind| Box::new(self.reify(kind)));
                Type::new(TypeKind::Function(name.clone(), arguments, returnable), typ.span)
            }
            TypeKind::Structure(structure) => {
                let members = structure.members.iter().map(|member| self.reify(member)).collect();
                Type::new(TypeKind::Structure(Structure::new(structure.target.clone(), members)), typ.span)
            }
            TypeKind::Union(structure) => {
                let members = structure.members.iter().map(|member| self.reify(member)).collect();
                Type::new(TypeKind::Union(Structure::new(structure.target.clone(), members)), typ.span)
            }
            TypeKind::Constructor(structure) => {
                let members = structure.members.iter().map(|member| self.reify(member)).collect();
                Type::new(TypeKind::Constructor(Structure::new(structure.target.clone(), members)), typ.span)
            }
            _ => typ.clone(),
        }
    }

    fn occurs(&mut self, identity: Identity, typ: &Type<'resolver>) -> bool {
        let flattened = self.reify(typ);

        match flattened.kind {
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
            TypeKind::Structure(ref structure) | TypeKind::Union(ref structure) | TypeKind::Constructor(ref structure) => structure.members.iter().any(|member| self.occurs(identity, member)),
            _ => false,
        }
    }

    pub fn unify(&mut self, span: Span<'resolver>, left: &Type<'resolver>, right: &Type<'resolver>) -> Type<'resolver> {
        let left = self.reify(left);
        let right = self.reify(right);

        if left == right {
            return left;
        }

        match (left.kind.clone(), right.kind.clone()) {
            (TypeKind::Variable(identity), _) => {
                if self.occurs(identity, &right) {
                    self.errors.push(ResolveError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                    return left;
                }

                self.variables[identity] = Some(right.clone());
                right
            }
            (_, TypeKind::Variable(identity)) => {
                if self.occurs(identity, &left) {
                    self.errors.push(ResolveError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                    return left;
                }

                self.variables[identity] = Some(left.clone());
                left
            }

            (TypeKind::Array { member: left_member, size: left_size }, TypeKind::Array { member: right_member, size: right_size }) if left_size == right_size => {
                let unified = self.unify(span, &left_member, &right_member);
                Type::new(TypeKind::Array { member: Box::new(unified), size: left_size }, left.span)
            }
            (TypeKind::Pointer { target: left_target }, TypeKind::Pointer { target: right_target }) => {
                let unified = self.unify(span, &left_target, &right_target);
                Type::new(TypeKind::Pointer { target: Box::new(unified) }, left.span)
            }
            (TypeKind::Tuple { members: left_members }, TypeKind::Tuple { members: right_members }) if left_members.len() == right_members.len() => {
                let mut unified = Vec::with_capacity(left_members.len());
                for (l, r) in left_members.iter().zip(right_members.iter()) {
                    unified.push(self.unify(span, l, r));
                }
                Type::new(TypeKind::Tuple { members: unified }, left.span)
            }

            (TypeKind::Structure(left_struct), TypeKind::Structure(right_struct)) if left_struct.target == right_struct.target && left_struct.members.len() == right_struct.members.len() => {
                let mut unified = Vec::with_capacity(left_struct.members.len());
                for (l, r) in left_struct.members.iter().zip(right_struct.members.iter()) {
                    unified.push(self.unify(span, l, r));
                }
                Type::new(TypeKind::Structure(Structure::new(left_struct.target.clone(), unified)), left.span)
            }
            (TypeKind::Union(left_struct), TypeKind::Union(right_struct)) if left_struct.target == right_struct.target && left_struct.members.len() == right_struct.members.len() => {
                let mut unified = Vec::with_capacity(left_struct.members.len());
                for (l, r) in left_struct.members.iter().zip(right_struct.members.iter()) {
                    unified.push(self.unify(span, l, r));
                }
                Type::new(TypeKind::Union(Structure::new(left_struct.target.clone(), unified)), left.span)
            }
            (TypeKind::Constructor(left_struct), TypeKind::Constructor(right_struct)) if left_struct.target == right_struct.target && left_struct.members.len() == right_struct.members.len() => {
                let mut unified = Vec::with_capacity(left_struct.members.len());
                for (l, r) in left_struct.members.iter().zip(right_struct.members.iter()) {
                    unified.push(self.unify(span, l, r));
                }
                Type::new(TypeKind::Constructor(Structure::new(left_struct.target.clone(), unified)), left.span)
            }

            (TypeKind::Integer { size: left_size, .. }, TypeKind::Integer { size: right_size, .. }) if left_size == right_size => left,
            (TypeKind::Float { size: left_size }, TypeKind::Float { size: right_size }) if left_size == right_size => left,
            (TypeKind::Pointer { target }, TypeKind::String) | (TypeKind::String, TypeKind::Pointer { target }) if matches!(target.kind, TypeKind::Integer { size: 8, .. }) => left,
            (TypeKind::Pointer { .. }, TypeKind::Integer { .. }) | (TypeKind::Integer { .. }, TypeKind::Pointer { .. }) => left,

            (TypeKind::Function(name, left_params, left_output), TypeKind::Function(_, right_params, right_output)) if left_params.len() == right_params.len() => {
                let mut unified = Vec::with_capacity(left_params.len());

                for (l, r) in left_params.iter().zip(right_params.iter()) {
                    unified.push(self.unify(span, l, r));
                }

                let output = match (left_output, right_output) {
                    (Some(left_kind), Some(right_kind)) => Some(Box::new(self.unify(span, &left_kind, &right_kind))),
                    (Some(left_kind), None) => Some(left_kind),
                    (None, Some(right_kind)) => Some(right_kind),
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(name, unified, output), left.span)
            }
            _ => {
                self.errors.push(ResolveError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                left
            }
        }
    }

    pub fn resolve(&mut self) {
        let mut input = self.input.clone();

        for element in input.iter_mut() {
            element.resolve(self);
        }

        self.input = input;
    }
}
