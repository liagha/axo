use crate::{
    data::{Boolean, Scale, Str},
    tracker::Span,
};
use crate::checker::{CheckError, ErrorKind};
use crate::data::*;
use crate::parser::{Element, ElementKind};
use crate::scanner::{OperatorKind, PunctuationKind, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Type<'ty> {
    pub kind: TypeKind<'ty>,
    pub span: Span<'ty>,
}

impl<'ty> Type<'ty> {
    pub fn new(kind: TypeKind<'ty>, span: Span<'ty>) -> Self {
        Self { kind, span }
    }

    pub fn unit(span: Span<'ty>) -> Self {
        Self::new(TypeKind::Tuple { members: Vec::new() }, span)
    }

    pub fn annotation(element: &Element<'ty>) -> Result<Type<'ty>, CheckError<'ty>> {
        match &element.kind {
            ElementKind::Literal(Token { kind: TokenKind::Identifier(name), span }) => {
                let name = name.as_str().unwrap();

                let kind = match name {
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
                    "Bool" | "Boolean" => TypeKind::Boolean,
                    "Char" | "Character" => TypeKind::Character,
                    "String" => TypeKind::String,
                    _ => return Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
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

                    let member = Type::annotation(&delimited.members[0])?;
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
                    let members: Result<Vec<Type<'ty>>, CheckError<'ty>> = delimited.members.iter().map(Type::annotation).collect();
                    Ok(Type::new(TypeKind::Tuple { members: members? }, element.span))
                }

                _ => Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
            },

            ElementKind::Unary(unary) => {
                if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
                    let item = Type::annotation(&unary.operand)?;
                    Ok(Type::new(TypeKind::Pointer { target: Box::from(item) }, element.span))
                } else {
                    Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span))
                }
            }

            _ => Err(CheckError::new(ErrorKind::InvalidAnnotation(element.clone()), element.span)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind<'ty> {
    Integer { size: Scale, signed: Boolean },
    Float { size: Scale },
    Boolean,
    String,
    Character,
    Pointer { target: Box<Type<'ty>> },
    Array { member: Box<Type<'ty>>, size: Scale },
    Tuple { members: Vec<Type<'ty>> },
    Void,
    Variable(Identity),
    Unknown,

    Constructor(Structure<Str<'ty>, Type<'ty>>),
    Structure(Structure<Str<'ty>, Type<'ty>>),
    Union(Structure<Str<'ty>, Type<'ty>>),
    Function(Str<'ty>, Vec<Type<'ty>>, Option<Box<Type<'ty>>>),
}

impl<'ty> PartialEq for Type<'ty> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
