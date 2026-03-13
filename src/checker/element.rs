use crate::{
    checker::{CheckError, Checkable, Checker, ErrorKind, Type, TypeKind},
    data::{Scale, Str, Structure},
    format::Show,
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
};

impl<'element> Checkable<'element> for Element<'element> {
    fn check(&mut self, checker: &mut Checker<'_, 'element>) {
        let span = self.span;

        let typ = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type::new(TypeKind::Integer { size: 64, signed: true }, literal.span),
                TokenKind::Float(_) => Type::new(TypeKind::Float { size: 64 }, literal.span),
                TokenKind::Boolean(_) => Type::new(TypeKind::Boolean, literal.span),
                TokenKind::String(_) => Type::new(TypeKind::String, literal.span),
                TokenKind::Character(_) => Type::new(TypeKind::Character, literal.span),
                TokenKind::Identifier(_) => {
                    if let Some(identity) = self.reference {
                        checker.lookup(identity, literal.span)
                    } else {
                        checker.fresh(literal.span)
                    }
                }
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
                        delimited.members[0].check(checker);
                        delimited.members[0].typ.clone()
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());
                        for member in &mut delimited.members {
                            member.check(checker);
                            members.push(member.typ.clone());
                        }
                        Type::new(TypeKind::Tuple { members }, span)
                    }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    let scope = checker.environment.clone();
                    let mut block = Type::new(TypeKind::Void, span);
                    let last = delimited.members.len().saturating_sub(1);

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.check(checker);
                        if index == last {
                            block = member.typ.clone();
                        }
                    }

                    checker.environment = scope;
                    block
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    let mut inner = checker.fresh(span);
                    for member in &mut delimited.members {
                        member.check(checker);
                        inner = checker.unify(member.span, &inner, &member.typ);
                    }
                    Type::new(
                        TypeKind::Array {
                            member: Box::new(inner),
                            size: delimited.members.len() as Scale,
                        },
                        span,
                    )
                }

                _ => Type::unit(span),
            },

            ElementKind::Unary(unary) => {
                unary.operand.check(checker);

                let TokenKind::Operator(operator) = &unary.operator.kind else {
                    checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => checker.unify(span, &unary.operand.typ, &Type::new(TypeKind::Boolean, span)),
                    [OperatorKind::Tilde] | [OperatorKind::Plus] | [OperatorKind::Minus] => unary.operand.typ.clone(),
                    [OperatorKind::Ampersand] => {
                        let addressable = match &unary.operand.kind {
                            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) | ElementKind::Index(_) => true,
                            ElementKind::Binary(binary) => matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot)),
                            ElementKind::Unary(inner) => matches!(inner.operator.kind, TokenKind::Operator(OperatorKind::Star)),
                            _ => false,
                        };

                        if addressable {
                            Type::new(TypeKind::Pointer { target: Box::new(unary.operand.typ.clone()) }, span)
                        } else {
                            checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            checker.fresh(span)
                        }
                    }
                    [OperatorKind::Star] => {
                        let target = checker.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(target.clone()) }, span);
                        checker.unify(span, &unary.operand.typ, &pointer);
                        target
                    }
                    _ => {
                        checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                        checker.fresh(span)
                    }
                }
            }

            ElementKind::Binary(binary) => {
                let TokenKind::Operator(operator) = &binary.operator.kind else {
                    checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Dot] => {
                        binary.left.check(checker);
                        if let ElementKind::Literal(Token { kind: TokenKind::Identifier(_), span: right_span }) = &binary.right.kind {
                            let field = checker.fresh(*right_span);
                            binary.right.typ = field.clone();
                            field
                        } else {
                            binary.right.check(checker);
                            binary.right.typ.clone()
                        }
                    }
                    _ => {
                        binary.left.check(checker);
                        binary.right.check(checker);

                        match operator.as_slice() {
                            [OperatorKind::Equal] => checker.unify(span, &binary.left.typ, &binary.right.typ),
                            [OperatorKind::Plus] | [OperatorKind::Minus] | [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                                let lhs = checker.reify(&binary.left.typ);
                                let rhs = checker.reify(&binary.right.typ);

                                let is_void = |typ: &Type| matches!(&typ.kind, TypeKind::Pointer { target } if matches!(&target.kind, TypeKind::Void));

                                if is_void(&lhs) || is_void(&rhs) {
                                    checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                    checker.fresh(span)
                                } else if matches!(lhs.kind, TypeKind::Pointer { .. }) {
                                    lhs
                                } else if matches!(rhs.kind, TypeKind::Pointer { .. }) {
                                    rhs
                                } else {
                                    checker.unify(span, &lhs, &rhs)
                                }
                            }
                            [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] | [OperatorKind::LeftAngle, OperatorKind::LeftAngle] | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                                checker.unify(span, &binary.left.typ, &binary.right.typ)
                            }
                            [OperatorKind::Ampersand, OperatorKind::Ampersand] | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                                let boolean = Type::new(TypeKind::Boolean, span);
                                checker.unify(span, &binary.left.typ, &boolean);
                                checker.unify(span, &binary.right.typ, &boolean);
                                boolean
                            }
                            [OperatorKind::Equal, OperatorKind::Equal] | [OperatorKind::Exclamation, OperatorKind::Equal] | [OperatorKind::LeftAngle] | [OperatorKind::LeftAngle, OperatorKind::Equal] | [OperatorKind::RightAngle] | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                                checker.unify(span, &binary.left.typ, &binary.right.typ);
                                Type::new(TypeKind::Boolean, span)
                            }
                            _ => {
                                checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                                checker.fresh(span)
                            }
                        }
                    }
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    checker.errors.push(CheckError::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                    self.typ = checker.fresh(span);
                    return;
                }

                index.target.check(checker);
                index.members[0].check(checker);

                let expected = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                checker.unify(span, &index.members[0].typ, &expected);

                let target = checker.reify(&index.target.typ);

                match target.kind {
                    TypeKind::Pointer { target } => *target,
                    TypeKind::Array { member, .. } => *member,
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) = index.members[0].kind {
                            if let Some(position) = usize::try_from(value).ok().filter(|&p| p < members.len()) {
                                members[position].clone()
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
                for member in &mut invoke.members {
                    member.check(checker);
                }

                let primitive = invoke.target.brand().and_then(|t| match &t.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                }).and_then(|n| n.as_str());

                match primitive {
                    Some("if") => {
                        let boolean = Type::new(TypeKind::Boolean, span);
                        checker.unify(invoke.members[0].span, &invoke.members[0].typ, &boolean);
                        checker.unify(span, &invoke.members[1].typ, &invoke.members[2].typ)
                    }
                    Some("while") => {
                        let boolean = Type::new(TypeKind::Boolean, span);
                        checker.unify(invoke.members[0].span, &invoke.members[0].typ, &boolean);
                        Type::unit(span)
                    }
                    _ => {
                        invoke.target.check(checker);

                        let output = checker.fresh(span);
                        let arguments = invoke.members.iter().map(|m| m.typ.clone()).collect();
                        let function = Type::new(TypeKind::Function(Str::default(), arguments, Some(Box::new(output.clone()))), span);

                        let unified = checker.unify(span, &invoke.target.typ, &function);

                        match unified.kind {
                            TypeKind::Function(_, _, Some(kind)) => *kind,
                            TypeKind::Function(_, _, None) => Type::new(TypeKind::Void, span),
                            _ => output,
                        }
                    }
                }
            }

            ElementKind::Construct(construct) => {
                construct.target.check(checker);

                for field in &mut construct.members {
                    if let ElementKind::Binary(binary) = &mut field.kind {
                        if let TokenKind::Operator(operator) = &binary.operator.kind {
                            if operator.as_slice() == [OperatorKind::Equal] {
                                if let ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) = binary.left.kind {
                                    binary.left.typ = checker.fresh(binary.left.span);
                                } else {
                                    binary.left.check(checker);
                                }
                                binary.right.check(checker);
                                field.typ = checker.unify(field.span, &binary.left.typ, &binary.right.typ);
                                continue;
                            }
                        }
                    }
                    field.check(checker);
                }

                let members = construct.members.iter().map(|f| f.typ.clone()).collect();
                let head = construct.target.brand().unwrap().format(0).into();
                let structure = Type::new(TypeKind::Structure(Structure::new(head, members)), span);

                checker.unify(span, &construct.target.typ, &structure)
            }

            ElementKind::Symbolize(symbol) => {
                let pre = checker.fresh(span);
                checker.environment.insert(symbol.identity, pre.clone());

                symbol.check(checker);

                let unified = checker.unify(span, &pre, &symbol.typ);
                symbol.typ = unified.clone();
                checker.environment.insert(symbol.identity, unified.clone());

                unified
            }
        };

        self.typ = typ;
    }


    fn reify(&mut self, checker: &mut Checker<'_, 'element>) {
        self.typ = checker.reify(&self.typ);

        match &mut self.kind {
            ElementKind::Literal(_) => {}
            ElementKind::Delimited(delimited) => {
                for member in &mut delimited.members {
                    member.reify(checker);
                }
            }
            ElementKind::Unary(unary) => {
                unary.operand.reify(checker);
            }
            ElementKind::Binary(binary) => {
                binary.left.reify(checker);
                binary.right.reify(checker);
            }
            ElementKind::Index(index) => {
                index.target.reify(checker);
                
                for member in &mut index.members {
                    member.reify(checker);
                }
            }
            ElementKind::Invoke(invoke) => {
                invoke.target.reify(checker);
                
                for member in &mut invoke.members {
                    member.reify(checker);
                }
            }
            ElementKind::Construct(construct) => {
                for member in &mut construct.members {
                    member.reify(checker);
                }
            }
            ElementKind::Symbolize(symbol) => {
                symbol.reify(checker);
            }
        }
    }
}
