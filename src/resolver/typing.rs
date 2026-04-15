use orbyte::Orbyte;
use crate::{
    data::{Aggregate, Boolean, Function, Identity, Interface, Scale, Str},
    parser::{Element, ElementKind},
    resolver::{ErrorKind, ResolveError, Resolver},
    scanner::{OperatorKind, PunctuationKind, TokenKind},
    tracker::Span,
};

#[derive(Clone, Debug, Orbyte)]
pub struct Type<'typing> {
    pub identity: Identity,
    pub kind: TypeKind<'typing>,
}

impl<'typing> Type<'typing> {
    pub fn new(identity: Identity, kind: TypeKind<'typing>) -> Self {
        Self { identity, kind }
    }
}

impl<'typing> From<TypeKind<'typing>> for Type<'typing> {
    fn from(kind: TypeKind<'typing>) -> Self {
        Self::new(0, kind)
    }
}

#[derive(Clone, Debug, Orbyte, PartialEq)]
pub enum TypeKind<'typing> {
    Integer { size: Scale, signed: Boolean },
    Float { size: Scale },
    Boolean,
    String,
    Character,
    Pointer { target: Box<Type<'typing>> },
    Array { member: Box<Type<'typing>>, size: Scale },
    Tuple { members: Box<Vec<Type<'typing>>> },
    Void,
    Variable(Identity),
    Unknown,
    Any,
    Type,
    Has(Str<'typing>, Box<Type<'typing>>),
    And(Box<Type<'typing>>, Box<Type<'typing>>),
    Or(Box<Type<'typing>>, Box<Type<'typing>>),
    Module(Str<'typing>),
    Structure(Box<Aggregate<Str<'typing>, Type<'typing>>>),
    Union(Box<Aggregate<Str<'typing>, Type<'typing>>>),
    Function(Box<Function<Str<'typing>, Type<'typing>, Type<'typing>, Option<Box<Type<'typing>>>>>),
}

impl<'typing> TypeKind<'typing> {
    #[inline(always)]
    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module(_))
    }

    #[inline(always)]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer { .. })
    }

    #[inline(always)]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float { .. })
    }

    #[inline(always)]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean)
    }

    #[inline(always)]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    #[inline(always)]
    pub fn is_character(&self) -> bool {
        matches!(self, Self::Character)
    }

    #[inline(always)]
    pub fn is_pointer(&self) -> bool {
        matches!(self, Self::Pointer { .. })
    }

    #[inline(always)]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }

    #[inline(always)]
    pub fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple { .. })
    }

    #[inline(always)]
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    #[inline(always)]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    #[inline(always)]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    #[inline(always)]
    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    #[inline(always)]
    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }

    #[inline(always)]
    pub fn is_has(&self) -> bool {
        matches!(self, Self::Has(_, _))
    }

    #[inline(always)]
    pub fn is_and(&self) -> bool {
        matches!(self, Self::And(_, _))
    }

    #[inline(always)]
    pub fn is_or(&self) -> bool {
        matches!(self, Self::Or(_, _))
    }

    #[inline(always)]
    pub fn is_structure(&self) -> bool {
        matches!(self, Self::Structure(_))
    }

    #[inline(always)]
    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    #[inline(always)]
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_module(self) -> Str<'typing> {
        match self {
            Self::Module(module) => module,
            _ => panic!("expected module"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_integer(self) -> (Scale, Boolean) {
        match self {
            Self::Integer { size, signed } => (size, signed),
            _ => panic!("expected integer"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_float(self) -> Scale {
        match self {
            Self::Float { size } => size,
            _ => panic!("expected float"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_pointer(self) -> Box<Type<'typing>> {
        match self {
            Self::Pointer { target } => target,
            _ => panic!("expected pointer"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_array(self) -> (Box<Type<'typing>>, Scale) {
        match self {
            Self::Array { member, size } => (member, size),
            _ => panic!("expected array"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_tuple(self) -> Box<Vec<Type<'typing>>> {
        match self {
            Self::Tuple { members } => members,
            _ => panic!("expected tuple"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_variable(self) -> Identity {
        match self {
            Self::Variable(identity) => identity,
            _ => panic!("expected variable"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_has(self) -> (Str<'typing>, Box<Type<'typing>>) {
        match self {
            Self::Has(name, target) => (name, target),
            _ => panic!("expected has"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_and(self) -> (Box<Type<'typing>>, Box<Type<'typing>>) {
        match self {
            Self::And(left, right) => (left, right),
            _ => panic!("expected and"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_or(self) -> (Box<Type<'typing>>, Box<Type<'typing>>) {
        match self {
            Self::Or(left, right) => (left, right),
            _ => panic!("expected or"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_structure(self) -> Box<Aggregate<Str<'typing>, Type<'typing>>> {
        match self {
            Self::Structure(structure) => structure,
            _ => panic!("expected structure"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_union(self) -> Box<Aggregate<Str<'typing>, Type<'typing>>> {
        match self {
            Self::Union(union) => union,
            _ => panic!("expected union"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_function(self) -> Box<Function<Str<'typing>, Type<'typing>, Type<'typing>, Option<Box<Type<'typing>>>>> {
        match self {
            Self::Function(function) => function,
            _ => panic!("expected function"),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_module(&self) -> Option<&Str<'typing>> {
        match self {
            Self::Module(module) => Some(module),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_integer(&self) -> Option<(&Scale, &Boolean)> {
        match self {
            Self::Integer { size, signed } => Some((size, signed)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_float(&self) -> Option<&Scale> {
        match self {
            Self::Float { size } => Some(size),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_pointer(&self) -> Option<&Box<Type<'typing>>> {
        match self {
            Self::Pointer { target } => Some(target),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_array(&self) -> Option<(&Box<Type<'typing>>, &Scale)> {
        match self {
            Self::Array { member, size } => Some((member, size)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_tuple(&self) -> Option<&Box<Vec<Type<'typing>>>> {
        match self {
            Self::Tuple { members } => Some(members),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_variable(&self) -> Option<&Identity> {
        match self {
            Self::Variable(identity) => Some(identity),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_has(&self) -> Option<(&Str<'typing>, &Box<Type<'typing>>)> {
        match self {
            Self::Has(name, target) => Some((name, target)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_and(&self) -> Option<(&Box<Type<'typing>>, &Box<Type<'typing>>)> {
        match self {
            Self::And(left, right) => Some((left, right)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_or(&self) -> Option<(&Box<Type<'typing>>, &Box<Type<'typing>>)> {
        match self {
            Self::Or(left, right) => Some((left, right)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_structure(&self) -> Option<&Box<Aggregate<Str<'typing>, Type<'typing>>>> {
        match self {
            Self::Structure(structure) => Some(structure),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_union(&self) -> Option<&Box<Aggregate<Str<'typing>, Type<'typing>>>> {
        match self {
            Self::Union(union) => Some(union),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_function(&self) -> Option<&Box<Function<Str<'typing>, Type<'typing>, Type<'typing>, Option<Box<Type<'typing>>>>>> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_module_mut(&mut self) -> Option<&mut Str<'typing>> {
        match self {
            Self::Module(module) => Some(module),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_integer_mut(&mut self) -> Option<(&mut Scale, &mut Boolean)> {
        match self {
            Self::Integer { size, signed } => Some((size, signed)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_float_mut(&mut self) -> Option<&mut Scale> {
        match self {
            Self::Float { size } => Some(size),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_pointer_mut(&mut self) -> Option<&mut Box<Type<'typing>>> {
        match self {
            Self::Pointer { target } => Some(target),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_array_mut(&mut self) -> Option<(&mut Box<Type<'typing>>, &mut Scale)> {
        match self {
            Self::Array { member, size } => Some((member, size)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_tuple_mut(&mut self) -> Option<&mut Box<Vec<Type<'typing>>>> {
        match self {
            Self::Tuple { members } => Some(members),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_variable_mut(&mut self) -> Option<&mut Identity> {
        match self {
            Self::Variable(identity) => Some(identity),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_has_mut(&mut self) -> Option<(&mut Str<'typing>, &mut Box<Type<'typing>>)> {
        match self {
            Self::Has(name, target) => Some((name, target)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_and_mut(&mut self) -> Option<(&mut Box<Type<'typing>>, &mut Box<Type<'typing>>)> {
        match self {
            Self::And(left, right) => Some((left, right)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_or_mut(&mut self) -> Option<(&mut Box<Type<'typing>>, &mut Box<Type<'typing>>)> {
        match self {
            Self::Or(left, right) => Some((left, right)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_structure_mut(&mut self) -> Option<&mut Box<Aggregate<Str<'typing>, Type<'typing>>>> {
        match self {
            Self::Structure(structure) => Some(structure),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_union_mut(&mut self) -> Option<&mut Box<Aggregate<Str<'typing>, Type<'typing>>>> {
        match self {
            Self::Union(union) => Some(union),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_function_mut(&mut self) -> Option<&mut Box<Function<Str<'typing>, Type<'typing>, Type<'typing>, Option<Box<Type<'typing>>>>>> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }
}

impl<'typing> PartialEq for Type<'typing> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'resolver> Resolver<'resolver> {
    pub fn fresh(&mut self) -> Type<'resolver> {
        let identity = self.variables.len();
        self.variables.push(None);
        Type::new(identity, TypeKind::Variable(identity))
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
            TypeKind::Has(_, target) => self.occurs(identity, target),
            TypeKind::And(left, right) | TypeKind::Or(left, right) => {
                self.occurs(identity, left) || self.occurs(identity, right)
            }
            TypeKind::Function(function) => {
                if function.members.iter().any(|item| self.occurs(identity, item)) {
                    return true;
                }
                if let Some(kind) = &function.output {
                    return self.occurs(identity, &*kind);
                }
                false
            }
            _ => false,
        }
    }

    pub fn unify(
        &mut self,
        span: Span,
        left: &Type<'resolver>,
        right: &Type<'resolver>,
    ) -> Type<'resolver> {
        let left = self.reify(left);
        let right = self.reify(right);

        if left == right {
            return left;
        }

        match (left.kind.clone(), right.kind.clone()) {
            (TypeKind::Unknown, _) => right.clone(),
            (_, TypeKind::Unknown) => left.clone(),
            (TypeKind::Any, _) => right.clone(),
            (_, TypeKind::Any) => left.clone(),

            (TypeKind::Variable(identity), _) => {
                if self.occurs(identity, &right) {
                    self.errors.push(ResolveError::new(
                        ErrorKind::Mismatch(left.clone(), right.clone()),
                        span,
                    ));
                    return left;
                }
                self.variables[identity] = Some(right.clone());
                right
            }
            (_, TypeKind::Variable(identity)) => {
                if self.occurs(identity, &left) {
                    self.errors.push(ResolveError::new(
                        ErrorKind::Mismatch(left.clone(), right.clone()),
                        span,
                    ));
                    return left;
                }
                self.variables[identity] = Some(left.clone());
                left
            }

            (TypeKind::Integer { size: 8, .. }, TypeKind::Character) => right.clone(),
            (TypeKind::Character, TypeKind::Integer { size: 8, .. }) => left.clone(),

            (
                TypeKind::Integer { size: left_size, signed: left_signed },
                TypeKind::Integer { size: right_size, signed: right_signed },
            ) if left_size == right_size && left_signed == right_signed => left,

            (TypeKind::Float { size: left_size }, TypeKind::Float { size: right_size })
            if left_size == right_size => left,

            (
                TypeKind::Array { member: left_item, size: left_size },
                TypeKind::Array { member: right_item, size: right_size },
            ) if left_size == right_size => {
                let unified = self.unify(span, &left_item, &right_item);
                Type::from(TypeKind::Array {
                    member: Box::new(unified),
                    size: left_size,
                })
            }

            (TypeKind::Pointer { target: left_target }, TypeKind::Pointer { target: right_target }) => {
                let unified = self.unify(span, &left_target, &right_target);
                Type::from(TypeKind::Pointer { target: Box::new(unified) })
            }

            (TypeKind::Tuple { members: left_items }, TypeKind::Tuple { members: right_items })
            if left_items.len() == right_items.len() => {
                let mut unified = Vec::with_capacity(left_items.len());
                for (first, second) in left_items.iter().zip(right_items.iter()) {
                    unified.push(self.unify(span, first, second));
                }
                Type::from(TypeKind::Tuple { members: Box::new(unified) })
            }

            (TypeKind::Structure(_), TypeKind::Structure(_))
            | (TypeKind::Union(_), TypeKind::Union(_))
            | (TypeKind::Module(_), TypeKind::Module(_))
            if left.identity == right.identity => left,

            (TypeKind::Has(name, target), TypeKind::Structure(aggr)) |
            (TypeKind::Has(name, target), TypeKind::Union(aggr)) => {
                let mut found = false;
                for member in &aggr.members {
                    if let TypeKind::Has(member_name, member_target) = &member.kind {
                        if member_name == &name {
                            self.unify(span, &target, member_target);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    self.errors.push(ResolveError::new(
                        ErrorKind::Mismatch(left.clone(), right.clone()),
                        span,
                    ));
                }
                right.clone()
            }

            (TypeKind::Structure(aggr), TypeKind::Has(name, target)) |
            (TypeKind::Union(aggr), TypeKind::Has(name, target)) => {
                let mut found = false;
                for member in &aggr.members {
                    if let TypeKind::Has(member_name, member_target) = &member.kind {
                        if member_name == &name {
                            self.unify(span, member_target, &target);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    self.errors.push(ResolveError::new(
                        ErrorKind::Mismatch(left.clone(), right.clone()),
                        span,
                    ));
                }
                left.clone()
            }

            (TypeKind::Has(left_name, left_target), TypeKind::Has(right_name, right_target)) => {
                if left_name == right_name {
                    let unified = self.unify(span, &left_target, &right_target);
                    Type::from(TypeKind::Has(left_name, Box::new(unified)))
                } else {
                    Type::from(TypeKind::And(Box::new(left.clone()), Box::new(right.clone())))
                }
            }

            (TypeKind::And(left_a, left_b), TypeKind::And(right_a, right_b)) => {
                let a = self.unify(span, &left_a, &right_a);
                let b = self.unify(span, &left_b, &right_b);
                Type::from(TypeKind::And(Box::new(a), Box::new(b)))
            }

            (TypeKind::And(a, b), _) => {
                let first = self.unify(span, &a, &right);
                self.unify(span, &b, &first)
            }

            (_, TypeKind::And(a, b)) => {
                let first = self.unify(span, &left, &a);
                self.unify(span, &first, &b)
            }

            (TypeKind::Or(left_a, left_b), TypeKind::Or(right_a, right_b)) => {
                let a = self.unify(span, &left_a, &right_a);
                let b = self.unify(span, &left_b, &right_b);
                Type::from(TypeKind::Or(Box::new(a), Box::new(b)))
            }

            (TypeKind::Function(left_func), TypeKind::Function(right_func))
            if left_func.members.len() == right_func.members.len() => {
                let mut unified = Vec::with_capacity(left_func.members.len());
                for (first, second) in left_func.members.iter().zip(right_func.members.iter()) {
                    unified.push(self.unify(span, first, second));
                }

                let output = match (left_func.output, right_func.output) {
                    (Some(first), Some(second)) => Some(Box::new(self.unify(span, &first, &second))),
                    (Some(first), None) => Some(first.clone()),
                    (None, Some(second)) => Some(second.clone()),
                    (None, None) => None,
                };

                let name = if left_func.target.is_empty() {
                    right_func.target.clone()
                } else {
                    left_func.target.clone()
                };

                let body = self.unify(span, &left_func.body, &right_func.body);

                Type::new(left.identity, TypeKind::Function(Box::new(Function::new(
                    name,
                    unified,
                    body,
                    output,
                    left_func.interface,
                    left_func.entry,
                    left_func.variadic
                ))))
            }

            _ => {
                self.errors.push(ResolveError::new(
                    ErrorKind::Mismatch(left.clone(), right.clone()),
                    span,
                ));
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
            TypeKind::Pointer { target } => Type::from(TypeKind::Pointer {
                target: Box::new(self.reify(target)),
            }),
            TypeKind::Array { member, size } => Type::from(TypeKind::Array {
                member: Box::new(self.reify(member)),
                size: *size,
            }),
            TypeKind::Tuple { members } => {
                let items = members.iter().map(|item| self.reify(item)).collect();
                Type::from(TypeKind::Tuple { members: Box::new(items) })
            }
            TypeKind::Has(name, target) => {
                Type::from(TypeKind::Has(name.clone(), Box::new(self.reify(target))))
            }
            TypeKind::And(left, right) => {
                Type::from(TypeKind::And(Box::new(self.reify(left)), Box::new(self.reify(right))))
            }
            TypeKind::Or(left, right) => {
                Type::from(TypeKind::Or(Box::new(self.reify(left)), Box::new(self.reify(right))))
            }
            TypeKind::Function(function) => {
                let members = function.members.iter().map(|item| self.reify(item)).collect();
                let returnable = function.output.as_ref().map(|kind| Box::new(self.reify(kind)));
                let body = self.reify(&function.body);
                Type::new(
                    typing.identity,
                    TypeKind::Function(Box::new(Function::new(function.target.clone(), members, body, returnable, Interface::Axo, false, false))),
                )
            }
            _ => typing.clone(),
        }
    }

    pub fn evaluate(&self, element: &Element<'resolver>) -> Result<Scale, ResolveError<'resolver>> {
        match &element.kind {
            ElementKind::Literal(token) => match &token.kind {
                TokenKind::Integer(value) => Ok(*value as Scale),
                _ => Err(ResolveError::new(
                    ErrorKind::InvalidAnnotation(element.clone()),
                    element.span,
                )),
            },
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
                        _ => Err(ResolveError::new(
                            ErrorKind::InvalidAnnotation(element.clone()),
                            element.span,
                        )),
                    }
                } else {
                    Err(ResolveError::new(
                        ErrorKind::InvalidAnnotation(element.clone()),
                        element.span,
                    ))
                }
            }
            _ => Err(ResolveError::new(
                ErrorKind::InvalidAnnotation(element.clone()),
                element.span,
            )),
        }
    }

    pub fn annotation(
        &mut self,
        element: &Element<'resolver>,
    ) -> Result<Type<'resolver>, ResolveError<'resolver>> {
        match &element.kind {
            ElementKind::Literal(token) => match &token.kind {
                TokenKind::Identifier(name) => {
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
                        "Void" => TypeKind::Void,
                        "Any" => TypeKind::Any,
                        "Type" => TypeKind::Type,
                        _ => {
                            return if let Ok(symbol) = self.lookup(element) {
                                Ok(symbol.typing)
                            } else {
                                Err(ResolveError::new(
                                    ErrorKind::InvalidAnnotation(element.clone()),
                                    token.span,
                                ))
                            }
                        }
                    };
                    Ok(Type::from(kind))
                }
                _ => Err(ResolveError::new(
                    ErrorKind::InvalidAnnotation(element.clone()),
                    token.span,
                )),
            },

            ElementKind::Binary(binary) => {
                let left = self.annotation(&binary.left)?;
                let right = self.annotation(&binary.right)?;
                if let TokenKind::Operator(operator) = &binary.operator.kind {
                    match operator.as_slice() {
                        [OperatorKind::Ampersand] => Ok(Type::from(TypeKind::And(Box::new(left), Box::new(right)))),
                        [OperatorKind::Pipe] => Ok(Type::from(TypeKind::Or(Box::new(left), Box::new(right)))),
                        _ => Err(ResolveError::new(
                            ErrorKind::InvalidAnnotation(element.clone()),
                            element.span,
                        )),
                    }
                } else {
                    Err(ResolveError::new(
                        ErrorKind::InvalidAnnotation(element.clone()),
                        element.span,
                    ))
                }
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
                        return Err(ResolveError::new(
                            ErrorKind::InvalidAnnotation(element.clone()),
                            element.span,
                        ));
                    }
                    let member = self.annotation(&delimited.members[0])?;
                    let size = self.evaluate(&delimited.members[1])?;
                    Ok(Type::from(TypeKind::Array { member: Box::new(member), size }))
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    _,
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.members.is_empty() {
                        Ok(Type::from(TypeKind::Tuple { members: Box::new(Vec::new()) }))
                    } else if delimited.separator.is_none() && delimited.members.len() == 1 {
                        self.annotation(&delimited.members[0])
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());
                        for member in &delimited.members {
                            members.push(self.annotation(member)?);
                        }
                        Ok(Type::from(TypeKind::Tuple { members: Box::new(members) }))
                    }
                }

                _ => Err(ResolveError::new(
                    ErrorKind::InvalidAnnotation(element.clone()),
                    element.span,
                )),
            },

            ElementKind::Unary(unary) => {
                if matches!(&unary.operator.kind, TokenKind::Operator(operator) if **operator == OperatorKind::Star) {
                    let item = self.annotation(&unary.operand)?;
                    Ok(Type::new(
                        item.identity,
                        TypeKind::Pointer { target: Box::from(item) },
                    ))
                } else {
                    Err(ResolveError::new(
                        ErrorKind::InvalidAnnotation(element.clone()),
                        element.span,
                    ))
                }
            }

            _ => Err(ResolveError::new(
                ErrorKind::InvalidAnnotation(element.clone()),
                element.span,
            )),
        }
    }
}
