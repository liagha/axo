use crate::{
    data::{Boolean, Identity, Scale, Str, Structure},
    parser::{Element, ElementKind},
    resolver::{ResolveError, ErrorKind, Resolver},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
};

#[derive(Clone, Debug)]
pub struct Type<'source> {
    pub kind: TypeKind<'source>,
    pub span: Span<'source>,
}

impl<'source> Type<'source> {
    pub fn new(kind: TypeKind<'source>, span: Span<'source>) -> Self {
        Self { kind, span }
    }

    pub fn unit(span: Span<'source>) -> Self {
        Self::new(TypeKind::Tuple { members: Vec::new() }, span)
    }

    pub fn annotation(resolver: &mut Resolver<'source>, element: &Element<'source>) -> Result<Type<'source>, ResolveError<'source>> {
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
                            return Ok(resolver.lookup(identity, *span));
                        }
                        return Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), *span));
                    }
                };

                Ok(Self::new(kind, *span))
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

                    let member = Type::annotation(resolver, &delimited.members[0])?;
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
                        members.push(Type::annotation(resolver, member)?);
                    }
                    Ok(Type::new(TypeKind::Tuple { members }, element.span))
                }

                _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
            },

            ElementKind::Unary(unary) => {
                if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
                    let item = Type::annotation(resolver, &unary.operand)?;
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
                                    parameters.push(Type::annotation(resolver, member)?);
                                }
                            }
                            _ => {
                                parameters.push(Type::annotation(resolver, &binary.left)?);
                            }
                        }

                        let output = Type::annotation(resolver, &binary.right)?;

                        Ok(Type::new(TypeKind::Function(Str::default(), parameters, Some(Box::new(output))), element.span))
                    }
                    _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
                }
            }

            _ => Err(ResolveError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind<'source> {
    Integer { size: Scale, signed: Boolean },
    Float { size: Scale },
    Boolean,
    String,
    Character,
    Pointer { target: Box<Type<'source>> },
    Array { member: Box<Type<'source>>, size: Scale },
    Tuple { members: Vec<Type<'source>> },
    Void,
    Variable(Identity),
    Unknown,
    Constructor(Identity, Structure<Str<'source>, Type<'source>>),
    Structure(Identity, Structure<Str<'source>, Type<'source>>),
    Union(Identity, Structure<Str<'source>, Type<'source>>),
    Function(Str<'source>, Vec<Type<'source>>, Option<Box<Type<'source>>>),
}

impl<'source> PartialEq for Type<'source> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
