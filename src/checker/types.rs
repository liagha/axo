// src/checker/types.rs
use crate::{
    checker::{CheckError, Checker, ErrorKind},
    data::{Boolean, Identity, Scale, Str, Structure},
    parser::{Element, ElementKind},
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

    pub fn annotation(checker: &mut Checker<'_, 'source>, element: &Element<'source>) -> Result<Type<'source>, CheckError<'source>> {
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
                            return Ok(checker.lookup(identity, *span));
                        }
                        return Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), *span));
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
                        return Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span));
                    }

                    let member = Type::annotation(checker, &delimited.members[0])?;
                    let size = match delimited.members[1].kind {
                        ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) => value as Scale,
                        _ => return Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
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
                        members.push(Type::annotation(checker, member)?);
                    }
                    Ok(Type::new(TypeKind::Tuple { members }, element.span))
                }

                _ => Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
            },

            ElementKind::Unary(unary) => {
                if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
                    let item = Type::annotation(checker, &unary.operand)?;
                    Ok(Type::new(TypeKind::Pointer { target: Box::from(item) }, element.span))
                } else {
                    Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span))
                }
            }

            ElementKind::Binary(binary) => {
                let TokenKind::Operator(operator) = &binary.operator.kind else {
                    return Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span));
                };

                match operator.as_slice() {
                    [OperatorKind::Minus, OperatorKind::RightAngle] => {
                        let mut parameters = Vec::new();

                        match &binary.left.kind {
                            ElementKind::Delimited(delimited) => {
                                for member in &delimited.members {
                                    parameters.push(Type::annotation(checker, member)?);
                                }
                            }
                            _ => {
                                parameters.push(Type::annotation(checker, &binary.left)?);
                            }
                        }

                        let output = Type::annotation(checker, &binary.right)?;

                        Ok(Type::new(TypeKind::Function(Str::default(), parameters, Some(Box::new(output))), element.span))
                    }
                    _ => Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
                }
            }

            _ => Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
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
    Constructor(Structure<Str<'source>, Type<'source>>),
    Structure(Structure<Str<'source>, Type<'source>>),
    Union(Structure<Str<'source>, Type<'source>>),
    Function(Str<'source>, Vec<Type<'source>>, Option<Box<Type<'source>>>),
}

impl<'source> PartialEq for Type<'source> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
