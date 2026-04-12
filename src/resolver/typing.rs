use orbyte::Orbyte;
use crate::{
    data::{Aggregate, Boolean, Identity, Scale, Str},
    parser::{Element, ElementKind},
    resolver::{ErrorKind, ResolveError, Resolver},
    scanner::{OperatorKind, PunctuationKind, TokenKind},
    tracker::Span,
};
use crate::data::{Function, Interface};

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
    Module(Str<'typing>),
    Integer {
        size: Scale,
        signed: Boolean,
    },
    Float {
        size: Scale,
    },
    Boolean,
    String,
    Character,
    Pointer {
        target: Box<Type<'typing>>,
    },
    Array {
        member: Box<Type<'typing>>,
        size: Scale,
    },
    Tuple {
        members: Box<Vec<Type<'typing>>>,
    },
    Void,
    Variable(Identity),
    Unknown,
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
        span: Span<'resolver>,
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

            (TypeKind::Module(_), TypeKind::Module(_)) if left.identity == right.identity => left,
            (
                TypeKind::Array {
                    member: left_item,
                    size: left_size,
                },
                TypeKind::Array {
                    member: right_item,
                    size: right_size,
                },
            ) if left_size == right_size => {
                let unified = self.unify(span, &left_item, &right_item);
                Type::from(TypeKind::Array {
                    member: Box::new(unified),
                    size: left_size,
                })
            }
            (
                TypeKind::Pointer {
                    target: left_target,
                },
                TypeKind::Pointer {
                    target: right_target,
                },
            ) => {
                let unified = self.unify(span, &left_target, &right_target);
                Type::from(TypeKind::Pointer {
                    target: Box::new(unified),
                })
            }
            (
                TypeKind::Tuple {
                    members: left_items,
                },
                TypeKind::Tuple {
                    members: right_items,
                },
            ) if left_items.len() == right_items.len() => {
                let mut unified = Vec::with_capacity(left_items.len());
                for (first, second) in left_items.iter().zip(right_items.iter()) {
                    unified.push(self.unify(span, first, second));
                }
                Type::from(TypeKind::Tuple { members: Box::new(unified) })
            }

            (TypeKind::Structure(_), TypeKind::Structure(_))
            | (TypeKind::Union(_), TypeKind::Union(_))
            if left.identity == right.identity =>
                {
                    left
                }

            (TypeKind::Integer { size: 8, .. }, TypeKind::Character) => right.clone(),

            (TypeKind::Character, TypeKind::Integer { size: 8, .. }) => left.clone(),

            (
                TypeKind::Integer {
                    size: left_size,
                    signed: left_signed,
                },
                TypeKind::Integer {
                    size: right_size,
                    signed: right_signed,
                },
            ) if left_size == right_size && left_signed == right_signed => left,
            (TypeKind::Float { size: left_size }, TypeKind::Float { size: right_size })
            if left_size == right_size =>
                {
                    left
                }

            (
                TypeKind::Function(left_function),
                TypeKind::Function(right_function),
            ) if left_function.members.len() == right_function.members.len() => {
                let mut unified = Vec::with_capacity(left_function.members.len());

                for (first, second) in left_function.members.iter().zip(right_function.members.iter()) {
                    unified.push(self.unify(span, first, second));
                }

                let output = match (left_function.output, right_function.output) {
                    (Some(first), Some(second)) => {
                        Some(Box::new(self.unify(span, &first, &second)))
                    }
                    (Some(first), None) => Some(first.clone()),
                    (None, Some(second)) => Some(second.clone()),
                    (None, None) => None,
                };

                let name = if left_function.target.is_empty() {
                    right_function.target.clone()
                } else {
                    left_function.target.clone()
                };

                let body = self.unify(span, &left_function.body, &right_function.body);

                Type::new(left.identity, TypeKind::Function(Box::new(Function::new(name, unified, body, output, Interface::Axo, false, false))))
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
            TypeKind::Function(function) => {
                let members = function.members.iter().map(|item| self.reify(item)).collect();
                let returnable = function.output.as_ref().map(|kind| Box::new(self.reify(kind)));
                let body = self.reify(&function.body);

                Type::new(
                    typing.identity,
                    TypeKind::Function(Box::new(Function::new(function.target, members, body, returnable, Interface::Axo, false, false))),
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
                        "Int8" => TypeKind::Integer {
                            size: 8,
                            signed: true,
                        },
                        "Int16" => TypeKind::Integer {
                            size: 16,
                            signed: true,
                        },
                        "Int32" => TypeKind::Integer {
                            size: 32,
                            signed: true,
                        },
                        "Int64" | "Integer" => TypeKind::Integer {
                            size: 64,
                            signed: true,
                        },
                        "UInt8" => TypeKind::Integer {
                            size: 8,
                            signed: false,
                        },
                        "UInt16" => TypeKind::Integer {
                            size: 16,
                            signed: false,
                        },
                        "UInt32" => TypeKind::Integer {
                            size: 32,
                            signed: false,
                        },
                        "UInt64" => TypeKind::Integer {
                            size: 64,
                            signed: false,
                        },
                        "Float32" => TypeKind::Float { size: 32 },
                        "Float64" | "Float" => TypeKind::Float { size: 64 },
                        "Boolean" => TypeKind::Boolean,
                        "Character" => TypeKind::Character,
                        "String" => TypeKind::String,
                        "Void" => TypeKind::Void,
                        _ => {
                            return if let Ok(symbol) = self.lookup(element) {
                                Ok(*symbol.typing)
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

                    Ok(Type::from(TypeKind::Array {
                        member: Box::new(member),
                        size,
                    }))
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    _,
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.members.is_empty() {
                        Ok(Type::from(TypeKind::Tuple {
                            members: Box::new(Vec::new()),
                        }))
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
                        TypeKind::Pointer {
                            target: Box::from(item),
                        },
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
