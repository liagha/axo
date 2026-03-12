use crate::{
    checker::{CheckError, Checkable, Checker, ErrorKind, Type, TypeKind},
    data::{Scale, Str, Structure},
    format::Show,
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
};

impl<'element> Checkable<'element> for Element<'element> {
    fn check(&mut self, checker: &mut Checker<'_, 'element>) {
        let span = self.span;

        let type_value = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type { kind: TypeKind::Integer { size: 64, signed: true }, span: literal.span },
                TokenKind::Float(_)   => Type { kind: TypeKind::Float { size: 64 }, span: literal.span },
                TokenKind::Boolean(_) => Type { kind: TypeKind::Boolean, span: literal.span },
                TokenKind::String(_)  => Type { kind: TypeKind::String, span: literal.span },
                TokenKind::Character(_) => Type { kind: TypeKind::Character, span: literal.span },
                TokenKind::Identifier(_) => {
                    if let Some(identity) = self.reference {
                        checker.lookup(identity, literal.span)
                    } else {
                        checker.fresh(literal.span)
                    }
                },
                _ => Type::unit(literal.span),
            },

            ElementKind::Delimited(delimited) => match (
                &delimited.start.kind,
                delimited.separator.as_ref().map(|token| &token.kind),
                &delimited.end.kind,
            ) {
                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.separator.is_none() && delimited.members.len() == 1 {
                        delimited.members[0].check(checker);
                        delimited.members[0].ty.clone()
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());
                        for member in &mut delimited.members {
                            member.check(checker);
                            members.push(member.ty.clone());
                        }
                        Type { kind: TypeKind::Tuple { members }, span: Span::void() }
                    }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    let mut block_type = Type::new(TypeKind::Void, Span::void());
                    let last = delimited.members.len().saturating_sub(1);

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.check(checker);
                        if index == last {
                            block_type = member.ty.clone();
                        }
                    }
                    block_type
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    let mut inner = checker.fresh(span);
                    for member in &mut delimited.members {
                        member.check(checker);
                        inner = checker.unify(member.span, &inner, &member.ty);
                    }
                    Type {
                        kind: TypeKind::Array {
                            member: Box::new(inner),
                            size: delimited.members.len() as Scale,
                        },
                        span: Span::void(),
                    }
                }

                _ => Type::unit(Span::void()),
            },

            ElementKind::Unary(unary) => {
                unary.operand.check(checker);

                let TokenKind::Operator(operator) = unary.operator.kind.clone() else {
                    checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => checker.unify(span, &unary.operand.ty, &Type { kind: TypeKind::Boolean, span }),
                    [OperatorKind::Tilde] | [OperatorKind::Plus] | [OperatorKind::Minus] => unary.operand.ty.clone(),
                    [OperatorKind::Ampersand] => {
                        let addressable = match &unary.operand.kind {
                            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) | ElementKind::Index(_) => {
                                true
                            }
                            ElementKind::Binary(binary) => {
                                matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot))
                            }
                            ElementKind::Unary(inner) => {
                                matches!(inner.operator.kind, TokenKind::Operator(OperatorKind::Star))
                            }
                            _ => false,
                        };

                        if addressable {
                            Type { kind: TypeKind::Pointer { target: Box::new(unary.operand.ty.clone()) }, span }
                        } else {
                            checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            checker.fresh(span)
                        }
                    }
                    [OperatorKind::Star] => {
                        let target = checker.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(target.clone()) }, span);
                        checker.unify(span, &unary.operand.ty, &pointer);
                        target
                    }
                    _ => {
                        checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                        checker.fresh(span)
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.check(checker);
                binary.right.check(checker);

                let TokenKind::Operator(operator) = binary.operator.kind.clone() else {
                    checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Equal] => checker.unify(span, &binary.left.ty, &binary.right.ty),
                    [OperatorKind::Plus] | [OperatorKind::Minus] | [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                        let left = checker.concretize(&binary.left.ty);
                        let right = checker.concretize(&binary.right.ty);

                        if matches!(left.kind, TypeKind::Pointer { .. }) {
                            left
                        } else if matches!(right.kind, TypeKind::Pointer { .. }) {
                            right
                        } else {
                            checker.unify(span, &left, &right)
                        }
                    }
                    [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] | [OperatorKind::LeftAngle, OperatorKind::LeftAngle] | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                        checker.unify(span, &binary.left.ty, &binary.right.ty)
                    }
                    [OperatorKind::Ampersand, OperatorKind::Ampersand] | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                        checker.unify(span, &binary.left.ty, &Type { kind: TypeKind::Boolean, span });
                        checker.unify(span, &binary.right.ty, &Type { kind: TypeKind::Boolean, span });
                        Type { kind: TypeKind::Boolean, span }
                    }
                    [OperatorKind::Equal, OperatorKind::Equal] | [OperatorKind::Exclamation, OperatorKind::Equal] | [OperatorKind::LeftAngle] | [OperatorKind::LeftAngle, OperatorKind::Equal] | [OperatorKind::RightAngle] | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                        checker.unify(span, &binary.left.ty, &binary.right.ty);
                        Type { kind: TypeKind::Boolean, span }
                    }
                    [OperatorKind::Dot] => binary.right.ty.clone(),
                    _ => {
                        checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                        checker.fresh(span)
                    }
                }
            }

            ElementKind::Index(index_element) => {
                if index_element.members.is_empty() {
                    self.ty = checker.fresh(span);
                    return;
                }

                index_element.target.check(checker);
                index_element.members[0].check(checker);

                let index_type = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                checker.unify(span, &index_element.members[0].ty, &index_type);

                let target = checker.concretize(&index_element.target.ty);

                match target.kind {
                    TypeKind::Pointer { target } => *target,
                    TypeKind::Array { member, .. } => *member,
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) = index_element.members[0].kind {
                            if let Some(index) = usize::try_from(value).ok().filter(|&i| i < members.len()) {
                                members[index].clone()
                            } else {
                                checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                                checker.fresh(span)
                            }
                        } else {
                            checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                            checker.fresh(span)
                        }
                    }
                    TypeKind::Variable(_) => {
                        let element = checker.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(element.clone()) }, span);
                        checker.unify(span, &target, &pointer);
                        element
                    }
                    _ => {
                        checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                        checker.fresh(span)
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                for member in invoke.members.iter_mut() {
                    member.check(checker);
                }

                let primitive = invoke.target.brand().and_then(|token| match token.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                }).and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        let boolean_type = Type::new(TypeKind::Boolean, span);
                        checker.unify(invoke.members[0].span, &invoke.members[0].ty, &boolean_type);

                        checker.unify(span, &invoke.members[1].ty, &invoke.members[2].ty)
                    }
                    Some("while") => {
                        let boolean_type = Type::new(TypeKind::Boolean, span);
                        checker.unify(invoke.members[0].span, &invoke.members[0].ty, &boolean_type);
                        Type::unit(span)
                    }
                    _ => {
                        invoke.target.check(checker);
                        let return_type = checker.fresh(span);
                        let arguments = invoke.members.iter().map(|member| member.ty.clone()).collect();
                        let function_type = Type::new(TypeKind::Function(Str::default(), arguments, Some(Box::new(return_type.clone()))), span);

                        let unified = checker.unify(span, &invoke.target.ty, &function_type);

                        match unified.kind {
                            TypeKind::Function(_, _, Some(output)) => *output,
                            TypeKind::Function(_, _, None) => Type::new(TypeKind::Void, span),
                            _ => {
                                return_type
                            }
                        }
                    }
                }
            }

            ElementKind::Construct(construct) => {
                for field in construct.members.iter_mut() {
                    field.check(checker);
                }

                let members = construct.members.iter().map(|field| field.ty.clone()).collect();
                let structure = Structure::new(Str::from(construct.target.brand().unwrap().format(0)), members);

                Type { kind: TypeKind::Structure(structure), span }
            }

            ElementKind::Symbolize(symbol) => {
                symbol.check(checker);

                if let Some(existing) = checker.environment.get(&symbol.identity).cloned() {
                    checker.unify(span, &existing, &symbol.ty);
                }

                checker.environment.insert(symbol.identity, symbol.ty.clone());
                symbol.ty.clone()
            }
        };

        self.ty = type_value;
    }
}
