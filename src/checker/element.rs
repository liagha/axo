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
                                .map(|member| member.ty.clone())
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

                        let last = delimited.members.len() - 1;

                        for (index, member) in delimited.members.iter_mut().enumerate() {
                            let errors = member.check();

                            if index == last {
                                if errors.is_empty() {
                                    return errors;
                                } else {
                                    ty = member.ty.clone();
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
                            let inner = delimited.members[0].ty.clone();

                            for member in delimited.members.iter_mut().skip(1) {
                                errors.extend(member.check());

                                if inner != member.ty {
                                    errors.push(
                                        CheckError::new(
                                            ErrorKind::Mismatch(
                                                inner.clone(),
                                                member.ty.clone(),
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
                                    unary.operator.clone()
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
                                            unary.operand.ty.clone(),
                                        ),
                                        self.span
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Tilde] => {
                        if unary.operand.ty.is_integer() {
                            Ok(unary.operand.ty.clone())
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::integer(64, true, self.span),
                                            unary.operand.ty.clone(),
                                        ),
                                        self.span,
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Plus] | [OperatorKind::Minus] => {
                        if unary.operand.ty.is_numeric() {
                            Ok(unary.operand.ty.clone())
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::integer(64, true, self.span),
                                            unary.operand.ty.clone(),
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
                            Ok(Type::pointer(unary.operand.ty.clone(), self.span))
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::InvalidOperation(
                                            unary.operator.clone()
                                        ),
                                        unary.operator.span,
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Star] => match unary.operand.ty.clone().kind {
                        TypeKind::Pointer { target: to } => Ok(*to),
                        TypeKind::Void => Ok(Type::new(TypeKind::Void, self.span)),
                        _ => {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::pointer(Type::new(TypeKind::Void, self.span), self.span),
                                            unary.operand.ty.clone(),
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
                                        unary.operator.clone()
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
                                    binary.operator.clone()
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
                            Ok(binary.left.ty.clone())
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            binary.left.ty.clone(),
                                            binary.right.ty.clone()
                                        ),
                                        binary.operator.span
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Plus] => {
                        if binary.left.ty.is_infer() && binary.right.ty.is_numeric() {
                            Ok(binary.right.ty.clone())
                        } else if binary.right.ty.is_infer() && binary.left.ty.is_numeric() {
                            Ok(binary.left.ty.clone())
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
                                    Ok(binary.right.ty.clone())
                                }
                                _ => {
                                    Err(
                                        vec![
                                            CheckError::new(
                                                ErrorKind::Mismatch(
                                                    binary.left.ty.clone(),
                                                    binary.right.ty.clone()
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
                                Ok(Type::integer(64, true, binary.operator.span))
                           } else {
                                Err(
                                    vec![
                                        CheckError::new(
                                            ErrorKind::Mismatch(
                                                binary.left.ty.clone(),
                                                binary.right.ty.clone()
                                            ),
                                            binary.operator.span
                                        )
                                    ]
                                )
                           }
                        } else if binary.left.ty.is_pointer() {
                            if binary.right.ty.is_integer() {
                                binary.left.ty.span = binary.operator.span;
                                Ok(binary.left.ty.clone())
                            } else {
                                Err(
                                    vec![
                                        CheckError::new(
                                            ErrorKind::InvalidOperation(
                                                binary.operator.clone()
                                            ),
                                            binary.operator.span,
                                        )
                                    ]
                                )
                            }
                        } else if binary.right.ty.is_pointer() {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::InvalidOperation(
                                            binary.operator.clone()
                                        ),
                                        binary.operator.span,
                                    )
                                ]
                            )
                        } else if binary.left.ty.is_infer() && binary.right.ty.is_numeric() {
                            Ok(binary.right.ty.clone())
                        } else if binary.right.ty.is_infer() && binary.left.ty.is_numeric() {
                            Ok(binary.left.ty.clone())
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
                            _ => {
                                Err(
                                    vec![
                                        CheckError::new(
                                            ErrorKind::Mismatch(
                                                binary.left.ty.clone(),
                                                binary.right.ty.clone()
                                            ),
                                            binary.operator.span
                                        )
                                    ]
                                )
                            },
                        }
                    }
                    }
                    [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                        if binary.left.ty.is_infer() && binary.right.ty.is_numeric() {
                            self.ty = binary.right.ty.clone();
                            return Vec::new();
                        }
                        if binary.right.ty.is_infer() && binary.left.ty.is_numeric() {
                            self.ty = binary.left.ty.clone();
                            return Vec::new();
                        }
                        if binary.left.ty.is_infer() && binary.right.ty.is_infer() {
                            self.ty = Type::new(TypeKind::Void, binary.operator.span);
                            return Vec::new();
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
                                    vec![
                                        CheckError::new(
                                            ErrorKind::Mismatch(
                                                binary.left.ty.clone(),
                                                binary.right.ty.clone()
                                            ),
                                            binary.operator.span
                                        )
                                    ]
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
                            Ok(binary.left.ty.clone())
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::integer(64, true, self.span),
                                            binary.right.ty.clone()
                                        ),
                                        self.span,
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Ampersand, OperatorKind::Ampersand]
                    | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                        if binary.left.ty.is_boolean() && binary.right.ty.is_boolean() {
                            Ok(Type::boolean(binary.operator.span))
                        } else {
                            Err(
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            Type::boolean(self.span),
                                            binary.right.ty.clone()
                                        ),
                                        self.span
                                    )
                                ]
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
                                vec![
                                    CheckError::new(
                                        ErrorKind::Mismatch(
                                            binary.left.ty.clone(),
                                            binary.right.ty.clone()
                                        ),
                                        binary.operator.span
                                    )
                                ]
                            )
                        }
                    }
                    [OperatorKind::Dot] => Ok(binary.right.ty.clone()),
                    _ => Err(
                        vec![
                            CheckError::new(
                                ErrorKind::InvalidOperation(
                                    binary.operator.clone()
                                ),
                                binary.operator.span,
                            )
                        ]
                    ),
                }
            }
           ElementKind::Index(index) => {
               if index.members.is_empty() {
                    self.ty = Type::unit(self.span);
                    return Vec::new();
               }

                let mut errors = index.target.check();
                errors.extend(index.members[0].check());

                if !errors.is_empty() {
                    return errors;
                }

                let target_ty = index.target.ty.clone();
                let index_ty = index.members[0].ty.clone();

                if !index_ty.is_integer() {
                    return vec![
                        CheckError::new(
                            ErrorKind::Mismatch(
                                Type::integer(64, true, self.span),
                                index_ty,
                            ),
                            self.span,
                        )
                    ];
                }

                match target_ty.kind {
                    TypeKind::Array { member, .. } => Ok(*member),
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token {
                            kind: TokenKind::Integer(value),
                            ..
                        }) = &index.members[0].kind
                        {
                            if let Ok(idx) = usize::try_from(*value) {
                                if idx < members.len() {
                                    self.ty = members[idx].clone();
                                    return Vec::new();
                                }
                            }
                        }

                        let token = Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        );
                        Err(
                            vec![
                                CheckError::new(
                                    ErrorKind::InvalidOperation(token),
                                    self.span,
                                )
                            ]
                        )
                    }
                    _ => {
                        let token = Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        );
                        Err(
                            vec![
                                CheckError::new(
                                    ErrorKind::InvalidOperation(token),
                                    self.span,
                                )
                            ]
                        )
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
                        let mut errors = invoke.members[0].check();

                        errors.extend(invoke.members[1].check());
                        errors.extend(invoke.members[2].check());

                        if !invoke.members[0].ty.is_boolean() {
                            errors.push(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type::boolean(self.span),
                                        invoke.members[0].ty.clone(),
                                    ),
                                    invoke.members[0].span,
                                )
                            )
                        }

                        if let Some(unified) = unify(&invoke.members[1].ty, &invoke.members[2].ty) {
                            Ok(unified)
                        } else {
                            errors.push(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        invoke.members[1].ty.clone(),
                                        invoke.members[2].ty.clone(),
                                    ),
                                    self.span
                                )
                            );

                            if errors.is_empty() {
                                Ok(Type::unit(self.span))
                            } else {
                                Err(errors)
                            }
                        }
                    }
                    Some("while") => {
                        let mut errors = invoke.members[0].check();

                        errors.extend(invoke.members[1].check());

                        if !invoke.members[0].ty.is_boolean() {
                            errors.push(
                                CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type::boolean(self.span),
                                        invoke.members[0].ty.clone(),
                                    ),
                                    invoke.members[0].span,
                                )
                            )
                        }

                        Ok(Type::unit(self.span))
                    }
                    _ => Ok(Type::new(TypeKind::Void, self.span)),
                }
            }
            ElementKind::Construct(construct) => {
                let members: Result<Vec<Type<'element>>, Vec<CheckError<'element>>> =
                    construct
                        .members
                        .iter_mut()
                        .map(|field| {
                            let errors = field.check();

                            if errors.is_empty() {
                                Ok(field.ty.clone())
                            } else {
                                Err(errors)
                            }
                        })
                        .collect();

                match members {
                    Ok(members) => {
                        let structure = Structure::new(
                            Str::from(construct.target.brand().unwrap().format(0)),
                            members,
                        );

                        Ok(Type::new(TypeKind::Structure(structure), self.span))
                    }

                    Err(errors) => {
                        Err(errors)
                    }
                }
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
