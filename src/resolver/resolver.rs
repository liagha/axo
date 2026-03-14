use {
    crate::{
        data::{
            Identity,
            Scale, Str,
            memory::replace,
        },
        parser::{Element, ElementKind, Symbol},
        scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        resolver::{
            ResolveError,
            ErrorKind,
            Type, TypeKind,
            scope::Scope,
        },
        tracker::Span,
    },
};

pub struct Resolver<'resolver> {
    pub scope: Scope<Symbol<'resolver>>,
    pub input: Vec<Element<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
    pub variables: Vec<Option<Type<'resolver>>>,
    pub returns: Vec<Type<'resolver>>,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            input: self.input.clone(),
            errors: self.errors.clone(),
            variables: self.variables.clone(),
            returns: self.returns.clone(),
        }
    }
}

pub trait Resolvable<'resolvable> {
    fn declare(&mut self, resolver: &mut Resolver<'resolvable>);
    fn resolve(&mut self, resolver: &mut Resolver<'resolvable>);
    fn reify(&mut self, resolver: &mut Resolver<'resolvable>);
}

impl<'resolver> Resolver<'resolver> {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            input: Vec::new(),
            errors: Vec::new(),
            variables: Vec::new(),
            returns: Vec::new(),
        }
    }

    pub fn set_input(&mut self, input: Vec<Element<'resolver>>) {
        self.input = input;
    }

    pub fn resolve(&mut self) {
        let mut input = self.input.clone();

        for element in input.iter_mut() {
            element.resolve(self);
        }

        for element in input.iter_mut() {
            element.reify(self);
        }

        self.input = input;
    }

    pub fn enter(&mut self) {
        let parent = replace(&mut self.scope, Scope::new());
        self.scope.attach(parent);
    }

    pub fn enter_scope(&mut self, scope: Scope<Symbol<'resolver>>) {
        let parent = replace(&mut self.scope, scope);
        self.scope.attach(parent);
    }

    pub fn exit(&mut self) {
        if let Some(parent) = self.scope.detach() {
            self.scope = parent;
        }
    }

    pub fn insert(&mut self, symbol: Symbol<'resolver>) {
        self.scope.insert(symbol);
    }

    pub fn fresh(&mut self, span: Span<'resolver>) -> Type<'resolver> {
        let identity = self.variables.len();
        self.variables.push(None);
        Type::new(TypeKind::Variable(identity), span)
    }

    pub fn lookup(&mut self, identity: Identity, span: Span<'resolver>) -> Type<'resolver> {
        if let Some(symbol) = self.scope.find(identity) {
            return symbol.typing.clone();
        }
        self.fresh(span)
    }

    pub fn occurs(&self, identity: Identity, typing: &Type<'resolver>) -> bool {
        match &typing.kind {
            TypeKind::Variable(variable) => {
                if identity == *variable {
                    return true;
                }
                if let Some(resolved) = &self.variables[*variable] {
                    return self.occurs(identity, resolved);
                }
                false
            }
            TypeKind::Pointer { target } => self.occurs(identity, target),
            TypeKind::Array { member, .. } => self.occurs(identity, member),
            TypeKind::Tuple { members } => members.iter().any(|member| self.occurs(identity, member)),
            TypeKind::Function(_, parameters, output) => {
                if parameters.iter().any(|parameter| self.occurs(identity, parameter)) {
                    return true;
                }
                if let Some(kind) = output {
                    return self.occurs(identity, kind);
                }
                false
            }
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
            (TypeKind::Unknown, _) => right.clone(),
            (_, TypeKind::Unknown) => left.clone(),

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
            (TypeKind::Tuple { members: left_items }, TypeKind::Tuple { members: right_items }) if left_items.len() == right_items.len() => {
                let mut unified = Vec::with_capacity(left_items.len());
                for (first, second) in left_items.iter().zip(right_items.iter()) {
                    unified.push(self.unify(span, first, second));
                }
                Type::new(TypeKind::Tuple { members: unified }, left.span)
            }

            (TypeKind::Structure(left_identity, _), TypeKind::Structure(right_identity, _)) if left_identity == right_identity => left,
            (TypeKind::Union(left_identity, _), TypeKind::Union(right_identity, _)) if left_identity == right_identity => left,
            (TypeKind::Constructor(left_identity, _), TypeKind::Constructor(right_identity, _)) if left_identity == right_identity => left,

            (TypeKind::Integer { .. }, TypeKind::Integer { .. }) => left,
            (TypeKind::Float { .. }, TypeKind::Float { .. }) => left,
            (TypeKind::Pointer { target }, TypeKind::String) | (TypeKind::String, TypeKind::Pointer { target }) if matches!(target.kind, TypeKind::Integer { size: 8, .. }) => left,

            (TypeKind::Pointer { .. }, TypeKind::Integer { .. }) | (TypeKind::Integer { .. }, TypeKind::Pointer { .. }) => left,

            (TypeKind::Function(name, left_args, left_output), TypeKind::Function(_, right_args, right_output)) if left_args.len() == right_args.len() => {
                let mut unified = Vec::with_capacity(left_args.len());

                for (first, second) in left_args.iter().zip(right_args.iter()) {
                    unified.push(self.unify(span, first, second));
                }

                let output = match (left_output, right_output) {
                    (Some(first), Some(second)) => Some(Box::new(self.unify(span, &first, &second))),
                    (Some(first), None) => Some(first),
                    (None, Some(second)) => Some(second),
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

    pub fn reify(&mut self, typing: &Type<'resolver>) -> Type<'resolver> {
        match &typing.kind {
            TypeKind::Variable(identity) => {
                if let Some(resolved) = self.variables[*identity].clone() {
                    let deep = self.reify(&resolved);
                    self.variables[*identity] = Some(deep.clone());
                    deep
                } else {
                    typing.clone()
                }
            }
            TypeKind::Pointer { target } => Type::new(TypeKind::Pointer { target: Box::new(self.reify(target)) }, typing.span),
            TypeKind::Array { member, size } => Type::new(TypeKind::Array { member: Box::new(self.reify(member)), size: *size }, typing.span),
            TypeKind::Tuple { members } => {
                let items = members.iter().map(|item| self.reify(item)).collect();
                Type::new(TypeKind::Tuple { members: items }, typing.span)
            }
            TypeKind::Function(name, parameters, output) => {
                let arguments = parameters.iter().map(|parameter| self.reify(parameter)).collect();
                let returnable = output.as_ref().map(|kind| Box::new(self.reify(kind)));
                Type::new(TypeKind::Function(name.clone(), arguments, returnable), typing.span)
            }
            _ => typing.clone(),
        }
    }

    pub fn annotation(&mut self, element: &Element<'resolver>) -> Result<Type<'resolver>, ResolveError<'resolver>> {
        match &element.kind {
            ElementKind::Literal(Token { kind: TokenKind::Identifier(name), span }) => {
                let text = name.as_str().unwrap();

                let kind = match text {
                    "Int8" => TypeKind::Integer { size: 8, signed: true },
                    "Int16" => TypeKind::Integer { size: 16, signed: true },
                    "Int32" => TypeKind::Integer { size: 32, signed: true },
                    "Int64" | "Integer" => TypeKind::Integer { size: 64, signed: true },
                    "UInt8" => TypeKind::Integer { size: 8, signed: false },
                    "UInt16" => TypeKind::Integer { size: 16, signed: false },
                    "UInt32" => TypeKind::Integer { size: 32, signed: false },
                    "UInt64" => TypeKind::Integer { size: 64, signed: false },
                    "Float32" => TypeKind::Float { size: 32 },
                    "Float64" | "Float" => TypeKind::Float { size: 64 },
                    "Boolean" => TypeKind::Boolean,
                    "Character" => TypeKind::Character,
                    "String" => TypeKind::String,
                    _ => {
                        if let Some(identity) = element.reference {
                            return Ok(self.lookup(identity, *span));
                        }
                        return Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), *span));
                    }
                };

                Ok(Type::new(kind, *span))
            }

            ElementKind::Delimited(delimited) => match (
                &delimited.start.kind,
                delimited.separator.as_ref().map(|token| &token.kind),
                &delimited.end.kind,
            ) {
                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    if delimited.members.len() != 2 {
                        return Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span));
                    }

                    let member = self.annotation(&delimited.members[0])?;
                    let size = match delimited.members[1].kind {
                        ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) => value as Scale,
                        _ => return Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
                    };

                    Ok(Type::new(TypeKind::Array { member: Box::new(member), size }, element.span))
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    let mut members = Vec::with_capacity(delimited.members.len());
                    for member in &delimited.members {
                        members.push(self.annotation(member)?);
                    }
                    Ok(Type::new(TypeKind::Tuple { members }, element.span))
                }

                _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
            },

            ElementKind::Unary(unary) => {
                if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
                    let item = self.annotation(&unary.operand)?;
                    Ok(Type::new(TypeKind::Pointer { target: Box::from(item) }, element.span))
                } else {
                    Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span))
                }
            }

            ElementKind::Binary(binary) => {
                let TokenKind::Operator(operator) = &binary.operator.kind else {
                    return Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span));
                };

                match operator.as_slice() {
                    [OperatorKind::Minus, OperatorKind::RightAngle] => {
                        let mut parameters = Vec::new();

                        match &binary.left.kind {
                            ElementKind::Delimited(delimited) => {
                                for member in &delimited.members {
                                    parameters.push(self.annotation(member)?);
                                }
                            }
                            _ => {
                                parameters.push(self.annotation(&binary.left)?);
                            }
                        }

                        let output = self.annotation(&binary.right)?;

                        Ok(Type::new(TypeKind::Function(Str::default(), parameters, Some(Box::new(output))), element.span))
                    }
                    _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
                }
            }

            _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
        }
    }
}
