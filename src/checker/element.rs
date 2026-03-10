use crate::{
    data::*,
    checker::{CheckError, Checkable, ErrorKind, Type, TypeKind},
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
    format::Show,
};

impl<'element> Checkable<'element> for Element<'element> {
    fn check(&mut self) -> Result<(), Vec<CheckError<'element>>> {
        let ty = match &mut self.kind.clone() {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type { kind: TypeKind::Integer { size: 64, signed: true }, span: literal.span },
                TokenKind::Float(_)   => Type { kind: TypeKind::Float { size: 64 }, span: literal.span },
                TokenKind::Boolean(_) => Type { kind: TypeKind::Boolean, span: literal.span },
                TokenKind::String(_)  => Type { kind: TypeKind::String, span: literal.span },
                TokenKind::Character(_) => Type { kind: TypeKind::Character, span: literal.span },
                _ => Type { kind: TypeKind::Void, span: literal.span },
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
                        return delimited.members[0].check();
                    }

                    let mut errors = Vec::new();
                    for member in delimited.members.iter_mut() {
                        if let Err(errs) = member.check() {
                            errors.extend(errs);
                        }
                    }

                    if !errors.is_empty() {
                        return Err(errors);
                    }

                    let members = delimited.members.iter().map(|m| m.ty.clone()).collect();
                    Type { kind: TypeKind::Tuple { members }, span: Span::void() }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    let last = delimited.members.len() - 1;
                    let mut ty = Type { kind: TypeKind::Tuple { members: Vec::new() }, span: Span::void() };

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.check()?;
                        if index == last {
                            ty = member.ty.clone();
                        }
                    }

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
                                member: Box::new(Type { kind: TypeKind::Void, span: Span::void() }),
                                size: 0,
                            },
                            span: Span::void(),
                        }
                    } else {
                        let mut errors = Vec::new();

                        if let Err(errs) = delimited.members[0].check() {
                            errors.extend(errs);
                        }
                        let inner = delimited.members[0].ty.clone();

                        for member in delimited.members.iter_mut().skip(1) {
                            if let Err(errs) = member.check() {
                                errors.extend(errs);
                            }
                            if inner != member.ty {
                                errors.push(CheckError::new(
                                    ErrorKind::Mismatch(inner.clone(), member.ty.clone()),
                                    member.span,
                                ));
                            }
                        }

                        if !errors.is_empty() {
                            return Err(errors);
                        }

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
                unary.operand.check()?;

                let TokenKind::Operator(operator) = unary.operator.kind.clone() else {
                    return Err(vec![CheckError::new(
                        ErrorKind::InvalidOperation(unary.operator.clone()),
                        unary.operator.span,
                    )]);
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => match unary.operand.ty.kind {
                        TypeKind::Boolean => Type { kind: TypeKind::Boolean, span: self.span },
                        _ => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(
                                Type { kind: TypeKind::Boolean, span: self.span },
                                unary.operand.ty.clone(),
                            ),
                            self.span,
                        )]),
                    },

                    [OperatorKind::Tilde] => match unary.operand.ty.kind {
                        TypeKind::Integer { .. } => unary.operand.ty.clone(),
                        _ => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(
                                Type { kind: TypeKind::Integer { size: 64, signed: true }, span: self.span },
                                unary.operand.ty.clone(),
                            ),
                            self.span,
                        )]),
                    },

                    [OperatorKind::Plus] | [OperatorKind::Minus] => match unary.operand.ty.kind {
                        TypeKind::Integer { .. } | TypeKind::Float { .. } => unary.operand.ty.clone(),
                        _ => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(
                                Type { kind: TypeKind::Integer { size: 64, signed: true }, span: self.span },
                                unary.operand.ty.clone(),
                            ),
                            self.span,
                        )]),
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
                                span: self.span,
                            },
                            false => return Err(vec![CheckError::new(
                                ErrorKind::InvalidOperation(unary.operator.clone()),
                                unary.operator.span,
                            )]),
                        }
                    }

                    [OperatorKind::Star] => match unary.operand.ty.clone().kind {
                        TypeKind::Pointer { target } => *target,
                        TypeKind::Void => Type { kind: TypeKind::Void, span: self.span },
                        _ => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(
                                Type {
                                    kind: TypeKind::Pointer {
                                        target: Box::new(Type { kind: TypeKind::Void, span: self.span }),
                                    },
                                    span: self.span,
                                },
                                unary.operand.ty.clone(),
                            ),
                            self.span,
                        )]),
                    },

                    _ => return Err(vec![CheckError::new(
                        ErrorKind::InvalidOperation(unary.operator.clone()),
                        unary.operator.span,
                    )]),
                }
            }

            ElementKind::Binary(binary) => {
                let mut errors = Vec::new();
                if let Err(errs) = binary.left.check()  { errors.extend(errs); }
                if let Err(errs) = binary.right.check() { errors.extend(errs); }
                if !errors.is_empty() { return Err(errors); }

                let TokenKind::Operator(operator) = binary.operator.kind.clone() else {
                    return Err(vec![CheckError::new(
                        ErrorKind::InvalidOperation(binary.operator.clone()),
                        binary.operator.span,
                    )]);
                };

                match operator.as_slice() {
                    [OperatorKind::Equal] => match Type::unify(&binary.left.ty, &binary.right.ty) {
                        Some(_) => binary.left.ty.clone(),
                        None => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                            binary.operator.span,
                        )]),
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
                        _ => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                            binary.operator.span,
                        )]),
                    },

                    [OperatorKind::Minus] => match (&binary.left.ty.kind, &binary.right.ty.kind) {
                        (TypeKind::Pointer { .. }, TypeKind::Pointer { .. }) => {
                            if binary.left.ty == binary.right.ty {
                                Type { kind: TypeKind::Integer { size: 64, signed: true }, span: binary.operator.span }
                            } else {
                                return Err(vec![CheckError::new(
                                    ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                    binary.operator.span,
                                )]);
                            }
                        }
                        (TypeKind::Pointer { .. }, TypeKind::Integer { .. }) => {
                            binary.left.ty.span = binary.operator.span;
                            binary.left.ty.clone()
                        }
                        (TypeKind::Pointer { .. }, _) => return Err(vec![CheckError::new(
                            ErrorKind::InvalidOperation(binary.operator.clone()),
                            binary.operator.span,
                        )]),
                        (_, TypeKind::Pointer { .. }) => return Err(vec![CheckError::new(
                            ErrorKind::InvalidOperation(binary.operator.clone()),
                            binary.operator.span,
                        )]),
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
                        _ => return Err(vec![CheckError::new(
                            ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                            binary.operator.span,
                        )]),
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
                            _ => return Err(vec![CheckError::new(
                                ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                binary.operator.span,
                            )]),
                        }
                    }

                    [OperatorKind::Ampersand]
                    | [OperatorKind::Pipe]
                    | [OperatorKind::Caret]
                    | [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                    | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (TypeKind::Integer { .. }, TypeKind::Integer { .. }) => binary.left.ty.clone(),
                            _ => return Err(vec![CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Integer { size: 64, signed: true }, span: self.span },
                                    binary.right.ty.clone(),
                                ),
                                self.span,
                            )]),
                        }
                    }

                    [OperatorKind::Ampersand, OperatorKind::Ampersand]
                    | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                        match (&binary.left.ty.kind, &binary.right.ty.kind) {
                            (TypeKind::Boolean, TypeKind::Boolean) => Type { kind: TypeKind::Boolean, span: binary.operator.span },
                            _ => return Err(vec![CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Boolean, span: self.span },
                                    binary.right.ty.clone(),
                                ),
                                self.span,
                            )]),
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
                            None => return Err(vec![CheckError::new(
                                ErrorKind::Mismatch(binary.left.ty.clone(), binary.right.ty.clone()),
                                binary.operator.span,
                            )]),
                        }
                    }

                    [OperatorKind::Dot] => binary.right.ty.clone(),

                    _ => return Err(vec![CheckError::new(
                        ErrorKind::InvalidOperation(binary.operator.clone()),
                        binary.operator.span,
                    )]),
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    self.ty = Type { kind: TypeKind::Tuple { members: Vec::new() }, span: self.span };
                    return Ok(());
                }

                index.target.check()?;
                index.members[0].check()?;

                let target_ty = index.target.ty.clone();
                let index_ty  = index.members[0].ty.clone();

                match index_ty.kind {
                    TypeKind::Integer { .. } => {}
                    _ => return Err(vec![CheckError::new(
                        ErrorKind::Mismatch(
                            Type { kind: TypeKind::Integer { size: 64, signed: true }, span: self.span },
                            index_ty,
                        ),
                        self.span,
                    )]),
                }

                match target_ty.kind {
                    TypeKind::Array { member, .. } => *member,
                    TypeKind::Tuple { members } => {
                        return match &index.members[0].kind {
                            ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) => {
                                match usize::try_from(*value).ok().filter(|&i| i < members.len()) {
                                    Some(idx) => {
                                        self.ty = members[idx].clone();
                                        Ok(())
                                    }
                                    None => Err(vec![CheckError::new(
                                        ErrorKind::InvalidOperation(Token::new(
                                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                            self.span,
                                        )),
                                        self.span,
                                    )]),
                                }
                            }
                            _ => Err(vec![CheckError::new(
                                ErrorKind::InvalidOperation(Token::new(
                                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                                    self.span,
                                )),
                                self.span,
                            )]),
                        }
                    }
                    _ => return Err(vec![CheckError::new(
                        ErrorKind::InvalidOperation(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        )),
                        self.span,
                    )]),
                }
            }

            ElementKind::Invoke(invoke) => {
                let primitive = invoke.target.brand()
                    .and_then(|token| match token.kind {
                        TokenKind::Identifier(name) => Some(name),
                        _ => None,
                    })
                    .and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        let mut errors = Vec::new();
                        for member in invoke.members.iter_mut() {
                            if let Err(errs) = member.check() { errors.extend(errs); }
                        }

                        match invoke.members[0].ty.kind {
                            TypeKind::Boolean => {}
                            _ => errors.push(CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Boolean, span: self.span },
                                    invoke.members[0].ty.clone(),
                                ),
                                invoke.members[0].span,
                            )),
                        }

                        if !errors.is_empty() {
                            return Err(errors);
                        }

                        match Type::unify(&invoke.members[1].ty, &invoke.members[2].ty) {
                            Some(ty) => ty,
                            None => return Err(vec![CheckError::new(
                                ErrorKind::Mismatch(
                                    invoke.members[1].ty.clone(),
                                    invoke.members[2].ty.clone(),
                                ),
                                self.span,
                            )]),
                        }
                    }

                    Some("while") => {
                        let mut errors = Vec::new();
                        for member in invoke.members.iter_mut() {
                            if let Err(errs) = member.check() { errors.extend(errs); }
                        }

                        match invoke.members[0].ty.kind {
                            TypeKind::Boolean => {}
                            _ => errors.push(CheckError::new(
                                ErrorKind::Mismatch(
                                    Type { kind: TypeKind::Boolean, span: self.span },
                                    invoke.members[0].ty.clone(),
                                ),
                                invoke.members[0].span,
                            )),
                        }

                        if !errors.is_empty() {
                            return Err(errors);
                        }

                        Type { kind: TypeKind::Tuple { members: Vec::new() }, span: self.span }
                    }

                    _ => Type { kind: TypeKind::Void, span: self.span },
                }
            }

            ElementKind::Construct(construct) => {
                let mut errors = Vec::new();
                for field in construct.members.iter_mut() {
                    if let Err(errs) = field.check() { errors.extend(errs); }
                }

                if !errors.is_empty() {
                    return Err(errors);
                }

                let members = construct.members.iter().map(|f| f.ty.clone()).collect();
                let structure = Structure::new(
                    Str::from(construct.target.brand().unwrap().format(0)),
                    members,
                );

                Type { kind: TypeKind::Structure(structure), span: self.span }
            }

            ElementKind::Symbolize(symbol) => return symbol.check(),
        };

        self.ty = ty;
        Ok(())
    }
}
