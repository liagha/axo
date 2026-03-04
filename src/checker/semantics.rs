use {
    crate::{
        data::Scale,
        parser::{Element, ElementKind},
        scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        checker::{Type, TypeKind},
    },
};

pub fn unify<'symbol>(expected: &Type<'symbol>, actual: &Type<'symbol>) -> Option<Type<'symbol>> {
    match (&expected.kind, &actual.kind) {
        (TypeKind::Infer, _) => Some(actual.clone()),
        (_, TypeKind::Infer) => Some(expected.clone()),
        (
            TypeKind::Pointer { to: expected_to },
            TypeKind::Pointer { to: actual_to },
        ) => {
            let unified = unify(expected_to, actual_to)?;
            Some(Type::pointer(unified, expected.span))
        }
        (
            TypeKind::Array {
                member: expected_member,
                size: expected_size,
            },
            TypeKind::Array {
                member: actual_member,
                size: actual_size,
            },
        ) if expected_size == actual_size => {
            let unified = unify(expected_member, actual_member)?;
            Some(Type::new(
                TypeKind::Array {
                    member: Box::new(unified),
                    size: *expected_size,
                },
                expected.span,
            ))
        }
        (TypeKind::Tuple { members: expected_members }, TypeKind::Tuple { members: actual_members })
            if expected_members.len() == actual_members.len() =>
        {
            let mut unified = Vec::with_capacity(expected_members.len());
            for (expected_member, actual_member) in expected_members.iter().zip(actual_members.iter()) {
                unified.push(unify(expected_member, actual_member)?);
            }
            Some(Type::new(TypeKind::Tuple { members: unified }, expected.span))
        }
        (TypeKind::Type(expected_inner), TypeKind::Type(actual_inner)) => {
            let unified = unify(expected_inner, actual_inner)?;
            Some(Type::new(TypeKind::Type(Box::new(unified)), expected.span))
        }
        (
            TypeKind::Integer {
                bits: expected_bits,
                signed: expected_signed,
            },
            TypeKind::Integer {
                bits: actual_bits,
                signed: actual_signed,
            },
        ) => Some(Type::integer(
            (*expected_bits).max(*actual_bits),
            *expected_signed || *actual_signed,
            expected.span,
        )),
        (TypeKind::Float { bits: expected_bits }, TypeKind::Float { bits: actual_bits }) => {
            Some(Type::float((*expected_bits).max(*actual_bits), expected.span))
        }
        (TypeKind::Float { bits }, TypeKind::Integer { .. })
        | (TypeKind::Integer { .. }, TypeKind::Float { bits }) => Some(Type::float(*bits, expected.span)),
        _ if expected == actual => Some(expected.clone()),
        _ => None,
    }
}

pub fn annotation<'symbol>(element: &Element<'symbol>) -> Option<Type<'symbol>> {
    match &element.kind {
        ElementKind::Literal(Token {
            kind: TokenKind::Identifier(name),
            span,
        }) => {
            let name = name.as_str()?;
            match TypeKind::from_name(name) {
                Some(TypeKind::Integer { bits, signed }) => Some(Type::integer(bits, signed, *span)),
                Some(TypeKind::Float { bits }) => Some(Type::float(bits, *span)),
                Some(TypeKind::Boolean) => Some(Type::boolean(*span)),
                Some(TypeKind::Char) => Some(Type::character(*span)),
                Some(_) | None => match name {
                    "String" => Some(Type::string(*span)),
                    "Infer" => Some(Type::new(TypeKind::Infer, *span)),
                    "Type" => Some(Type::new(
                        TypeKind::Type(Box::new(Type::new(TypeKind::Infer, *span))),
                        *span,
                    )),
                    _ => Some(Type::new(TypeKind::Infer, *span)),
                },
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
                    return None;
                }
                let member = annotation(&delimited.members[0])?;
                let size = match delimited.members[1].kind {
                    ElementKind::Literal(Token {
                        kind: TokenKind::Integer(value),
                        ..
                    }) => value as Scale,
                    _ => return None,
                };
                Some(Type::new(
                    TypeKind::Array {
                        member: Box::new(member),
                        size,
                    },
                    element.span,
                ))
            }
            (
                TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                TokenKind::Punctuation(PunctuationKind::RightParenthesis),
            ) => {
                let members: Option<Vec<Type<'symbol>>> =
                    delimited.members.iter().map(annotation).collect();
                Some(Type::new(
                    TypeKind::Tuple { members: members? },
                    element.span,
                ))
            }
            _ => None,
        },
        ElementKind::Unary(unary) => {
            if matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star)) {
                let item = annotation(&unary.operand)?;
                Some(Type::pointer(item, element.span))
            } else {
                None
            }
        }
        _ => None,
    }
}