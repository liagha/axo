use crate::{
    data::{Aggregate, Delimited, Scale, Str},
    parser::{Element, ElementKind, SymbolKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Spanned,
};

fn assignable(element: &Element) -> bool {
    match &element.kind {
        ElementKind::Literal(token) => matches!(token.kind, TokenKind::Identifier(_)),
        ElementKind::Index(index) => assignable(&index.target),
        ElementKind::Binary(binary) => {
            matches!(&binary.operator.kind, TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Dot])
                && assignable(&binary.left)
        }
        ElementKind::Unary(unary) => {
            matches!(&unary.operator.kind, TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Star])
        }
        _ => false,
    }
}

impl<'element> Resolvable<'element> for Element<'element> {
    fn declare(&mut self, resolver: &mut Resolver<'element>) {
        match &mut self.kind {
            ElementKind::Symbolize(symbol) => {
                symbol.declare(resolver);
                self.typing = symbol.typing.clone();
            }
            ElementKind::Delimited(delimited) => {
                if let Delimited {
                    start:
                        Token {
                            kind: TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            ..
                        },
                    members,
                    separator:
                        None
                        | Some(Token {
                            kind: TokenKind::Punctuation(PunctuationKind::Semicolon),
                            ..
                        }),
                    end:
                        Token {
                            kind: TokenKind::Punctuation(PunctuationKind::RightBrace),
                            ..
                        },
                } = delimited.as_mut()
                {
                    for member in members {
                        member.declare(resolver);
                    }
                }
            }
            _ => {}
        }
    }

    fn resolve(&mut self, resolver: &mut Resolver<'element>) {
        if matches!(
            &self.kind,
            ElementKind::Literal(Token {
                kind: TokenKind::Identifier(_),
                ..
            }) | ElementKind::Construct(_)
                | ElementKind::Invoke(_)
        ) {
            match resolver.lookup(self) {
                Ok(symbol) => {
                    self.reference = Some(symbol.identity);
                    match &mut self.kind {
                        ElementKind::Construct(construct) => {
                            construct.target.reference = Some(symbol.identity)
                        }
                        ElementKind::Invoke(invoke) => {
                            invoke.target.reference = Some(symbol.identity)
                        }
                        _ => {}
                    }
                }
                Err(errors) => resolver.errors.extend(errors),
            }
        }

        self.typing = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type::from(TypeKind::Integer {
                    size: 64,
                    signed: true,
                }),
                TokenKind::Float(_) => Type::from(TypeKind::Float { size: 64 }),
                TokenKind::Boolean(_) => Type::from(TypeKind::Boolean),
                TokenKind::String(_) => Type::from(TypeKind::String),
                TokenKind::Character(_) => Type::from(TypeKind::Character),
                TokenKind::Identifier(_) => {
                    if let Ok(symbol) = resolver.lookup(self) {
                        symbol.typing
                    } else {
                        self.typing.clone()
                    }
                }
                _ => Type::from(TypeKind::Void),
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

                        Type::from(TypeKind::Tuple { members })
                    }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    resolver.enter();

                    let mut block = Type::from(TypeKind::Void);

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
                    let mut inner = resolver.fresh();

                    for member in &mut delimited.members {
                        member.resolve(resolver);
                        inner = resolver.unify(member.span, &inner, &member.typing);
                    }

                    Type::from(TypeKind::Array {
                        member: Box::new(inner),
                        size: delimited.members.len() as Scale,
                    })
                }

                _ => Type::from(TypeKind::Void),
            },

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);

                match &unary.operator.kind {
                    TokenKind::Operator(operator) => match operator.as_slice() {
                        [OperatorKind::Exclamation] => resolver.unify(
                            unary.operand.span,
                            &unary.operand.typing,
                            &Type::from(TypeKind::Boolean),
                        ),
                        [OperatorKind::Tilde] => {
                            let expect = resolver.reify(&unary.operand.typing);

                            if !matches!(
                                expect.kind,
                                TypeKind::Integer { .. } | TypeKind::Variable(_)
                            ) {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidOperation(unary.operator.clone()),
                                    unary.operator.span,
                                ));
                            }

                            unary.operand.typing.clone()
                        }
                        [OperatorKind::Plus] | [OperatorKind::Minus] => {
                            unary.operand.typing.clone()
                        }
                        [OperatorKind::Ampersand] => {
                            if assignable(&unary.operand) {
                                Type::new(
                                    unary.operand.typing.identity,
                                    TypeKind::Pointer {
                                        target: Box::new(unary.operand.typing.clone()),
                                    },
                                )
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidOperation(unary.operator.clone()),
                                    unary.operator.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Star] => {
                            let target = resolver.fresh();
                            let pointer = Type::new(
                                target.identity,
                                TypeKind::Pointer {
                                    target: Box::new(target.clone()),
                                },
                            );

                            resolver.unify(unary.operand.span, &unary.operand.typing, &pointer);

                            target
                        }
                        _ => {
                            resolver.errors.push(Error::new(
                                ErrorKind::InvalidOperation(unary.operator.clone()),
                                unary.operator.span,
                            ));
                            resolver.fresh()
                        }
                    },
                    _ => {
                        resolver.errors.push(Error::new(
                            ErrorKind::InvalidOperation(unary.operator.clone()),
                            unary.operator.span,
                        ));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                match &binary.operator.kind {
                    TokenKind::Operator(operator) => match operator.as_slice() {
                        [OperatorKind::Dot] => {
                            let mut left = resolver.reify(&binary.left.typing);

                            while let TypeKind::Pointer { target } = left.kind {
                                left = resolver.reify(&target);
                            }

                            let mut scope = None;

                            if let Some(reference) = binary.left.reference {
                                if let Some(symbol) = resolver.get_symbol(reference).cloned() {
                                    if !symbol.is_instance() {
                                        scope = Some(symbol.scope);
                                    }
                                }
                            }

                            if scope.is_none() {
                                if let Some(symbol) = resolver.get_symbol(left.identity).cloned() {
                                    scope = Some(symbol.scope);
                                }
                            }

                            if let Some(scope) = scope {
                                resolver.enter_scope(scope);
                                binary.right.resolve(resolver);
                                resolver.exit();

                                self.reference = binary.right.reference;
                                binary.right.typing.clone()
                            } else {
                                if !matches!(left.kind, TypeKind::Unknown) {
                                    resolver.errors.push(Error::new(
                                        ErrorKind::InvalidOperation(binary.operator.clone()),
                                        binary.operator.span,
                                    ));
                                }

                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Equal] => {
                            binary.right.resolve(resolver);

                            resolver.unify(
                                binary.right.span,
                                &binary.left.typing,
                                &binary.right.typing,
                            )
                        }
                        [OperatorKind::Plus]
                        | [OperatorKind::Minus]
                        | [OperatorKind::Star]
                        | [OperatorKind::Slash]
                        | [OperatorKind::Percent] => {
                            binary.right.resolve(resolver);

                            let left = resolver.reify(&binary.left.typing);
                            let right = resolver.reify(&binary.right.typing);

                            let is_valid = |typing: &Type| {
                                matches!(
                                    typing.kind,
                                    TypeKind::Integer { .. }
                                        | TypeKind::Float { .. }
                                        | TypeKind::Pointer { .. }
                                        | TypeKind::Variable(_)
                                        | TypeKind::Unknown
                                )
                            };

                            if is_valid(&left) && is_valid(&right) {
                                resolver.unify(binary.right.span, &left, &right)
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.right.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                        | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                            binary.right.resolve(resolver);

                            let left = resolver.reify(&binary.left.typing);
                            let right = resolver.reify(&binary.right.typing);

                            let is_valid = |typing: &Type| {
                                matches!(
                                    typing.kind,
                                    TypeKind::Integer { .. }
                                        | TypeKind::Variable(_)
                                        | TypeKind::Unknown
                                )
                            };

                            if is_valid(&left) && is_valid(&right) {
                                resolver.unify(binary.right.span, &left, &right)
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.right.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Ampersand]
                        | [OperatorKind::Pipe]
                        | [OperatorKind::Caret] => {
                            binary.right.resolve(resolver);

                            let left = resolver.reify(&binary.left.typing);
                            let right = resolver.reify(&binary.right.typing);
                            let is_valid = |typing: &Type| {
                                matches!(
                                    typing.kind,
                                    TypeKind::Integer { .. }
                                        | TypeKind::Boolean
                                        | TypeKind::Variable(_)
                                        | TypeKind::Unknown
                                )
                            };

                            if is_valid(&left) && is_valid(&right) {
                                resolver.unify(
                                    binary.right.span,
                                    &binary.left.typing,
                                    &binary.right.typing,
                                )
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.right.span,
                                ));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Ampersand, OperatorKind::Ampersand]
                        | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                            binary.right.resolve(resolver);

                            let boolean = Type::from(TypeKind::Boolean);

                            resolver.unify(binary.right.span, &binary.left.typing, &boolean);
                            resolver.unify(binary.right.span, &binary.right.typing, &boolean);

                            boolean
                        }
                        [OperatorKind::Equal, OperatorKind::Equal]
                        | [OperatorKind::Exclamation, OperatorKind::Equal]
                        | [OperatorKind::LeftAngle]
                        | [OperatorKind::LeftAngle, OperatorKind::Equal]
                        | [OperatorKind::RightAngle]
                        | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                            binary.right.resolve(resolver);

                            let merged = resolver.unify(
                                binary.right.span,
                                &binary.left.typing,
                                &binary.right.typing,
                            );
                            let real = resolver.reify(&merged);

                            let is_valid = matches!(
                                real.kind,
                                TypeKind::Integer { .. }
                                    | TypeKind::Float { .. }
                                    | TypeKind::Boolean
                                    | TypeKind::Character
                                    | TypeKind::String
                                    | TypeKind::Pointer { .. }
                                    | TypeKind::Variable(_)
                                    | TypeKind::Unknown
                            );

                            if !is_valid {
                                resolver.errors.push(Error::new(
                                    ErrorKind::InvalidOperation(binary.operator.clone()),
                                    binary.right.span,
                                ));
                            }

                            Type::from(TypeKind::Boolean)
                        }
                        _ => {
                            binary.right.resolve(resolver);

                            resolver.errors.push(Error::new(
                                ErrorKind::InvalidOperation(binary.operator.clone()),
                                binary.operator.span,
                            ));
                            resolver.fresh()
                        }
                    },
                    _ => {
                        resolver.errors.push(Error::new(
                            ErrorKind::InvalidOperation(binary.operator.clone()),
                            binary.operator.span,
                        ));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    resolver
                        .errors
                        .push(Error::new(ErrorKind::EmptyIndex, self.span));

                    resolver.fresh()
                } else {
                    index.target.resolve(resolver);
                    index.members[0].resolve(resolver);

                    let target = resolver.reify(&index.target.typing);
                    let member = resolver.reify(&index.members[0].typing);

                    let expect = Type::from(TypeKind::Integer {
                        size: 64,
                        signed: true,
                    });

                    resolver.unify(index.members.span(), &member, &expect);

                    match target.kind {
                        TypeKind::Pointer { target: base } => {
                            if let TypeKind::Array { member, .. } = base.kind {
                                *member
                            } else {
                                *base
                            }
                        }
                        TypeKind::Array { member, .. } => *member,
                        TypeKind::Tuple { members } => {
                            let value = if let ElementKind::Literal(Token {
                                kind: TokenKind::Integer(literal),
                                ..
                            }) = index.members[0].kind
                            {
                                usize::try_from(literal).unwrap()
                            } else {
                                unreachable!()
                            };

                            if value < members.len() {
                                members[value].clone()
                            } else {
                                resolver.errors.push(Error::new(
                                    ErrorKind::IndexOutOfBounds(value, members.len()),
                                    index.members.span(),
                                ));
                                resolver.fresh()
                            }
                        }
                        TypeKind::Variable(_) => {
                            let element = resolver.fresh();
                            let pointer = Type::new(
                                element.identity,
                                TypeKind::Pointer {
                                    target: Box::new(element.clone()),
                                },
                            );

                            resolver.unify(index.members.span(), &target, &pointer);

                            element
                        }
                        _ => {
                            resolver
                                .errors
                                .push(Error::new(ErrorKind::UnIndexable, index.target.span));
                            resolver.fresh()
                        }
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                let target = invoke.target.target().and_then(|name| name.as_str());

                match target {
                    Some("if") => {
                        if invoke.members.len() < 2 {
                            Type::from(TypeKind::Void)
                        } else {
                            resolver.enter();

                            invoke.members[0].resolve(resolver);

                            let boolean = Type::from(TypeKind::Boolean);

                            resolver.unify(
                                invoke.members[0].span,
                                &invoke.members[0].typing,
                                &boolean,
                            );

                            invoke.members[1].resolve(resolver);

                            let then = invoke.members[1].typing.clone();

                            let typing = if invoke.members.len() == 3 {
                                invoke.members[2].resolve(resolver);

                                resolver.unify(
                                    invoke.members[2].span,
                                    &then,
                                    &invoke.members[2].typing,
                                )
                            } else {
                                let void = Type::from(TypeKind::Void);
                                resolver.unify(invoke.members[1].span, &then, &void);
                                void
                            };

                            resolver.exit();

                            typing
                        }
                    }
                    Some("while") => {
                        if !invoke.members.is_empty() {
                            invoke.members[0].resolve(resolver);

                            let boolean = Type::from(TypeKind::Boolean);

                            resolver.unify(
                                invoke.members[0].span,
                                &invoke.members[0].typing,
                                &boolean,
                            );
                        }

                        if invoke.members.len() > 1 {
                            invoke.members[1].resolve(resolver);
                        }

                        Type::from(TypeKind::Void)
                    }
                    Some("return") => {
                        if !invoke.members.is_empty() {
                            invoke.members[0].resolve(resolver);
                        }

                        let value = invoke.members.first().map_or_else(
                            || Type::from(TypeKind::Void),
                            |member| member.typing.clone(),
                        );

                        if let Some(expect) = resolver.returns.last().cloned() {
                            resolver.unify(self.span, &expect, &value);
                        }

                        Type::from(TypeKind::Unknown)
                    }
                    Some("continue") | Some("break") => Type::from(TypeKind::Unknown),
                    _ => {
                        for member in &mut invoke.members {
                            member.resolve(resolver);
                        }

                        let output = resolver.fresh();
                        let mut members = Vec::new();

                        members.extend(invoke.members.iter().map(|member| member.typing.clone()));

                        let mut function = Type::from(TypeKind::Function(
                            Str::default(),
                            members,
                            Some(Box::new(output.clone())),
                        ));
                        function = resolver.unify(self.span, &invoke.target.typing, &function);

                        match function.kind {
                            TypeKind::Function(_, _, Some(kind)) => *kind,
                            TypeKind::Function(_, _, None) => Type::from(TypeKind::Void),
                            _ => output,
                        }
                    }
                }
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);

                let mut layout = Vec::new();
                let mut typing = None;

                if let Some(reference) = self.reference {
                    if let Some(symbol) = resolver.get_symbol(reference).cloned() {
                        match symbol.kind {
                            SymbolKind::Structure(mut structure) => {
                                resolver.enter_scope(symbol.scope.clone());

                                for member in &mut structure.members {
                                    if member.is_instance() {
                                        member.resolve(resolver);
                                        layout.push(member.typing.clone());
                                    }
                                }

                                for (index, member) in construct.members.iter_mut().enumerate() {
                                    member.resolve(resolver);

                                    if let Some(member_type) = layout.get(index) {
                                        resolver.unify(member.span, &member.typing, member_type);
                                    }
                                }

                                resolver.exit();
                                typing = Some(TypeKind::Structure(Aggregate::new(
                                    construct.target.target().unwrap(),
                                    layout.clone(),
                                )));
                            }
                            SymbolKind::Union(mut structure) => {
                                resolver.enter_scope(symbol.scope.clone());

                                for member in &mut structure.members {
                                    if member.is_instance() {
                                        member.resolve(resolver);
                                        layout.push(member.typing.clone());
                                    }
                                }

                                for (index, member) in construct.members.iter_mut().enumerate() {
                                    member.resolve(resolver);

                                    if let Some(member_type) = layout.get(index) {
                                        resolver.unify(member.span, &member.typing, member_type);
                                    }
                                }

                                resolver.exit();
                                typing = Some(TypeKind::Union(Aggregate::new(
                                    construct.target.target().unwrap(),
                                    layout.clone(),
                                )));
                            }
                            _ => {}
                        }
                    }
                }

                let head = construct.target.target().unwrap();
                let aggregate = Aggregate::new(head, layout);

                Type::new(
                    self.reference.unwrap(),
                    typing.unwrap_or(TypeKind::Structure(aggregate)),
                )
            }

            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);
                symbol.resolve(resolver);
                symbol.typing.clone()
            }
        };
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

    fn is_instance(&self) -> bool {
        matches!(
            self.typing.kind,
            TypeKind::Structure(_) | TypeKind::Union(_)
        )
    }
}
