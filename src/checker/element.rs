use crate::{
    data::*,
    checker::{CheckError, Checkable, ErrorKind, Type, TypeKind},
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
    format::Show,
};

impl<'element> Checkable<'element> for Element<'element> {
    fn check(&mut self, errors: &mut Vec<CheckError<'element>>) {
        let span = self.span;

        let ty = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type { kind: TypeKind::Integer { size: 64, signed: true }, span: literal.span },
                TokenKind::Float(_)   => Type { kind: TypeKind::Float { size: 64 }, span: literal.span },
                TokenKind::Boolean(_) => Type { kind: TypeKind::Boolean, span: literal.span },
                TokenKind::String(_)  => Type { kind: TypeKind::String, span: literal.span },
                TokenKind::Character(_) => Type { kind: TypeKind::Character, span: literal.span },
                _ => Type::unit(literal.span),
            },

            ElementKind::Delimited(delimited) => match (
                &delimited.start.kind,
                delimited.separator.as_ref().map(|t| &t.kind),
                &delimited.end.kind,
            ) {
                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.separator.is_none() && delimited.members.len() == 1 {
                        delimited.members[0].check(errors);
                        delimited.members[0].ty.clone()
                    } else {
                        let mut failed = false;

                        for member in delimited.members.iter_mut() {
                            member.check(errors);

                            if member.ty.kind == TypeKind::Unknown { failed = true; }
                        }

                        if failed { return; }

                        let members = delimited.members.iter().map(|m| m.ty.clone()).collect();
                        Type { kind: TypeKind::Tuple { members }, span: Span::void() }
                    }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    let last = delimited.members.len().saturating_sub(1);
                    let mut ty = Type { kind: TypeKind::Tuple { members: Vec::new() }, span: Span::void() };
                    let mut failed = false;

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.check(errors);
                        if member.ty.kind == TypeKind::Unknown { failed = true; }

                        if index == last {
                            ty = member.ty.clone();
                        }
                    }

                    if failed { return; }
                    ty
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    if delimited.members.is_empty() {
                        Type {
                            kind: TypeKind::Array {
                                member: Box::new(Type::new(TypeKind::Tuple { members: Vec::new() }, self.span)),
                                size: 0,
                            },
                            span: Span::void(),
                        }
                    } else {
                        let mut failed = false;

                        for member in delimited.members.iter_mut() {
                            member.check(errors);
                            if member.ty.kind == TypeKind::Unknown { failed = true; }
                        }

                        if failed { return; }

                        let inner = delimited.members[0].ty.clone();
                        for member in delimited.members.iter().skip(1) {
                            if inner != member.ty {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(inner.clone(), member.ty.clone()),
                                    member.span,
                                ));
                                failed = true;
                            }
                        }

                        if failed { return; }

                        Type {
                            kind: TypeKind::Array {
                                member: Box::new(inner),
                                size: delimited.members.len() as Scale,
                            },
                            span: Span::void(),
                        }
                    }
                }

                _ => Type { kind: TypeKind::Tuple { members: Vec::new() }, span: Span::void() },
            },

            ElementKind::Unary(unary) => {
                unary.operand.check(errors);
                if unary.operand.ty.kind == TypeKind::Unknown { return; }

                let TokenKind::Operator(operator) = unary.operator.kind.clone() else {
                    errors.push(CheckError::new(
                        ErrorKind::InvalidOperation(unary.operator.clone()),
                        unary.operator.span,
                    ));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => match unary.operand.ty.kind {
                        TypeKind::Boolean => Type { kind: TypeKind::Boolean, span },
                        _ => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Boolean, span },
                                    unary.operand.ty.clone(),
                                ),
                                span,
                            ));
                            return;
                        }
                    },

                    [OperatorKind::Tilde] => match unary.operand.ty.kind {
                        TypeKind::Integer { .. } => unary.operand.ty.clone(),
                        _ => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Integer { size: 64, signed: true }, span },
                                    unary.operand.ty.clone(),
                                ),
                                span,
                            ));
                            return;
                        }
                    },

                    [OperatorKind::Plus] | [OperatorKind::Minus] => match unary.operand.ty.kind {
                        TypeKind::Integer { .. } | TypeKind::Float { .. } => unary.operand.ty.clone(),
                        _ => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Integer { size: 64, signed: true }, span },
                                    unary.operand.ty.clone(),
                                ),
                                span,
                            ));
                            return;
                        }
                    },

                    [OperatorKind::Ampersand] => {
                        let addressable = match &unary.operand.kind {
                            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => true,
                            ElementKind::Index(_) => true,
                            ElementKind::Binary(binary) => matches!(
                                binary.operator.kind,
                                TokenKind::Operator(OperatorKind::Dot)
                            ),
                            ElementKind::Unary(inner) => matches!(
                                inner.operator.kind,
                                TokenKind::Operator(OperatorKind::Star)
                            ),
                            _ => false,
                        };

                        match addressable {
                            true => Type {
                                kind: TypeKind::Pointer { target: Box::new(unary.operand.ty.clone()) },
                                span,
                            },
                            false => {
                                errors.push(CheckError::new(
                                    ErrorKind::InvalidOperation(unary.operator.clone()),
                                    unary.operator.span,
                                ));
                                return;
                            }
                        }
                    }

                    [OperatorKind::Star] => match unary.operand.ty.clone().kind {
                        TypeKind::Pointer { target } => *target,
                        _ => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(
                                    Type {
                                        kind: TypeKind::Pointer {
                                            target: Box::new(Type::new(TypeKind::Tuple { members: Vec::new() }, span)),
                                        },
                                        span,
                                    },
                                    unary.operand.ty.clone(),
                                ),
                                span,
                            ));
                            return;
                        }
                    },

                    _ => {
                        errors.push(CheckError::new(
                            ErrorKind::InvalidOperation(unary.operator.clone()),
                            unary.operator.span,
                        ));
                        return;
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.check(errors);
                binary.right.check(errors);

                if binary.left.ty.kind == TypeKind::Unknown || binary.right.ty.kind == TypeKind::Unknown {
                    return;
                }

                let TokenKind::Operator(operator) = binary.operator.kind.clone() else {
                    errors.push(CheckError::new(
                        ErrorKind::InvalidOperation(binary.operator.clone()),
                        binary.operator.span,
                    ));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Equal] => match Type::unify(&binary.left.ty, &binary.right.ty) {
                        Some(_) => binary.left.ty.clone(),
                        None => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                binary.operator.span,
                            ));
                            return;
                        }
                    },

                    [OperatorKind::Plus] => match (&binary.left.ty.kind, &binary.right.ty.kind) {
                        (TypeKind::Integer { size: ls, signed: la }, TypeKind::Integer { size: rs, signed: ra }) => Type {
                            kind: TypeKind::Integer { size: (*ls).max(*rs), signed: *la || *ra },
                            span: binary.operator.span,
                        },
                        (TypeKind::Float { size: ls }, TypeKind::Float { size: rs }) => Type {
                            kind: TypeKind::Float { size: (*ls).max(*rs) },
                            span: binary.operator.span,
                        },
                        (TypeKind::Float { size }, TypeKind::Integer { .. })
                        | (TypeKind::Integer { .. }, TypeKind::Float { size }) => Type {
                            kind: TypeKind::Float { size: *size },
                            span: binary.operator.span,
                        },
                        (TypeKind::Integer { .. } | TypeKind::Float { .. }, TypeKind::Pointer { .. }) => {
                            binary.right.ty.span = binary.operator.span;
                            binary.right.ty.clone()
                        }
                        _ => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                binary.operator.span,
                            ));
                            return;
                        }
                    },

                    [OperatorKind::Minus] => match (&binary.left.ty.kind, &binary.right.ty.kind) {
                        (TypeKind::Pointer { .. }, TypeKind::Pointer { .. }) => {
                            if binary.left.ty == binary.right.ty {
                                Type { kind: TypeKind::Integer { size: 64, signed: true }, span: binary.operator.span }
                            } else {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                    binary.operator.span,
                                ));
                                return;
                            }
                        }
                        (TypeKind::Pointer { .. }, TypeKind::Integer { .. }) => {
                            binary.left.ty.span = binary.operator.span;
                            binary.left.ty.clone()
                        }
                        (TypeKind::Pointer { .. }, _) => {
                            errors.push(CheckError::new(
                                ErrorKind::InvalidOperation(binary.operator.clone()),
                                binary.operator.span,
                            ));
                            return;
                        }
                        (_, TypeKind::Pointer { .. }) => {
                            errors.push(CheckError::new(
                                ErrorKind::InvalidOperation(binary.operator.clone()),
                                binary.operator.span,
                            ));
                            return;
                        }
                        (TypeKind::Integer { size: ls, signed: la }, TypeKind::Integer { size: rs, signed: ra }) => Type {
                            kind: TypeKind::Integer { size: (*ls).max(*rs), signed: *la || *ra },
                            span: binary.operator.span,
                        },
                        (TypeKind::Float { size: ls }, TypeKind::Float { size: rs }) => Type {
                            kind: TypeKind::Float { size: (*ls).max(*rs) },
                            span: binary.operator.span,
                        },
                        (TypeKind::Float { size }, TypeKind::Integer { .. })
                        | (TypeKind::Integer { .. }, TypeKind::Float { size }) => Type {
                            kind: TypeKind::Float { size: *size },
                            span: binary.operator.span,
                        },
                        _ => {
                            errors.push(CheckError::new(
                                ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                binary.operator.span,
                            ));
                            return;
                        }
                    },

                    [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (TypeKind::Integer { size: ls, signed: la }, TypeKind::Integer { size: rs, signed: ra }) => Type {
                                kind: TypeKind::Integer { size: (*ls).max(*rs), signed: *la || *ra },
                                span: binary.operator.span,
                            },
                            (TypeKind::Float { size: ls }, TypeKind::Float { size: rs }) => Type {
                                kind: TypeKind::Float { size: (*ls).max(*rs) },
                                span: binary.operator.span,
                            },
                            (TypeKind::Float { size }, TypeKind::Integer { .. })
                            | (TypeKind::Integer { .. }, TypeKind::Float { size }) => Type {
                                kind: TypeKind::Float { size: *size },
                                span: binary.operator.span,
                            },
                            _ => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                    binary.operator.span,
                                ));
                                return;
                            }
                        }
                    }

                    [OperatorKind::Ampersand]
                    | [OperatorKind::Pipe]
                    | [OperatorKind::Caret]
                    | [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                    | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (TypeKind::Integer { .. }, TypeKind::Integer { .. }) => binary.left.ty.clone(),
                            _ => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type { kind: TypeKind::Integer { size: 64, signed: true }, span },
                                        binary.right.ty.clone(),
                                    ),
                                    span,
                                ));
                                return;
                            }
                        }
                    }

                    [OperatorKind::Ampersand, OperatorKind::Ampersand]
                    | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (TypeKind::Boolean, TypeKind::Boolean) => Type { kind: TypeKind::Boolean, span: binary.operator.span },
                            _ => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type { kind: TypeKind::Boolean, span },
                                        binary.right.ty.clone(),
                                    ),
                                    span,
                                ));
                                return;
                            }
                        }
                    }

                    [OperatorKind::Equal, OperatorKind::Equal]
                    | [OperatorKind::Exclamation, OperatorKind::Equal]
                    | [OperatorKind::LeftAngle]
                    | [OperatorKind::LeftAngle, OperatorKind::Equal]
                    | [OperatorKind::RightAngle]
                    | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                        match Type::unify(&binary.left.ty, &binary.right.ty) {
                            Some(_) => Type { kind: TypeKind::Boolean, span: binary.operator.span },
                            None => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                    binary.operator.span,
                                ));
                                return;
                            }
                        }
                    }

                    [OperatorKind::Dot] => binary.right.ty.clone(),

                    _ => {
                        errors.push(CheckError::new(
                            ErrorKind::InvalidOperation(binary.operator.clone()),
                            binary.operator.span,
                        ));
                        return;
                    }
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    self.ty = Type { kind: TypeKind::Tuple { members: Vec::new() }, span };
                    return;
                }

                index.target.check(errors);
                index.members[0].check(errors);

                if index.target.ty.kind == TypeKind::Unknown || index.members[0].ty.kind == TypeKind::Unknown {
                    return;
                }

                let target_ty = index.target.ty.clone();
                let index_ty  = index.members[0].ty.clone();

                match index_ty.kind {
                    TypeKind::Integer { .. } => {}
                    _ => {
                        errors.push(CheckError::new(
                            ErrorKind::Mismatch(
                                Type { kind: TypeKind::Integer { size: 64, signed: true }, span },
                                index_ty,
                            ),
                            span,
                        ));
                        return;
                    }
                }

                match target_ty.kind {
                    TypeKind::Array { member, .. } => *member,
                    TypeKind::Tuple { members } => {
                        match &index.members[0].kind {
                            ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) => {
                                match usize::try_from(*value).ok().filter(|&i| i < members.len()) {
                                    Some(idx) => members[idx].clone(),
                                    None => {
                                        errors.push(CheckError::new(
                                            ErrorKind::InvalidOperation(Token::new(
                                                TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                                span,
                                            )),
                                            span,
                                        ));
                                        return;
                                    }
                                }
                            }
                            _ => {
                                errors.push(CheckError::new(
                                    ErrorKind::InvalidOperation(Token::new(
                                        TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                        span,
                                    )),
                                    span,
                                ));
                                return;
                            }
                        }
                    }
                    _ => {
                        errors.push(CheckError::new(
                            ErrorKind::InvalidOperation(Token::new(
                                TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                span,
                            )),
                            span,
                        ));
                        return;
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                let mut failed = false;
                for member in invoke.members.iter_mut() {
                    member.check(errors);
                    if member.ty.kind == TypeKind::Unknown { failed = true; }
                }

                if failed { return; }

                let primitive = invoke.target.brand()
                    .and_then(|token| match token.kind {
                        TokenKind::Identifier(name) => Some(name),
                        _ => None,
                    })
                    .and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        match invoke.members[0].ty.kind {
                            TypeKind::Boolean => {}
                            _ => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type { kind: TypeKind::Boolean, span },
                                        invoke.members[0].ty.clone(),
                                    ),
                                    invoke.members[0].span,
                                ));
                                failed = true;
                            }
                        }

                        if failed { return; }

                        match Type::unify(&invoke.members[1].ty, &invoke.members[2].ty) {
                            Some(ty) => ty,
                            None => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(
                                        invoke.members[1].ty.clone(),
                                        invoke.members[2].ty.clone(),
                                    ),
                                    span,
                                ));
                                return;
                            }
                        }
                    }

                    Some("while") => {
                        match invoke.members[0].ty.kind {
                            TypeKind::Boolean => {}
                            _ => {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(
                                        Type { kind: TypeKind::Boolean, span },
                                        invoke.members[0].ty.clone(),
                                    ),
                                    invoke.members[0].span,
                                ));
                                failed = true;
                            }
                        }

                        if failed { return; }

                        Type { kind: TypeKind::Tuple { members: Vec::new() }, span }
                    }

                    _ => Type::new(TypeKind::Tuple { members: Vec::new() }, span),
                }
            }

            ElementKind::Construct(construct) => {
                let mut failed = false;
                for field in construct.members.iter_mut() {
                    field.check(errors);
                    if field.ty.kind == TypeKind::Unknown { failed = true; }
                }

                if failed { return; }

                let members = construct.members.iter().map(|f| f.ty.clone()).collect();
                let structure = Structure::new(
                    Str::from(construct.target.brand().unwrap().format(0)),
                    members,
                );

                Type { kind: TypeKind::Structure(structure), span }
            }

            ElementKind::Symbolize(symbol) => {
                symbol.check(errors);

                symbol.ty.clone()
            }
        };

        self.ty = ty;
    }
}
