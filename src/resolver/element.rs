use crate::{
    data::{Scale, Structure},
    format::Show,
    parser::{Element, ElementKind, SymbolKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn declare(&mut self, resolver: &mut Resolver<'element>) {
        if let ElementKind::Symbolize(symbol) = &mut self.kind {
            symbol.declare(resolver);
            self.typing = symbol.typing.clone();
        }
    }

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

        if let Some(reference) = identity {
            self.reference = Some(reference);
            match &mut self.kind {
                ElementKind::Construct(construct) => construct.target.reference = Some(reference),
                ElementKind::Invoke(invoke) => invoke.target.reference = Some(reference),
                _ => {}
            }
        }

        let typing = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type::new(TypeKind::Integer { size: 64, signed: true }, literal.span),
                TokenKind::Float(_) => Type::new(TypeKind::Float { size: 64 }, literal.span),
                TokenKind::Boolean(_) => Type::new(TypeKind::Boolean, literal.span),
                TokenKind::String(_) => Type::new(TypeKind::String, literal.span),
                TokenKind::Character(_) => Type::new(TypeKind::Character, literal.span),
                TokenKind::Identifier(_) => {
                    if let Some(reference) = self.reference {
                        resolver.lookup(reference, literal.span)
                    } else {
                        resolver.fresh(literal.span)
                    }
                }
                _ => Type::void(literal.span),
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
                        delimited.members[0].resolve(resolver);
                        delimited.members[0].typing.clone()
                    } else {
                        let mut members = Vec::with_capacity(delimited.members.len());
                        for member in &mut delimited.members {
                            member.resolve(resolver);
                            members.push(member.typing.clone());
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
                            block = member.typing.clone();
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
                        inner = resolver.unify(member.span, &inner, &member.typing);
                    }
                    Type::new(
                        TypeKind::Array {
                            member: Box::new(inner),
                            size: delimited.members.len() as Scale,
                        },
                        span,
                    )
                }

                _ => Type::void(span),
            },

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);

                let TokenKind::Operator(operator) = &unary.operator.kind else {
                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                    return;
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => resolver.unify(span, &unary.operand.typing, &Type::new(TypeKind::Boolean, span)),
                    [OperatorKind::Tilde] | [OperatorKind::Plus] | [OperatorKind::Minus] => unary.operand.typing.clone(),
                    [OperatorKind::Ampersand] => {
                        let addressable = match &unary.operand.kind {
                            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) | ElementKind::Index(_) => true,
                            ElementKind::Binary(binary) => matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot)),
                            ElementKind::Unary(inner) => matches!(inner.operator.kind, TokenKind::Operator(OperatorKind::Star)),
                            _ => false,
                        };

                        if addressable {
                            Type::new(TypeKind::Pointer { target: Box::new(unary.operand.typing.clone()) }, span)
                        } else {
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            resolver.fresh(span)
                        }
                    }
                    [OperatorKind::Star] => {
                        let target = resolver.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(target.clone()) }, span);
                        resolver.unify(span, &unary.operand.typing, &pointer);
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
                        let mut namespace = None;

                        if let Some(reference) = binary.left.reference {
                            if let Some(symbol) = resolver.scope.find(reference) {
                                if matches!(symbol.kind, SymbolKind::Module(_) | SymbolKind::Structure(_) | SymbolKind::Union(_)) {
                                    namespace = Some(symbol.scope.clone());
                                }
                            }
                        }

                        if namespace.is_none() {
                            let left = resolver.reify(&binary.left.typing);
                            let target = match left.kind {
                                TypeKind::Pointer { target } => target.kind,
                                kind => kind,
                            };
                            match target {
                                TypeKind::Structure(reference, _) | TypeKind::Union(reference, _) => {
                                    if let Some(symbol) = resolver.scope.find(reference) {
                                        namespace = Some(symbol.scope.clone());
                                    }
                                }
                                _ => {}
                            }
                        }

                        if let Some(scope) = namespace {
                            resolver.enter_scope(scope);
                            binary.right.resolve(resolver);
                            resolver.exit();
                            self.reference = binary.right.reference;
                        } else {
                            binary.right.resolve(resolver);
                        }

                        binary.right.typing.clone()
                    }
                    _ => {
                        binary.right.resolve(resolver);

                        match operator.as_slice() {
                            [OperatorKind::Equal] => resolver.unify(span, &binary.left.typing, &binary.right.typing),
                            [OperatorKind::Plus] | [OperatorKind::Minus] | [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                                let left = resolver.reify(&binary.left.typing);
                                let right = resolver.reify(&binary.right.typing);

                                let valid = |typing: &Type| matches!(&typing.kind, TypeKind::Integer { .. } | TypeKind::Float { .. } | TypeKind::Pointer { .. } | TypeKind::Variable(_) | TypeKind::Unknown);

                                if !valid(&left) || !valid(&right) {
                                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                    resolver.fresh(span)
                                } else {
                                    resolver.unify(span, &left, &right)
                                }
                            }
                            [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] => {
                                resolver.unify(span, &binary.left.typing, &binary.right.typing)
                            }
                            [OperatorKind::Ampersand, OperatorKind::Ampersand] | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                                let boolean = Type::new(TypeKind::Boolean, span);
                                resolver.unify(span, &binary.left.typing, &boolean);
                                resolver.unify(span, &binary.right.typing, &boolean);
                                boolean
                            }
                            [OperatorKind::Equal, OperatorKind::Equal] | [OperatorKind::Exclamation, OperatorKind::Equal] | [OperatorKind::LeftAngle] | [OperatorKind::LeftAngle, OperatorKind::Equal] | [OperatorKind::RightAngle] | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                                let unified = resolver.unify(span, &binary.left.typing, &binary.right.typing);
                                let check = resolver.reify(&unified);

                                let valid = matches!(
                                    check.kind,
                                    TypeKind::Integer { .. } | TypeKind::Float { .. } | TypeKind::Boolean | TypeKind::Character | TypeKind::String | TypeKind::Pointer { .. } | TypeKind::Variable(_)
                                );

                                if !valid {
                                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                }

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
                    self.typing = resolver.fresh(span);
                    return;
                }

                index.target.resolve(resolver);
                index.members[0].resolve(resolver);

                let target = resolver.reify(&index.target.typing);
                let parameter = resolver.reify(&index.members[0].typing);

                match target.kind {
                    TypeKind::Pointer { target } => {
                        let expected = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                        resolver.unify(span, &parameter, &expected);
                        *target
                    }
                    TypeKind::Array { member, .. } => {
                        let expected = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                        resolver.unify(span, &parameter, &expected);
                        *member
                    }
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token { kind: TokenKind::Integer(value), .. }) = index.members[0].kind {
                            if let Some(position) = usize::try_from(value).ok().filter(|&position| position < members.len()) {
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
                        let expected = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                        resolver.unify(span, &parameter, &expected);

                        let element = resolver.fresh(span);
                        let pointer = Type::new(TypeKind::Pointer { target: Box::new(element.clone()) }, span);
                        resolver.unify(span, &target, &pointer);
                        element
                    }
                    TypeKind::Unknown => Type::new(TypeKind::Unknown, span),
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                        resolver.fresh(span)
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                let mut namespace = false;
                if let Some(reference) = invoke.target.reference {
                    if let Some(symbol) = resolver.scope.find(reference) {
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

                let primitive = invoke.target.brand().and_then(|brand| match &brand.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                }).and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        let boolean = Type::new(TypeKind::Boolean, span);
                        resolver.unify(invoke.members[0].span, &invoke.members[0].typing, &boolean);
                        if invoke.members.len() == 3 {
                            resolver.unify(span, &invoke.members[1].typing, &invoke.members[2].typing)
                        } else {
                            Type::new(TypeKind::Void, span)
                        }
                    }
                    Some("while") => {
                        let boolean = Type::new(TypeKind::Boolean, span);
                        resolver.unify(invoke.members[0].span, &invoke.members[0].typing, &boolean);
                        Type::void(span)
                    }
                    _ => {
                        let output = resolver.fresh(span);
                        let arguments = invoke.members.iter().map(|member| member.typing.clone()).collect();
                        let function = Type::new(TypeKind::Function(crate::data::Str::default(), arguments, Some(Box::new(output.clone()))), span);

                        let unified = resolver.unify(span, &invoke.target.typing, &function);

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

                let mut identifier = 0;
                let mut layout = Vec::new();
                let mut namespace = None; // Fetch the nested scope mapping representing the structure mapping namespace

                if let Some(reference) = construct.target.reference {
                    identifier = reference;
                    if let Some(symbol) = resolver.scope.find(reference) {
                        if let SymbolKind::Structure(structure) | SymbolKind::Union(structure) = &symbol.kind {
                            namespace = Some(symbol.scope.clone());
                            for member in &structure.members {
                                if let SymbolKind::Binding(binding) = &member.kind {
                                    if let Some(token) = binding.target.brand() {
                                        layout.push((token.format(0), member.typing.clone()));
                                    }
                                }
                            }
                        }
                    }
                }

                for field in &mut construct.members {
                    if let ElementKind::Binary(binary) = &mut field.kind {
                        if let TokenKind::Operator(operator) = &binary.operator.kind {
                            if operator.as_slice() == [OperatorKind::Equal] {
                                binary.right.resolve(resolver);

                                let mut matched = false;
                                if let Some(token) = binary.left.brand() {
                                    let name = token.format(0);
                                    for (key, typing) in &layout {
                                        if key == &name {
                                            field.typing = resolver.unify(field.span, typing, &binary.right.typing);

                                            // Ensure binary.left has the correct resolution
                                            if let Some(scope) = &namespace {
                                                resolver.enter_scope(scope.clone());
                                                binary.left.resolve(resolver);
                                                resolver.exit();
                                            } else {
                                                binary.left.typing = typing.clone();
                                            }

                                            matched = true;
                                            break;
                                        }
                                    }
                                }

                                if !matched {
                                    field.typing = resolver.fresh(field.span);
                                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), field.span));
                                }
                                continue;
                            }
                        }
                    }
                    field.resolve(resolver);
                }

                let members = layout.into_iter().map(|(_, typing)| typing).collect();
                let head = construct.target.brand().unwrap().format(0).into();
                let structure = Type::new(TypeKind::Structure(identifier, Structure::new(head, members)), span);

                resolver.unify(span, &construct.target.typing, &structure)
            }

            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);
                symbol.resolve(resolver);
                symbol.typing.clone()
            }
        };

        self.typing = typing;
    }

    fn reify(&mut self, resolver: &mut Resolver<'element>) {
        self.typing = resolver.reify(&self.typing);

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
