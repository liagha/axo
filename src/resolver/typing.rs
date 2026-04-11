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
                TypeKind::Function(left_func),
                TypeKind::Function(right_func),
            ) if left_func.members.len() == right_func.members.len() => {
                let mut unified = Vec::with_capacity(left_func.members.len());

                for (first, second) in left_func.members.iter().zip(right_func.members.iter()) {
                    unified.push(self.unify(span, first, second));
                }

                let output = match (left_func.output, right_func.output) {
                    (Some(first), Some(second)) => {
                        Some(Box::new(self.unify(span, &first, &second)))
                    }
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
            TypeKind::Function(func) => {
                let members = func.members.iter().map(|item| self.reify(item)).collect();
                let returnable = func.output.as_ref().map(|kind| Box::new(self.reify(kind)));
                let body = self.reify(&func.body);

                Type::new(
                    typing.identity,
                    TypeKind::Function(Box::new(Function::new(func.target, members, body, returnable, Interface::Axo, false, false))),
                )
            }
            _ => typing.clone(),
        }
    }

    pub fn evaluate(&self, element: &Element<'resolver>) -> Result<Scale, ResolveError<'resolver>> {
        match &element.kind {
            ElementKind::Literal(token) => match &token.kind {
                TokenKind::Integer(value) => Ok(**value as Scale),
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
                if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
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
