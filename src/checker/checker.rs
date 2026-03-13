use crate::{
    checker::{CheckError, ErrorKind, Type, TypeKind},
    data::{memory::take, Identity, Structure},
    internal::hash::Map,
    parser::{Element, Symbol, SymbolKind},
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
    fn reify(&mut self, checker: &mut Checker<'_, 'source>);
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
            element.reify(self);
        }

        let environment = take(&mut self.environment);
        let mut resolved = Map::with_capacity(environment.len());

        for (identity, typ) in environment {
            resolved.insert(identity, self.reify(&typ));
        }

        self.environment = resolved;
        *self.input = elements;
    }

    pub fn reify(&mut self, typ: &Type<'source>) -> Type<'source> {
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

    fn occurs(&mut self, identity: Identity, typ: &Type<'source>) -> bool {
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

    pub fn unify(&mut self, span: Span<'source>, left: &Type<'source>, right: &Type<'source>) -> Type<'source> {
        let left = self.reify(left);
        let right = self.reify(right);

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

            (TypeKind::Array { member: src_member, size: src_size }, TypeKind::Array { member: dst_member, size: dst_size }) if src_size == dst_size => {
                let unified = self.unify(span, &src_member, &dst_member);
                Type::new(TypeKind::Array { member: Box::new(unified), size: src_size }, left.span)
            }
            (TypeKind::Pointer { target: src_target }, TypeKind::Pointer { target: dst_target }) => {
                let unified = self.unify(span, &src_target, &dst_target);
                Type::new(TypeKind::Pointer { target: Box::new(unified) }, left.span)
            }
            (TypeKind::Tuple { members: src_members }, TypeKind::Tuple { members: dst_members }) if src_members.len() == dst_members.len() => {
                let mut unified = Vec::with_capacity(src_members.len());
                for (src_item, dst_item) in src_members.iter().zip(dst_members.iter()) {
                    unified.push(self.unify(span, src_item, dst_item));
                }
                Type::new(TypeKind::Tuple { members: unified }, left.span)
            }

            (TypeKind::Structure(src), TypeKind::Structure(dst)) if src.target == dst.target && src.members.len() == dst.members.len() => {
                let mut unified = Vec::with_capacity(src.members.len());
                for (src_item, dst_item) in src.members.iter().zip(dst.members.iter()) {
                    unified.push(self.unify(span, src_item, dst_item));
                }
                Type::new(TypeKind::Structure(Structure::new(src.target.clone(), unified)), left.span)
            }
            (TypeKind::Union(src), TypeKind::Union(dst)) if src.target == dst.target && src.members.len() == dst.members.len() => {
                let mut unified = Vec::with_capacity(src.members.len());
                for (src_item, dst_item) in src.members.iter().zip(dst.members.iter()) {
                    unified.push(self.unify(span, src_item, dst_item));
                }
                Type::new(TypeKind::Union(Structure::new(src.target.clone(), unified)), left.span)
            }
            (TypeKind::Constructor(src), TypeKind::Constructor(dst)) if src.target == dst.target && src.members.len() == dst.members.len() => {
                let mut unified = Vec::with_capacity(src.members.len());
                for (src_item, dst_item) in src.members.iter().zip(dst.members.iter()) {
                    unified.push(self.unify(span, src_item, dst_item));
                }
                Type::new(TypeKind::Constructor(Structure::new(src.target.clone(), unified)), left.span)
            }

            (TypeKind::Integer { size: src_size, signed: src_signed }, TypeKind::Integer { size: dst_size, signed: dst_signed }) if src_size == dst_size && src_signed == dst_signed => {
                Type::new(TypeKind::Integer { size: src_size, signed: src_signed }, left.span)
            }
            (TypeKind::Float { size: src_size }, TypeKind::Float { size: dst_size }) if src_size == dst_size => {
                Type::new(TypeKind::Float { size: src_size }, left.span)
            }

            (TypeKind::Function(name, src_params, src_output), TypeKind::Function(_, dst_params, dst_output)) if src_params.len() == dst_params.len() => {
                let mut unified = Vec::with_capacity(src_params.len());

                for (src_item, dst_item) in src_params.iter().zip(dst_params.iter()) {
                    unified.push(self.unify(span, src_item, dst_item));
                }

                let output = match (src_output, dst_output) {
                    (Some(src_kind), Some(dst_kind)) => Some(Box::new(self.unify(span, &src_kind, &dst_kind))),
                    (Some(src_kind), None) => {
                        let void = Type::new(TypeKind::Void, span);
                        Some(Box::new(self.unify(span, &src_kind, &void)))
                    }
                    (None, Some(dst_kind)) => {
                        let void = Type::new(TypeKind::Void, span);
                        Some(Box::new(self.unify(span, &void, &dst_kind)))
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
