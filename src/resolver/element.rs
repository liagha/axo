use crate::{
    data::{Scale, Structure},
    format::Show,
    parser::{Element, ElementKind, SymbolKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(&mut self, resolver: &mut Resolver<'element>) {
        let span = self.span;
        let mut identity = None;

        if matches!(
            &self.kind,
            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. })
                | ElementKind::Construct(_)
                | ElementKind::Invoke(_)
        ) {
            match resolver.scope.lookup(self) {
                Ok(symbol) => identity = Some(symbol.identity),
                Err(errors) => resolver.errors.extend(errors),
            }
        }

        if let Some(id) = identity {
            self.reference = Some(id);
            match &mut self.kind {
                ElementKind::Construct(construct) => construct.target.reference = Some(id),
                ElementKind::Invoke(invoke) => invoke.target.reference = Some(id),
                _ => {}
            }
        }

        let typ = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type::new(TypeKind::Integer { size: 64, signed: true }, literal.span),
                TokenKind::Float(_) => Type::new(TypeKind::Float { size: 64 }, literal.span),
                TokenKind::Boolean(_) => Type::new(TypeKind::Boolean, literal.span),
                TokenKind::String(_) => Type::new(TypeKind::String, literal.span),
                TokenKind::Character(_) => Type::new(TypeKind::Character, literal.span),
                TokenKind::Identifier(_) => {
                    if let Some(id) = self.reference {
                        resolver.lookup(id, literal.span)
                    } else {
                        resolver.fresh(literal.span)
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
                        delimited.members[0].resolve(resolver);
                        delimited.members[0].typ.clone()
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());
                        for member in &mut delimited.members {
                            member.resolve(resolver);
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
                    resolver.enter();
                    let mut block = Type::new(TypeKind::Void, span);
                    let last = delimited.members.len().saturating_sub(1);

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.resolve(resolver);
                        if index == last {
                            block = member.typ.clone();
                        }
                    }

                    resolver.exit();
                    block
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    let mut inner = resolver.fresh(span);
                    for member in &mut delimited.members {
                        member.resolve(resolver);
                        inner = resolver.unify(member.span, &inner, &member.typ);
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
                unary.operand.resolve(resolver);

                let TokenKind::Operator(operator) = &unary.operator.kind else {
                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => resolver.unify(span, &unary.operand.typ, &Type::new(TypeKind::Boolean, span)),
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
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            resolver.fresh(span)
                        }
                    }
                    [OperatorKind::Star] => {
                        let target = resolver.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(target.clone()) }, span);
                        resolver.unify(span, &unary.operand.typ, &pointer);
                        target
                    }
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                        resolver.fresh(span)
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                let TokenKind::Operator(operator) = &binary.operator.kind else {
                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Dot] => {
                        let mut namespace = false;
                        if let Some(id) = binary.left.reference {
                            if let Some(symbol) = resolver.scope.get_identity(id) {
                                if matches!(symbol.kind, SymbolKind::Module(_) | SymbolKind::Structure(_) | SymbolKind::Union(_)) {
                                    namespace = true;
                                    resolver.enter_scope(symbol.scope.clone());
                                    binary.right.resolve(resolver);
                                    resolver.exit();
                                }
                            }
                        }

                        if namespace {
                            self.reference = binary.right.reference;
                        } else {
                            binary.right.resolve(resolver);
                        }

                        binary.right.typ.clone()
                    }
                    _ => {
                        binary.right.resolve(resolver);

                        match operator.as_slice() {
                            [OperatorKind::Equal] => resolver.unify(span, &binary.left.typ, &binary.right.typ),
                            [OperatorKind::Plus] | [OperatorKind::Minus] | [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                                let left = resolver.reify(&binary.left.typ);
                                let right = resolver.reify(&binary.right.typ);

                                let valid = |typ: &Type| matches!(&typ.kind, TypeKind::Integer { .. } | TypeKind::Float { .. } | TypeKind::Pointer { .. } | TypeKind::Variable(_));

                                if !valid(&left) || !valid(&right) {
                                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                    resolver.fresh(span)
                                } else if matches!(left.kind, TypeKind::Pointer { .. }) {
                                    left
                                } else if matches!(right.kind, TypeKind::Pointer { .. }) {
                                    right
                                } else {
                                    resolver.unify(span, &left, &right)
                                }
                            }
                            [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] | [OperatorKind::LeftAngle, OperatorKind::LeftAngle] | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                                resolver.unify(span, &binary.left.typ, &binary.right.typ)
                            }
                            [OperatorKind::Ampersand, OperatorKind::Ampersand] | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                                let boolean = Type::new(TypeKind::Boolean, span);
                                resolver.unify(span, &binary.left.typ, &boolean);
                                resolver.unify(span, &binary.right.typ, &boolean);
                                boolean
                            }
                            [OperatorKind::Equal, OperatorKind::Equal] | [OperatorKind::Exclamation, OperatorKind::Equal] | [OperatorKind::LeftAngle] | [OperatorKind::LeftAngle, OperatorKind::Equal] | [OperatorKind::RightAngle] | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                                resolver.unify(span, &binary.left.typ, &binary.right.typ);
                                Type::new(TypeKind::Boolean, span)
                            }
                            _ => {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                                resolver.fresh(span)
                            }
                        }
                    }
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                    self.typ = resolver.fresh(span);
                    return;
                }

                index.target.resolve(resolver);
                index.members[0].resolve(resolver);

                let target = resolver.reify(&index.target.typ);
                let parameter = resolver.reify(&index.members[0].typ);

                let valid = matches!(parameter.kind, TypeKind::Integer { .. } | TypeKind::Variable(_));

                if !valid {
                    let expected = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                    resolver.errors.push(Error::new(ErrorKind::Mismatch(parameter.clone(), expected), span));
                }

                match target.kind {
                    TypeKind::Pointer { target } => *target,
                    TypeKind::Array { member, .. } => *member,
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) = index.members[0].kind {
                            if let Some(position) = usize::try_from(value).ok().filter(|&p| p < members.len()) {
                                members[position].clone()
                            } else {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                                resolver.fresh(span)
                            }
                        } else {
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                            resolver.fresh(span)
                        }
                    }
                    TypeKind::Variable(_) => {
                        let element = resolver.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(element.clone()) }, span);
                        resolver.unify(span, &target, &pointer);
                        element
                    }
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                        resolver.fresh(span)
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                let mut namespace = false;
                if let Some(id) = invoke.target.reference {
                    if let Some(symbol) = resolver.scope.get_identity(id) {
                        if matches!(symbol.kind, SymbolKind::Function(_)) {
                            resolver.enter_scope(symbol.scope.clone());
                            namespace = true;
                        }
                    }
                }

                for member in &mut invoke.members {
                    member.resolve(resolver);
                }

                if namespace {
                    resolver.exit();
                }

                let primitive = invoke.target.brand().and_then(|t| match &t.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                }).and_then(|n| n.as_str());

                match primitive {
                    Some("if") => {
                        let boolean = Type::new(TypeKind::Boolean, span);
                        resolver.unify(invoke.members[0].span, &invoke.members[0].typ, &boolean);
                        resolver.unify(span, &invoke.members[1].typ, &invoke.members[2].typ)
                    }
                    Some("while") => {
                        let boolean = Type::new(TypeKind::Boolean, span);
                        resolver.unify(invoke.members[0].span, &invoke.members[0].typ, &boolean);
                        Type::unit(span)
                    }
                    _ => {
                        let output = resolver.fresh(span);
                        let arguments = invoke.members.iter().map(|m| m.typ.clone()).collect();
                        let function = Type::new(TypeKind::Function(crate::data::Str::default(), arguments, Some(Box::new(output.clone()))), span);

                        let unified = resolver.unify(span, &invoke.target.typ, &function);

                        match unified.kind {
                            TypeKind::Function(_, _, Some(kind)) => *kind,
                            TypeKind::Function(_, _, None) => Type::new(TypeKind::Void, span),
                            _ => output,
                        }
                    }
                }
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);

                let mut namespace = false;
                if let Some(id) = construct.target.reference {
                    if let Some(symbol) = resolver.scope.get_identity(id) {
                        if matches!(symbol.kind, SymbolKind::Structure(_) | SymbolKind::Union(_)) {
                            resolver.enter_scope(symbol.scope.clone());
                            namespace = true;
                        }
                    }
                }

                for field in &mut construct.members {
                    if let ElementKind::Binary(binary) = &mut field.kind {
                        if let TokenKind::Operator(operator) = &binary.operator.kind {
                            if operator.as_slice() == [OperatorKind::Equal] {
                                if let ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) = binary.left.kind {
                                    binary.left.typ = resolver.fresh(binary.left.span);
                                } else {
                                    binary.left.resolve(resolver);
                                }
                                binary.right.resolve(resolver);
                                field.typ = resolver.unify(field.span, &binary.left.typ, &binary.right.typ);
                                continue;
                            }
                        }
                    }
                    field.resolve(resolver);
                }

                if namespace {
                    resolver.exit();
                }

                let members = construct.members.iter().map(|f| f.typ.clone()).collect();
                let head = construct.target.brand().unwrap().format(0).into();
                let structure = Type::new(TypeKind::Structure(Structure::new(head, members)), span);

                resolver.unify(span, &construct.target.typ, &structure)
            }

            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);

                let pre = resolver.fresh(span);
                resolver.bind(symbol.identity, pre.clone());

                symbol.resolve(resolver);

                let unified = resolver.unify(span, &pre, &symbol.typ);
                symbol.typ = unified.clone();
                resolver.bind(symbol.identity, unified.clone());

                unified
            }
        };

        self.typ = typ;
    }

    fn reify(&mut self, resolver: &mut Resolver<'element>) {
        self.typ = resolver.reify(&self.typ);

        match &mut self.kind {
            ElementKind::Literal(_) => {}
            ElementKind::Delimited(delimited) => {
                for member in &mut delimited.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Unary(unary) => {
                unary.operand.reify(resolver);
            }
            ElementKind::Binary(binary) => {
                binary.left.reify(resolver);
                binary.right.reify(resolver);
            }
            ElementKind::Index(index) => {
                index.target.reify(resolver);
                for member in &mut index.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Invoke(invoke) => {
                invoke.target.reify(resolver);
                for member in &mut invoke.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Construct(construct) => {
                construct.target.reify(resolver);
                for member in &mut construct.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Symbolize(symbol) => {
                symbol.reify(resolver);
            }
        }
    }
}
