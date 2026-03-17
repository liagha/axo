use crate::{
    data::{memory::replace, Identity, Scale},
    parser::{Element, ElementKind, Symbol},
    resolver::{scope::Scope, ErrorKind, ResolveError, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
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
    fn is_instance(&self) -> bool {
        false
    }
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

    pub fn fresh(&mut self) -> Type<'resolver> {
        let identity = self.variables.len();

        self.variables.push(None);

        Type::new(identity, TypeKind::Variable(identity))
    }

    pub fn lookup(&mut self, identity: Identity) -> Type<'resolver> {
        if let Some(symbol) = self.scope.find(identity) {
            return symbol.typing.clone();
        }
        self.fresh()
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
            TypeKind::Tuple { members } => members.iter().any(|item| self.occurs(identity, item)),
            TypeKind::Function(_, parameters, output) => {
                if parameters.iter().any(|item| self.occurs(identity, item)) {
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

            (TypeKind::Array { member: left_item, size: left_size }, TypeKind::Array { member: right_item, size: right_size }) if left_size == right_size => {
                let unified = self.unify(span, &left_item, &right_item);
                Type::from_kind(TypeKind::Array { member: Box::new(unified), size: left_size })
            }
            (TypeKind::Pointer { target: left_target }, TypeKind::Pointer { target: right_target }) => {
                let unified = self.unify(span, &left_target, &right_target);
                Type::from_kind(TypeKind::Pointer { target: Box::new(unified) })
            }
            (TypeKind::Tuple { members: left_items }, TypeKind::Tuple { members: right_items }) if left_items.len() == right_items.len() => {
                let mut unified = Vec::with_capacity(left_items.len());
                for (first, second) in left_items.iter().zip(right_items.iter()) {
                    unified.push(self.unify(span, first, second));
                }
                Type::from_kind(TypeKind::Tuple { members: unified })
            }

            (TypeKind::Structure(_), TypeKind::Structure(_))
            | (TypeKind::Union(_), TypeKind::Union(_))
            | (TypeKind::Enumeration(_), TypeKind::Enumeration(_)) if left.identity == right.identity => left,

            (TypeKind::Integer { size: left_size, signed: left_signed }, TypeKind::Integer { size: right_size, signed: right_signed }) if left_size == right_size && left_signed == right_signed => left,
            (TypeKind::Float { size: left_size }, TypeKind::Float { size: right_size }) if left_size == right_size => left,

            (TypeKind::Function(left_name, left_args, left_output), TypeKind::Function(right_name, right_args, right_output)) if left_args.len() == right_args.len() => {
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

                let name = if left_name.is_empty() { right_name } else { left_name };

                Type::new(left.identity, TypeKind::Function(name, unified, output))
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
            TypeKind::Pointer { target } => Type::from_kind(TypeKind::Pointer { target: Box::new(self.reify(target)) }),
            TypeKind::Array { member, size } => Type::from_kind(TypeKind::Array { member: Box::new(self.reify(member)), size: *size }),
            TypeKind::Tuple { members } => {
                let items = members.iter().map(|item| self.reify(item)).collect();
                Type::from_kind(TypeKind::Tuple { members: items })
            }
            TypeKind::Function(name, parameters, output) => {
                let arguments = parameters.iter().map(|item| self.reify(item)).collect();
                let returnable = output.as_ref().map(|kind| Box::new(self.reify(kind)));

                Type::new(typing.identity, TypeKind::Function(name.clone(), arguments, returnable))
            }
            _ => typing.clone(),
        }
    }

    pub fn evaluate(&self, element: &Element<'resolver>) -> Result<Scale, ResolveError<'resolver>> {
        match &element.kind {
            ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) => Ok(*value as Scale),
            ElementKind::Binary(binary) => {
                let left = self.evaluate(&binary.left)?;
                let right = self.evaluate(&binary.right)?;

                if let TokenKind::Operator(operator) = &binary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Plus] => Ok(left + right),
                        [OperatorKind::Minus] => Ok(left - right),
                        [OperatorKind::Star] => Ok(left * right),
                        [OperatorKind::Slash] => Ok(left / right),
                        [OperatorKind::Percent] => Ok(left % right),
                        _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
                    }
                } else {
                    Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span))
                }
            }
            _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
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
                    "Void" | "Unit" => TypeKind::Tuple { members: Vec::new() },
                    _ => {
                        if let Some(identity) = element.reference {
                            let typing = self.lookup(identity);

                            return Ok(typing);
                        }
                        return Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), *span));
                    }
                };

                Ok(Type::from_kind(kind))
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
                    let size = self.evaluate(&delimited.members[1])?;

                    Ok(Type::from_kind(TypeKind::Array { member: Box::new(member), size }))
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    _,
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.members.is_empty() {
                        Ok(Type::from_kind(TypeKind::Tuple { members: Vec::new() }))
                    } else if delimited.separator.is_none() && delimited.members.len() == 1 {
                        self.annotation(&delimited.members[0])
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());
                        for member in &delimited.members {
                            members.push(self.annotation(member)?);
                        }
                        Ok(Type::from_kind(TypeKind::Tuple { members }))
                    }
                }

                _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
            },

            ElementKind::Unary(unary) => {
                if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
                    let item = self.annotation(&unary.operand)?;
                    Ok(Type::new(item.identity, TypeKind::Pointer { target: Box::from(item) }))
                } else {
                    Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span))
                }
            }

            _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
        }
    }
}
