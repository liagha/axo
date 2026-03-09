use crate::{
    data::*,
    checker::{unify, CheckError, Checkable, ErrorKind, Type, TypeKind},
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
    format::Show,
};

impl<'element> Checkable<'element> for Element<'element> {
    fn check(&mut self) -> Vec<CheckError<'element>> {
        let result = match &mut self.kind.clone() {
            ElementKind::Literal(literal) => {
                let ty = match literal.kind {
                    TokenKind::Integer(_) => Type::integer(64, true, literal.span),
                    TokenKind::Float(_) => Type::float(64, literal.span),
                    TokenKind::Boolean(_) => Type::boolean(literal.span),
                    TokenKind::String(_) => Type::string(literal.span),
                    TokenKind::Character(_) => Type::character(literal.span),
                    _ => Type::new(TypeKind::Void, literal.span),
                };
                
                Ok(ty)
            },

            ElementKind::Delimited(delimited) => {
                match (
                    &delimited.start.kind,
                    delimited.separator.as_ref().map(|token| &token.kind),
                    &delimited.end.kind,
                ) {
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    )
                    | (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        if delimited.separator.is_none() && delimited.members.len() == 1 {
                            return delimited.members[0].check();
                        }

                        let errors : Vec<CheckError<'element>> = delimited
                            .members
                            .iter_mut()
                            .map(|member| member.check())
                            .flatten()
                            .collect();

                        if errors.is_empty() {
                            let members = delimited
                                .members
                                .iter()
                                .map(|member| member.ty)
                                .collect::<Vec<_>>();

                            Ok(
                                Type::new(
                                    TypeKind::Tuple { members },
                                    Span::void(),
                                )
                            )
                        } else {
                            Err(errors)
                        }
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    )
                    | (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) => {
                        let mut ty = Type::unit(Span::void());
                        
                        for (index, member) in delimited.members.iter_mut().enumerate() {
                            if index == delimited.members.len() - 1 {
                                let errors = member.check();

                                if errors.is_empty() {
                                    return errors
                                } else {
                                    ty = member.ty
                                }
                            }
                        }
                        
                        Ok(ty)
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                    ) |
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightBracket),
                    ) => {
                        if delimited.members.is_empty() {
                            Ok(
                                Type::new(
                                    TypeKind::Array {
                                        member: Box::new(Type::new(TypeKind::Void, Span::void())),
                                        size: 0,
                                    },
                                    Span::void(),
                                )
                            )
                        } else {
                            let mut errors = delimited.members[0].check();
                            let inner = delimited.members[0].ty;

                            for member in delimited.members.iter_mut().skip(1) {
                                errors.extend(member.check());

                                if inner != member.ty {
                                    errors.push(
                                        CheckError::new(
                                            ErrorKind::Mismatch(
                                                inner,
                                                member.ty,
                                            ),
                                            member.span,
                                        )
                                    );
                                }
                            }

                            Ok(
                                Type::new(
                                    TypeKind::Array {
                                        member: Box::new(inner),
                                        size: delimited.members.len() as Scale,
                                    },
                                    Span::void(),
                                )
                            )
                        }
                    }

                    _ => Ok(Type::unit(Span::void())),
                }
            },

            ElementKind::Unary(unary) => {
                let mut errors = unary.operand.check();
                
                if !errors.is_empty() {
                    return errors;
                }

                let operator = match unary.operator.kind.clone() {
                    TokenKind::Operator(operator) => operator,
                    _ => {
                        errors.push(
                            CheckError::new(
                                ErrorKind::InvalidOperation(
                                    unary.operator
                                ),
                                unary.operator.span,
                            )
                        );
                        
                        return errors;
                    },
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => {
                        if unary.operand.ty.is_boolean() {
                            Ok(Type::boolean(self.span))
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::boolean(self.span),
                                            unary.operand.ty,
                                        ),
                                        self.span
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Tilde] => {
                        if unary.operand.ty.is_integer() {
                            Ok(unary.operand.ty)
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::integer(64, true, self.span),
                                            unary.operand.ty,
                                        ),
                                        self.span,
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Plus] | [OperatorKind::Minus] => {
                        if unary.operand.ty.is_numeric() {
                            Ok(unary.operand.ty)
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::integer(64, true, self.span),
                                            unary.operand.ty,
                                        ),
                                        self.span,
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Ampersand] => {
                        let addressable = match &unary.operand.kind {
                            ElementKind::Literal(Token {
                                                     kind: TokenKind::Identifier(_),
                                                     ..
                                                 }) => true,
                            ElementKind::Index(_) => true,
                            ElementKind::Binary(binary) => {
                                matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot))
                            }
                            ElementKind::Unary(unary) => {
                                matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star))
                            }
                            _ => false,
                        };

                        if addressable {
                            Ok(Type::pointer(unary.operand.ty, self.span))
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::InvalidOperation(
                                            unary.operator
                                        ),
                                        unary.operator.span,
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Star] => match unary.operand.ty.kind {
                        TypeKind::Pointer { target: to } => Ok(*to),
                        TypeKind::Void => Ok(Type::new(TypeKind::Void, self.span)),
                        _ => {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::pointer(Type::new(TypeKind::Void, self.span), self.span),
                                            unary.operand.ty,
                                        ),
                                        self.span
                                    )
                                ]
                            )
                        },
                    },
                    _ => {
                        Err(
                            vec![
                                CheckError::new(
                                    ErrorKind::InvalidOperation(
                                        unary.operator
                                    ),
                                    unary.operator.span,
                                )
                            ]
                        )
                    },
                }
            }
            
            ElementKind::Binary(binary) => {
                let mut errors = binary.left.check();
                errors.extend(binary.right.check());

                if !errors.is_empty() {
                    return errors;
                }

                let operator = match binary.operator.kind.clone() {
                    TokenKind::Operator(operator) => operator,
                    _ => {
                        errors.push(
                            CheckError::new(
                                ErrorKind::InvalidOperation(
                                    binary.operator
                                ),
                                binary.operator.span,
                            )
                        );

                        return errors;
                    },
                };

                match operator.as_slice() {
                    [OperatorKind::Equal] => {
                        if unify(&binary.left.ty, &binary.right.ty).is_some() {
                            Ok(binary.left.ty)
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            binary.left.ty,
                                            binary.right.ty
                                        ),
                                        binary.operator.span
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Plus] => {
                        if binary.left.ty.is_infer() && binary.right.ty.is_numeric() {
                            Ok(binary.right.ty)
                        } else if binary.right.ty.is_infer() && binary.left.ty.is_numeric() {
                            Ok(binary.left.ty)
                        } else if binary.left.ty.is_infer() && binary.right.ty.is_infer() {
                            Ok(Type::new(TypeKind::Void, binary.operator.span))
                        } else {
                            match (&binary.left.ty.kind, &binary.right.ty.kind) {
                                (
                                    TypeKind::Integer {
                                        size: left_bits,
                                        signed: left_signed,
                                    },
                                    TypeKind::Integer {
                                        size: right_bits,
                                        signed: right_signed,
                                    },
                                ) => Ok(Type::integer(
                                    (*left_bits).max(*right_bits),
                                    *left_signed || *right_signed,
                                    binary.operator.span,
                                )),
                                (TypeKind::Float { size: left_bits }, TypeKind::Float { size: right_bits }) => {
                                    Ok(Type::float((*left_bits).max(*right_bits), binary.operator.span))
                                }
                                (TypeKind::Float { size: bits }, TypeKind::Integer { .. })
                                | (TypeKind::Integer { .. }, TypeKind::Float { size: bits }) => Ok(Type::float(*bits, binary.operator.span)),
                                (TypeKind::Integer { .. } | TypeKind::Float { .. }, TypeKind::Pointer { .. }) => {
                                    binary.right.ty.span = binary.operator.span;
                                    Ok(binary.right.ty)
                                }
                                _ => {
                                    Err(
                                        vec![
                                            CheckError::new(
                                                ErrorKind::Mismatch(
                                                    binary.left.ty,
                                                    binary.right.ty
                                                ),
                                                binary.operator.span
                                            )
                                        ]
                                    )
                                },
                            }
                        }
                    }
                    [OperatorKind::Minus] => {
                        if binary.left.ty.is_pointer() && binary.right.ty.is_pointer() {
                            if binary.left.ty == binary.right.ty {
                                return Ok(Type::integer(64, true, binary.operator.span));
                            }

                            return Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        binary.left.ty,
                                        binary.right.ty
                                    ),
                                    binary.operator.span
                                )
                            )
                        }

                        if binary.left.ty.is_pointer() {
                            if binary.right.ty.is_integer() {
                                binary.left.ty.span = binary.operator.span;
                                return Ok(binary.left.ty);
                            }
                            return Err(invalid(binary.operator.clone()));
                        }

                        if binary.right.ty.is_pointer() {
                            return Err(invalid(binary.operator.clone()));
                        }

                        if binary.left.ty.is_infer() && binary.right.ty.is_numeric() {
                            return Ok(binary.right.ty);
                        }
                        if binary.right.ty.is_infer() && binary.left.ty.is_numeric() {
                            return Ok(binary.left.ty);
                        }
                        if binary.left.ty.is_infer() && binary.right.ty.is_infer() {
                            return Ok(Type::new(TypeKind::Void, binary.operator.span));
                        }
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (
                                TypeKind::Integer {
                                    size: left_bits,
                                    signed: left_signed,
                                },
                                TypeKind::Integer {
                                    size: right_bits,
                                    signed: right_signed,
                                },
                            ) => Ok(Type::integer(
                                (*left_bits).max(*right_bits),
                                *left_signed || *right_signed,
                                binary.operator.span,
                            )),
                            (TypeKind::Float { size: left_bits }, TypeKind::Float { size: right_bits }) => {
                                Ok(Type::float((*left_bits).max(*right_bits), binary.operator.span))
                            }
                            (TypeKind::Float { size: bits }, TypeKind::Integer { .. })
                            | (TypeKind::Integer { .. }, TypeKind::Float { size: bits }) => Ok(Type::float(*bits, binary.operator.span)),
                            _ => {
                                Err(
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            binary.left.ty,
                                            binary.right.ty
                                        ),
                                        binary.operator.span
                                    )
                                )
                            },
                        }
                    }
                    [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                        if binary.left.ty.is_infer() && binary.right.ty.is_numeric() {
                            return Ok(binary.right.ty);
                        }
                        if binary.right.ty.is_infer() && binary.left.ty.is_numeric() {
                            return Ok(binary.left.ty);
                        }
                        if binary.left.ty.is_infer() && binary.right.ty.is_infer() {
                            return Ok(Type::new(TypeKind::Void, binary.operator.span));
                        }
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (
                                TypeKind::Integer {
                                    size: left_bits,
                                    signed: left_signed,
                                },
                                TypeKind::Integer {
                                    size: right_bits,
                                    signed: right_signed,
                                },
                            ) => Ok(Type::integer(
                                (*left_bits).max(*right_bits),
                                *left_signed || *right_signed,
                                binary.operator.span,
                            )),
                            (TypeKind::Float { size: left_bits }, TypeKind::Float { size: right_bits }) => {
                                Ok(Type::float((*left_bits).max(*right_bits), binary.operator.span))
                            }
                            (TypeKind::Float { size: bits }, TypeKind::Integer { .. })
                            | (TypeKind::Integer { .. }, TypeKind::Float { size: bits }) => Ok(Type::float(*bits, binary.operator.span)),
                            _ => {
                                Err(
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            binary.left.ty,
                                            binary.right.ty
                                        ),
                                        binary.operator.span
                                    )
                                )
                            },
                        }
                    }
                    [OperatorKind::Ampersand]
                    | [OperatorKind::Pipe]
                    | [OperatorKind::Caret]
                    | [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                    | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                        if binary.left.ty.is_integer() && binary.right.ty.is_integer() {
                            Ok(binary.left.ty)
                        } else {
                            Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type::integer(64, true, self.span),
                                        binary.right.ty
                                    ),
                                    self.span,
                                )
                            )
                        }
                    }
                    [OperatorKind::Ampersand, OperatorKind::Ampersand]
                    | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                        if binary.left.ty.is_boolean() && binary.right.ty.is_boolean() {
                            Ok(Type::boolean(binary.operator.span))
                        } else {
                            Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type::boolean(self.span),
                                        binary.right.ty
                                    ),
                                    self.span
                                )
                            )
                        }
                    }
                    [OperatorKind::Equal, OperatorKind::Equal]
                    | [OperatorKind::Exclamation, OperatorKind::Equal]
                    | [OperatorKind::LeftAngle]
                    | [OperatorKind::LeftAngle, OperatorKind::Equal]
                    | [OperatorKind::RightAngle]
                    | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                        if unify(&binary.left.ty, &binary.right.ty).is_some() {
                            Ok(Type::boolean(binary.operator.span))
                        } else {
                            Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        binary.left.ty,
                                        binary.right.ty
                                    ),
                                    binary.operator.span
                                )
                            )
                        }
                    }
                    [OperatorKind::Dot] => Ok(binary.right.ty),
                    _ => Err(invalid(binary.operator)),
                }
            }
            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    return Ok(Type::unit(self.span));
                }

                let target = index.target.check()?;
                let ty = index.members[0].check()?;

                if !ty.is_integer() {
                    return Err(
                        CheckError::new(
                            ErrorKind::Mismatch(
                                Type::integer(64, true, self.span),
                                ty,
                            ),
                            self.span,
                        )
                    )
                }

                match target.kind {
                    TypeKind::Array { member, .. } => Ok(*member),
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token {
                            kind: TokenKind::Integer(value),
                            ..
                        }) = &index.members[0].kind
                        {
                            if let Ok(idx) = usize::try_from(*value) {
                                if idx < members.len() {
                                    return Ok(members[idx].clone());
                                }
                            }
                        }

                        let token = Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        );
                        Err(invalid(token))
                    }
                    _ => {
                        let token = Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        );
                        Err(invalid(token))
                    }
                }
            }
            ElementKind::Invoke(invoke) => {
                let primitive = invoke.target.brand().and_then(|token| match token.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                })
                    .and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        let condition = invoke.members[0].check()?;

                        if !condition.is_boolean() {
                            return Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type::boolean(self.span),
                                        condition,
                                    ),
                                    invoke.members[0].span,
                                )
                            )
                        }

                        let then = invoke.members[1].check()?;
                        let otherwise = invoke.members[2].check()?;

                        if let Some(unified) = unify(&then, &otherwise) {
                            Ok(unified)
                        } else {
                            Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        then,
                                        otherwise,
                                    ),
                                    self.span
                                )
                            )
                        }
                    }
                    Some("while") => {
                        if invoke.members.len() != 2 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("while")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        let condition = invoke.members[0].check()?;

                        if !condition.is_boolean() {
                            return Err(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type::boolean(self.span),
                                        condition,
                                    ),
                                    invoke.members[0].span,
                                )
                            )
                        }

                        invoke.members[1].check()?;

                        Ok(Type::unit(self.span))
                    }
                    _ => Ok(Type::new(TypeKind::Void, self.span)),
                }
            }
            ElementKind::Construct(construct) => {
                if let Some(target) = construct.target.brand() {
                    if let TokenKind::Identifier(name) = target.kind {
                        if let Some(name) = name.as_str() {
                            match name {
                                "Integer" | "Int32" | "Int64" => {
                                    let mut size = None;
                                    let mut signed = None;

                                    for member in construct.members {
                                        if let ElementKind::Binary(binary) = &member.kind {
                                            if let TokenKind::Operator(operator) = &binary.operator.kind {
                                                if operator.as_slice() != [OperatorKind::Equal] {
                                                    continue;
                                                }
                                            } else {
                                                continue;
                                            }

                                            let field = binary.left.brand().and_then(|token| match token.kind {
                                                TokenKind::Identifier(name) => Some(name),
                                                _ => None,
                                            });

                                            match field.as_ref().and_then(|name| name.as_str()) {
                                                Some("size") => {
                                                    if let ElementKind::Literal(Token {
                                                                                    kind: TokenKind::Integer(value),
                                                                                    ..
                                                                                }) = binary.right.kind
                                                    {
                                                        size = value.try_into().ok();
                                                    }
                                                }
                                                Some("signed") => {
                                                    if let ElementKind::Literal(Token {
                                                                                    kind: TokenKind::Boolean(value),
                                                                                    ..
                                                                                }) = binary.right.kind
                                                    {
                                                        signed = Some(value);
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }

                                    return Ok(
                                        Type::integer(
                                            if name == "Int32" {
                                                32
                                            } else {
                                                size.unwrap_or(64)
                                            },
                                            signed.unwrap_or(true),
                                            self.span,
                                        )
                                    );
                                }

                                "Float" => {
                                    let mut size = 64;

                                    for member in construct.members {
                                        if let ElementKind::Binary(binary) = &member.kind {
                                            if let TokenKind::Operator(operator) = &binary.operator.kind {
                                                if operator.as_slice() != [OperatorKind::Equal] {
                                                    continue;
                                                }
                                            } else {
                                                continue;
                                            }

                                            let field = binary.left.brand().and_then(|token| match token.kind {
                                                TokenKind::Identifier(name) => Some(name),
                                                _ => None,
                                            });

                                            if matches!(field.as_ref().and_then(|name| name.as_str()), Some("size")) {
                                                if let ElementKind::Literal(Token {
                                                                                kind: TokenKind::Integer(value),
                                                                                ..
                                                                            }) = binary.right.kind
                                                {
                                                    size = value;
                                                }
                                            }
                                        }
                                    }

                                    return Ok(Type::float(size as Scale, self.span));
                                }

                                _ => {
                                    let ty =
                                        match TypeKind::from_name(name) {
                                            Some(TypeKind::Integer { size: bits, signed }) => Some(Type::integer(bits, signed, self.span)),
                                            Some(TypeKind::Float { size: bits }) => Some(Type::float(bits, self.span)),
                                            Some(TypeKind::Boolean) => Some(Type::boolean(self.span)),
                                            Some(TypeKind::Character) => Some(Type::character(self.span)),
                                            Some(_) | None => match name {
                                                "String" => Some(Type::string(self.span)),
                                                "Type" => Some(Type::new(
                                                    TypeKind::Type(Box::new(Type::new(TypeKind::Void, self.span))),
                                                    self.span,
                                                )),
                                                _ => None,
                                            },
                                        };

                                    if let Some(kind) = ty {
                                        return Ok(kind);
                                    }
                                }
                            }
                        }
                    }
                }

                let members: Result<Vec<Box<Type<'element>>>, CheckError<'element>> = construct
                    .members
                    .iter()
                    .map(|field| field.check().map(Box::new))
                    .collect();

                let structure = Structure::new(
                    Str::from(construct.target.brand().unwrap().format(0)),
                    members?,
                );

                Ok(Type::new(TypeKind::Structure(structure), self.span))
            }
            ElementKind::Symbolize(_) => Ok(Type::unit(self.span)),
        };
        
        match result { 
            Ok(ty) => {
                self.ty = ty;
                
                Vec::new()
            },
            Err(errors) => errors,
        }
    }
}
