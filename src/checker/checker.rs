use crate::{
    checker::{CheckError, Type, TypeKind},
    data::{Identity, Structure},
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::Resolver,
    tracker::Span,
};
use std::collections::HashMap;
use crate::checker::ErrorKind;

pub struct Checker<'check, 'source> {
    pub input: &'check mut Vec<Element<'source>>,
    pub resolver: &'check Resolver<'source>,
    pub environment: HashMap<Identity, Type<'source>>,
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
            environment: HashMap::new(),
            errors: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn check(&mut self) {
        let mut elements = std::mem::take(self.input);

        for element in &mut elements {
            element.check(self);
        }

        for element in &mut elements {
            self.concretize_element(element);
        }

        let environment = std::mem::take(&mut self.environment);
        let mut concretized_environment = HashMap::with_capacity(environment.len());

        for (identity, type_value) in environment {
            concretized_environment.insert(identity, self.concretize(&type_value));
        }

        self.environment = concretized_environment;
        *self.input = elements;
    }

    pub fn concretize_element(&mut self, element: &mut Element<'source>) {
        element.ty = self.concretize(&element.ty);

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
        symbol.ty = self.concretize(&symbol.ty);

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
            _ => {}
        }
    }

    pub fn lookup(&mut self, identity: Identity, span: Span<'source>) -> Type<'source> {
        if let Some(type_value) = self.environment.get(&identity) {
            return type_value.clone();
        }

        if let Some(symbol) = self.resolver.scope.get_identity(identity) {
            let mut cloned = symbol.clone();

            let variable = self.fresh(span);
            self.environment.insert(identity, variable.clone());

            cloned.check(self);

            let unified = self.unify(span, &variable, &cloned.ty);
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

    pub fn concretize(&mut self, type_value: &Type<'source>) -> Type<'source> {
        match &type_value.kind {
            TypeKind::Variable(identity) => {
                if let Some(resolved) = self.variables[*identity].clone() {
                    let deep = self.concretize(&resolved);
                    self.variables[*identity] = Some(deep.clone());
                    deep
                } else {
                    type_value.clone()
                }
            }
            TypeKind::Pointer { target } => {
                Type::new(TypeKind::Pointer { target: Box::new(self.concretize(target)) }, type_value.span)
            }
            TypeKind::Array { member, size } => {
                Type::new(TypeKind::Array { member: Box::new(self.concretize(member)), size: *size }, type_value.span)
            }
            TypeKind::Tuple { members } => {
                Type::new(TypeKind::Tuple { members: members.iter().map(|member| self.concretize(member)).collect() }, type_value.span)
            }
            TypeKind::Function(name, parameters, output) => {
                Type::new(
                    TypeKind::Function(
                        name.clone(),
                        parameters.iter().map(|parameter| self.concretize(parameter)).collect(),
                        output.as_ref().map(|output_type| Box::new(self.concretize(output_type))),
                    ),
                    type_value.span,
                )
            }
            TypeKind::Structure(structure) => {
                let members = structure.members.iter().map(|member| self.concretize(member)).collect();
                Type::new(TypeKind::Structure(Structure::new(structure.target.clone(), members)), type_value.span)
            }
            TypeKind::Union(structure) => {
                let members = structure.members.iter().map(|member| self.concretize(member)).collect();
                Type::new(TypeKind::Union(Structure::new(structure.target.clone(), members)), type_value.span)
            }
            TypeKind::Constructor(structure) => {
                let members = structure.members.iter().map(|member| self.concretize(member)).collect();
                Type::new(TypeKind::Constructor(Structure::new(structure.target.clone(), members)), type_value.span)
            }
            _ => type_value.clone(),
        }
    }

    fn occurs(&mut self, identity: Identity, type_value: &Type<'source>) -> bool {
        let concretized = self.concretize(type_value);

        match concretized.kind {
            TypeKind::Variable(variable_identity) => identity == variable_identity,
            TypeKind::Pointer { ref target } => self.occurs(identity, target),
            TypeKind::Array { ref member, .. } => self.occurs(identity, member),
            TypeKind::Tuple { ref members } => members.iter().any(|member| self.occurs(identity, member)),
            TypeKind::Function(_, ref parameters, ref output) => {
                if parameters.iter().any(|parameter| self.occurs(identity, parameter)) {
                    return true;
                }
                if let Some(output_type) = output {
                    return self.occurs(identity, output_type);
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
            (TypeKind::Pointer { target: left_target }, TypeKind::Pointer { target: right_target }) => {
                let unified_target = self.unify(span, &left_target, &right_target);
                Type::new(TypeKind::Pointer { target: Box::new(unified_target) }, left.span)
            }
            (TypeKind::Array { member: left_member, size: left_size }, TypeKind::Array { member: right_member, size: right_size }) if left_size == right_size => {
                let unified_member = self.unify(span, &left_member, &right_member);
                Type::new(TypeKind::Array { member: Box::new(unified_member), size: left_size }, left.span)
            }
            (TypeKind::Tuple { members: left_members }, TypeKind::Tuple { members: right_members }) if left_members.len() == right_members.len() => {
                let mut unified_members = Vec::with_capacity(left_members.len());
                for (left_member, right_member) in left_members.iter().zip(right_members.iter()) {
                    unified_members.push(self.unify(span, left_member, right_member));
                }
                Type::new(TypeKind::Tuple { members: unified_members }, left.span)
            }
            (TypeKind::Integer { size: left_size, signed: left_signed }, TypeKind::Integer { size: right_size, signed: right_signed }) => {
                Type::new(TypeKind::Integer { size: left_size.max(right_size), signed: left_signed || right_signed }, left.span)
            }
            (TypeKind::Float { size: left_size }, TypeKind::Float { size: right_size }) => {
                Type::new(TypeKind::Float { size: left_size.max(right_size) }, left.span)
            }
            (TypeKind::Function(name, left_parameters, left_output), TypeKind::Function(_, right_parameters, right_output)) if left_parameters.len() == right_parameters.len() => {
                let mut unified_parameters = Vec::with_capacity(left_parameters.len());

                for (left_parameter, right_parameter) in left_parameters.iter().zip(right_parameters.iter()) {
                    unified_parameters.push(self.unify(span, left_parameter, right_parameter));
                }

                let unified_output = match (left_output, right_output) {
                    (Some(left_out), Some(right_out)) => {
                        Some(Box::new(self.unify(span, &left_out, &right_out)))
                    }
                    (Some(left_out), None) => {
                        let void_type = Type::new(TypeKind::Void, span);
                        Some(Box::new(self.unify(span, &left_out, &void_type)))
                    }
                    (None, Some(right_out)) => {
                        let void_type = Type::new(TypeKind::Void, span);
                        Some(Box::new(self.unify(span, &void_type, &right_out)))
                    }
                    (None, None) => None,
                };

                Type::new(TypeKind::Function(name, unified_parameters, unified_output), left.span)
            }
            _ => {
                self.errors.push(CheckError::new(ErrorKind::Mismatch(left.clone(), right.clone()), span));
                left
            }
        }
    }
}
