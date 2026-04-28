use crate::{
    data::{Binding, BindingKind, Delimited, Function, Interface, Scale, Str},
    parser::{Element, ElementKind, SymbolKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Spanned,
};

impl<'a> Element<'a> {
    fn assignable(&self) -> bool {
        match &self.kind {
            ElementKind::Literal(token) => matches!(token.kind, TokenKind::Identifier(_)),
            ElementKind::Index(index) => index.target.assignable(),
            ElementKind::Binary(binary) => {
                matches!(
                    &binary.operator.kind,
                    TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Dot]
                ) && binary.left.assignable()
            }
            ElementKind::Unary(unary) => {
                matches!(
                    &unary.operator.kind,
                    TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Star]
                )
            }
            _ => false,
        }
    }

    fn combine(mut members: Vec<Type<'a>>) -> Option<Type<'a>> {
        let mut current = members.pop()?;

        while let Some(next) = members.pop() {
            current = Type::from(TypeKind::And(Box::new(next), Box::new(current)));
        }

        Some(current)
    }

    fn name(typing: &Type<'a>) -> Option<Str<'a>> {
        match &typing.kind {
            TypeKind::Binding(binding) => Some(binding.target),
            TypeKind::Function(function) if !function.target.is_empty() => Some(function.target),
            TypeKind::Has(target) => Self::name(target),
            _ => None,
        }
    }

    fn link(
        resolver: &mut Resolver<'a>,
        kind: &ElementKind<'a>,
        span: crate::tracker::Span,
        reference: &mut Option<crate::data::Identity>,
    ) {
        if let Some(token) = kind.try_unwrap_literal() {
            if matches!(token.kind, TokenKind::Identifier(_)) {
                let element = Element {
                    identity: 0,
                    kind: kind.clone(),
                    span,
                    reference: None,
                    typing: Type::from(TypeKind::Unknown),
                };
                match resolver.lookup(&element) {
                    Ok(symbol) => *reference = Some(symbol.identity),
                    Err(errors) => resolver.errors.extend(errors),
                }
            }
        }
    }

    fn binding(typing: Type<'a>, name: Str<'a>) -> Type<'a> {
        Type::from(TypeKind::Binding(Box::new(Binding::new(
            name,
            Some(Box::new(typing)),
            None,
            BindingKind::Let,
        ))))
    }

    fn has(typing: Type<'a>, name: Str<'a>) -> Type<'a> {
        Type::from(TypeKind::Has(Box::new(Self::binding(typing, name))))
    }

    fn invalid_unary(
        resolver: &mut Resolver<'a>,
        operator: Token<'a>,
        operand: Type<'a>,
    ) -> Type<'a> {
        resolver.errors.push(Error::new(
            ErrorKind::InvalidUnary(operator.clone(), operand),
            operator.span,
        ));
        resolver.fresh()
    }

    fn invalid_binary(
        resolver: &mut Resolver<'a>,
        operator: Token<'a>,
        left: Type<'a>,
        right: Type<'a>,
        span: crate::tracker::Span,
    ) -> Type<'a> {
        resolver.errors.push(Error::new(
            ErrorKind::InvalidBinary(operator, left, right),
            span,
        ));
        resolver.fresh()
    }

    fn literal(
        resolver: &mut Resolver<'a>,
        current: Type<'a>,
        reference: Option<crate::data::Identity>,
        literal: &Token<'a>,
    ) -> Type<'a> {
        match literal.kind {
            TokenKind::Integer(_) => Type::from(TypeKind::Integer {
                size: 64,
                signed: true,
            }),
            TokenKind::Float(_) => Type::from(TypeKind::Float { size: 64 }),
            TokenKind::Boolean(_) => Type::from(TypeKind::Boolean),
            TokenKind::String(_) => Type::from(TypeKind::String),
            TokenKind::Character(_) => Type::from(TypeKind::Character),
            TokenKind::Identifier(_) => match reference
                .and_then(|identity| resolver.get_symbol(identity).cloned())
            {
                Some(symbol) => {
                    let typing = resolver.reify(&symbol.typing);
                    match typing.kind {
                        TypeKind::Binding(binding) => match (binding.value, binding.annotation) {
                            (Some(value), _) => *value,
                            (None, Some(annotation)) => *annotation,
                            (None, None) => Type::from(TypeKind::Unknown),
                        },
                        _ => typing,
                    }
                }
                None => current,
            },
            _ => Type::from(TypeKind::Void),
        }
    }

    fn group(
        resolver: &mut Resolver<'a>,
        _span: crate::tracker::Span,
        delimited: &mut Delimited<Token<'a>, Element<'a>>,
    ) -> Type<'a> {
        match (
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
                    let members = delimited
                        .members
                        .iter_mut()
                        .map(|member| {
                            member.resolve(resolver);
                            member.typing.clone()
                        })
                        .collect();

                    Type::from(TypeKind::Tuple {
                        members: Box::new(members),
                    })
                }
            }
            (
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                TokenKind::Punctuation(PunctuationKind::RightBrace),
            ) => {
                let (typing, _) = resolver.nest(|resolver| {
                    let last = delimited.members.len().saturating_sub(1);
                    let mut block = Type::from(TypeKind::Void);

                    for (index, member) in delimited.members.iter_mut().enumerate() {
                        member.resolve(resolver);
                        if index == last {
                            block = member.typing.clone();
                        }
                    }

                    block
                });

                typing
            }
            (
                TokenKind::Punctuation(PunctuationKind::LeftBracket),
                None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                TokenKind::Punctuation(PunctuationKind::RightBracket),
            ) => {
                let mut member = resolver.fresh();

                for item in &mut delimited.members {
                    item.resolve(resolver);
                    member = resolver.unify(item.span, &member, &item.typing);
                }

                Type::from(TypeKind::Array {
                    member: Box::new(member),
                    size: delimited.members.len() as Scale,
                })
            }
            _ => Type::from(TypeKind::Void),
        }
    }

    fn unary(
        resolver: &mut Resolver<'a>,
        unary: &mut crate::data::Unary<Token<'a>, Element<'a>>,
    ) -> Type<'a> {
        unary.operand.resolve(resolver);

        match &unary.operator.kind {
            TokenKind::Operator(operator) => match operator.as_slice() {
                [OperatorKind::Exclamation] => resolver.unify(
                    unary.operand.span,
                    &unary.operand.typing,
                    &Type::from(TypeKind::Boolean),
                ),
                [OperatorKind::Tilde] => {
                    let typing = resolver.reify(&unary.operand.typing);
                    if !typing.kind.is_integer() && !typing.kind.is_variable() {
                        Self::invalid_unary(resolver, unary.operator.clone(), typing)
                    } else {
                        unary.operand.typing.clone()
                    }
                }
                [OperatorKind::Plus] | [OperatorKind::Minus] => unary.operand.typing.clone(),
                [OperatorKind::Ampersand] => {
                    if unary.operand.assignable() {
                        Type::new(
                            unary.operand.typing.identity,
                            TypeKind::Pointer {
                                target: Box::new(unary.operand.typing.clone()),
                            },
                        )
                    } else {
                        Self::invalid_unary(
                            resolver,
                            unary.operator.clone(),
                            unary.operand.typing.clone(),
                        )
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
                _ => Self::invalid_unary(
                    resolver,
                    unary.operator.clone(),
                    unary.operand.typing.clone(),
                ),
            },
            _ => Self::invalid_unary(
                resolver,
                unary.operator.clone(),
                unary.operand.typing.clone(),
            ),
        }
    }

    fn access(
        resolver: &mut Resolver<'a>,
        reference: &mut Option<crate::data::Identity>,
        left: &mut Element<'a>,
        operator: Token<'a>,
        right: &mut Element<'a>,
    ) -> Type<'a> {
        let mut typing = resolver.reify(&left.typing);

        while typing.kind.is_pointer() {
            typing = resolver.reify(&typing.kind.unwrap_pointer());
        }

        let scope = left
            .reference
            .and_then(|reference| resolver.get_symbol(reference).cloned())
            .filter(|symbol| !symbol.is_instance())
            .map(|symbol| symbol.scope)
            .or_else(|| {
                typing
                    .kind
                    .is_module()
                    .then(|| {
                        resolver
                            .get_symbol(typing.identity)
                            .cloned()
                            .map(|symbol| symbol.scope)
                    })
                    .flatten()
            });

        if let Some(scope) = scope {
            let (_, _) = resolver.within(*scope, |resolver| {
                right.resolve(resolver);
            });
            *reference = right.reference;
            return right.typing.clone();
        }

        match right.target() {
            Some(name) => {
                let member = resolver.fresh();
                let unified = resolver.unify(
                    right.span,
                    &typing,
                    &Self::has(member.clone(), name.clone()),
                );
                left.typing = unified;

                if let Some(symbol) = resolver.get_symbol(typing.identity).cloned() {
                    let (_, _) = resolver.within(*symbol.scope.clone(), |resolver| {
                        if let Ok(found) = resolver.lookup(right) {
                            right.reference = Some(found.identity);
                            *reference = Some(found.identity);
                        }
                    });
                }

                member
            }
            None => {
                right.resolve(resolver);
                Self::invalid_binary(resolver, operator, typing, right.typing.clone(), right.span)
            }
        }
    }

    fn math(
        resolver: &mut Resolver<'a>,
        operator: Token<'a>,
        left: Type<'a>,
        right: Type<'a>,
        span: crate::tracker::Span,
    ) -> Type<'a> {
        let valid = |kind: &TypeKind| {
            kind.is_integer()
                || kind.is_float()
                || kind.is_pointer()
                || kind.is_variable()
                || kind.is_unknown()
        };

        if valid(&left.kind) && valid(&right.kind) {
            resolver.unify(span, &left, &right)
        } else {
            Self::invalid_binary(resolver, operator, left, right, span)
        }
    }

    fn bits(
        resolver: &mut Resolver<'a>,
        operator: Token<'a>,
        left: Type<'a>,
        right: Type<'a>,
        span: crate::tracker::Span,
    ) -> Type<'a> {
        let valid = |kind: &TypeKind| {
            kind.is_integer() || kind.is_boolean() || kind.is_variable() || kind.is_unknown()
        };

        if valid(&left.kind) && valid(&right.kind) {
            resolver.unify(span, &left, &right)
        } else {
            Self::invalid_binary(resolver, operator, left, right, span)
        }
    }

    fn shifts(
        resolver: &mut Resolver<'a>,
        operator: Token<'a>,
        left: Type<'a>,
        right: Type<'a>,
        span: crate::tracker::Span,
    ) -> Type<'a> {
        let valid = |kind: &TypeKind| kind.is_integer() || kind.is_variable() || kind.is_unknown();

        if valid(&left.kind) && valid(&right.kind) {
            resolver.unify(span, &left, &right)
        } else {
            Self::invalid_binary(resolver, operator, left, right, span)
        }
    }

    fn compare(
        resolver: &mut Resolver<'a>,
        operator: Token<'a>,
        left: Type<'a>,
        right: Type<'a>,
        span: crate::tracker::Span,
    ) -> Type<'a> {
        let merged = resolver.unify(span, &left, &right);
        let merged = resolver.reify(&merged);

        let valid = merged.kind.is_integer()
            || merged.kind.is_float()
            || merged.kind.is_boolean()
            || merged.kind.is_character()
            || merged.kind.is_string()
            || merged.kind.is_pointer()
            || merged.kind.is_variable()
            || merged.kind.is_unknown();

        if !valid {
            resolver.errors.push(Error::new(
                ErrorKind::InvalidBinary(operator, left, right),
                span,
            ));
        }

        Type::from(TypeKind::Boolean)
    }

    fn binary(
        resolver: &mut Resolver<'a>,
        reference: &mut Option<crate::data::Identity>,
        binary: &mut crate::data::Binary<Element<'a>, Token<'a>, Element<'a>>,
    ) -> Type<'a> {
        binary.left.resolve(resolver);

        match &binary.operator.kind {
            TokenKind::Operator(operator) => match operator.as_slice() {
                [OperatorKind::Dot] => Self::access(
                    resolver,
                    reference,
                    &mut binary.left,
                    binary.operator.clone(),
                    &mut binary.right,
                ),
                [OperatorKind::Equal] => {
                    binary.right.resolve(resolver);
                    resolver.unify(binary.right.span, &binary.left.typing, &binary.right.typing)
                }
                [OperatorKind::Plus]
                | [OperatorKind::Minus]
                | [OperatorKind::Star]
                | [OperatorKind::Slash]
                | [OperatorKind::Percent] => {
                    binary.right.resolve(resolver);
                    let left = resolver.reify(&binary.left.typing);
                    let right = resolver.reify(&binary.right.typing);
                    Self::math(
                        resolver,
                        binary.operator.clone(),
                        left,
                        right,
                        binary.right.span,
                    )
                }
                [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                    binary.right.resolve(resolver);
                    let left = resolver.reify(&binary.left.typing);
                    let right = resolver.reify(&binary.right.typing);
                    Self::shifts(
                        resolver,
                        binary.operator.clone(),
                        left,
                        right,
                        binary.right.span,
                    )
                }
                [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] => {
                    binary.right.resolve(resolver);
                    let left = resolver.reify(&binary.left.typing);
                    let right = resolver.reify(&binary.right.typing);
                    Self::bits(
                        resolver,
                        binary.operator.clone(),
                        left,
                        right,
                        binary.right.span,
                    )
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
                    Self::compare(
                        resolver,
                        binary.operator.clone(),
                        binary.left.typing.clone(),
                        binary.right.typing.clone(),
                        binary.right.span,
                    )
                }
                _ => {
                    binary.right.resolve(resolver);
                    Self::invalid_binary(
                        resolver,
                        binary.operator.clone(),
                        binary.left.typing.clone(),
                        binary.right.typing.clone(),
                        binary.operator.span,
                    )
                }
            },
            _ => {
                binary.right.resolve(resolver);
                Self::invalid_binary(
                    resolver,
                    binary.operator.clone(),
                    binary.left.typing.clone(),
                    binary.right.typing.clone(),
                    binary.operator.span,
                )
            }
        }
    }

    fn index(
        resolver: &mut Resolver<'a>,
        span: crate::tracker::Span,
        index: &mut crate::data::Index<Element<'a>, Element<'a>>,
    ) -> Type<'a> {
        if index.members.is_empty() {
            resolver
                .errors
                .push(Error::new(ErrorKind::EmptyIndex, span));
            return resolver.fresh();
        }

        index.target.resolve(resolver);
        index.members[0].resolve(resolver);

        let target = resolver.reify(&index.target.typing);
        let member = resolver.reify(&index.members[0].typing);
        let expect = Type::from(TypeKind::Integer {
            size: 64,
            signed: true,
        });

        resolver.unify(index.members.span(), &member, &expect);

        if target.kind.is_pointer() {
            let base = target.kind.unwrap_pointer();
            if base.kind.is_array() {
                *base.kind.unwrap_array().0
            } else {
                *base
            }
        } else if target.kind.is_array() {
            *target.kind.unwrap_array().0
        } else if target.kind.is_tuple() {
            let members = target.kind.unwrap_tuple();
            let value = index.members[0]
                .kind
                .try_unwrap_literal()
                .and_then(|token| match &token.kind {
                    TokenKind::Integer(value) => usize::try_from(*value).ok(),
                    _ => None,
                })
                .unwrap_or(0);

            if value < members.len() {
                members[value].clone()
            } else {
                resolver.errors.push(Error::new(
                    ErrorKind::IndexBounds(value, members.len()),
                    index.members.span(),
                ));
                resolver.fresh()
            }
        } else if target.kind.is_variable() {
            let element = resolver.fresh();
            let pointer = Type::new(
                element.identity,
                TypeKind::Pointer {
                    target: Box::new(element.clone()),
                },
            );

            resolver.unify(index.members.span(), &target, &pointer);
            element
        } else {
            resolver
                .errors
                .push(Error::new(ErrorKind::Unindexable, index.target.span));
            resolver.fresh()
        }
    }

    fn invoke(
        resolver: &mut Resolver<'a>,
        span: crate::tracker::Span,
        reference: &mut Option<crate::data::Identity>,
        invoke: &mut crate::data::Invoke<Element<'a>, Element<'a>>,
    ) -> Type<'a> {
        match invoke.target.target().and_then(|name| name.as_str()) {
            Some("if") => {
                if invoke.members.len() < 2 {
                    return Type::from(TypeKind::Void);
                }

                let (typing, _) = resolver.nest(|resolver| {
                    invoke.members[0].resolve(resolver);
                    let boolean = Type::from(TypeKind::Boolean);
                    resolver.unify(invoke.members[0].span, &invoke.members[0].typing, &boolean);

                    invoke.members[1].resolve(resolver);
                    let then = invoke.members[1].typing.clone();

                    if invoke.members.len() == 3 {
                        invoke.members[2].resolve(resolver);
                        resolver.unify(invoke.members[2].span, &then, &invoke.members[2].typing)
                    } else {
                        let void = Type::from(TypeKind::Void);
                        resolver.unify(invoke.members[1].span, &then, &void);
                        void
                    }
                });

                typing
            }
            Some("while") => {
                if !invoke.members.is_empty() {
                    invoke.members[0].resolve(resolver);
                    let boolean = Type::from(TypeKind::Boolean);
                    resolver.unify(invoke.members[0].span, &invoke.members[0].typing, &boolean);
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
                    resolver.unify(span, &expect, &value);
                }

                Type::from(TypeKind::Unknown)
            }
            Some("continue") | Some("break") => Type::from(TypeKind::Unknown),
            _ => {
                for member in &mut invoke.members {
                    member.resolve(resolver);
                }

                let output = resolver.fresh();
                let expected = Type::from(TypeKind::Function(Box::new(Function::new(
                    Str::default(),
                    invoke
                        .members
                        .iter()
                        .map(|member| member.typing.clone())
                        .collect(),
                    resolver.fresh(),
                    Some(Box::new(output.clone())),
                    Interface::Axo,
                    false,
                    false,
                ))));

                let selected = resolver
                    .candidates(&invoke.target)
                    .into_iter()
                    .find(|symbol| {
                        matches!(symbol.kind, SymbolKind::Function(_)) && {
                            let mut trial = resolver.clone();
                            let before = trial.errors.len();
                            let _ = trial.unify(span, &symbol.typing, &expected);
                            trial.errors.len() == before
                        }
                    });

                if let Some(symbol) = selected {
                    invoke.target.reference = Some(symbol.identity);
                    invoke.target.typing = symbol.typing.clone();
                    *reference = Some(symbol.identity);
                } else {
                    invoke.target.resolve(resolver);
                    *reference = invoke.target.reference;
                }

                let function = resolver.unify(span, &invoke.target.typing, &expected);

                if function.kind.is_function() {
                    function
                        .kind
                        .unwrap_function()
                        .output
                        .map(|kind| *kind)
                        .unwrap_or_else(|| Type::from(TypeKind::Void))
                } else {
                    output
                }
            }
        }
    }

    fn construct(
        resolver: &mut Resolver<'a>,
        span: crate::tracker::Span,
        reference: &mut Option<crate::data::Identity>,
        construct: &mut crate::data::Aggregate<Element<'a>, Element<'a>>,
    ) -> Type<'a> {
        let mut members = Vec::with_capacity(construct.members.len());

        for member in &mut construct.members {
            match &mut member.kind {
                ElementKind::Binary(binary)
                    if matches!(
                        &binary.operator.kind,
                        TokenKind::Operator(operator) if operator.as_slice() == [OperatorKind::Equal]
                    ) && binary.left.target().is_some() =>
                {
                    binary.right.resolve(resolver);
                    members.push((binary.left.target(), binary.right.typing.clone()));
                }
                _ => {
                    member.resolve(resolver);
                    members.push((None, member.typing.clone()));
                }
            }
        }

        let build = |layout: Option<&Vec<Type<'a>>>| {
            let members = members
                .iter()
                .enumerate()
                .map(|(index, (name, typing))| {
                    let label = name.clone().or_else(|| {
                        layout
                            .and_then(|layout| layout.get(index))
                            .and_then(Self::name)
                    });

                    match label {
                        Some(name) => Self::has(typing.clone(), name),
                        None => typing.clone(),
                    }
                })
                .collect::<Vec<_>>();

            Self::combine(members)
        };

        let selected = resolver
            .candidates(&construct.target)
            .into_iter()
            .find(|symbol| {
                let layout = match &symbol.typing.kind {
                    TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => {
                        Some(&aggregate.members)
                    }
                    _ => None,
                };

                let Some(expect) = build(layout) else {
                    return true;
                };

                let mut trial = resolver.clone();
                let before = trial.errors.len();
                let _ = trial.unify(span, &expect, &symbol.typing);
                trial.errors.len() == before
            });

        if let Some(symbol) = selected {
            construct.target.reference = Some(symbol.identity);
            construct.target.typing = symbol.typing.clone();
            *reference = Some(symbol.identity);
        } else {
            construct.target.resolve(resolver);
            *reference = construct.target.reference;
        }

        let layout = match &construct.target.typing.kind {
            TypeKind::Structure(aggregate) | TypeKind::Union(aggregate) => Some(&aggregate.members),
            _ => None,
        };

        match build(layout) {
            Some(expect) => resolver.unify(span, &expect, &construct.target.typing),
            None => construct.target.typing.clone(),
        }
    }
}

impl<'a> Resolvable<'a> for Element<'a> {
    fn declare(&mut self, resolver: &mut Resolver<'a>) {
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

    fn resolve(&mut self, resolver: &mut Resolver<'a>) {
        if self.kind.is_literal() {
            Self::link(resolver, &self.kind, self.span, &mut self.reference);
        }

        self.typing = match &mut self.kind {
            ElementKind::Literal(literal) => {
                Self::literal(resolver, self.typing.clone(), self.reference, literal)
            }
            ElementKind::Delimited(delimited) => Self::group(resolver, self.span, delimited),
            ElementKind::Unary(unary) => Self::unary(resolver, unary),
            ElementKind::Binary(binary) => Self::binary(resolver, &mut self.reference, binary),
            ElementKind::Index(index) => Self::index(resolver, self.span, index),
            ElementKind::Invoke(invoke) => {
                Self::invoke(resolver, self.span, &mut self.reference, invoke)
            }
            ElementKind::Construct(construct) => {
                Self::construct(resolver, self.span, &mut self.reference, construct)
            }
            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);
                symbol.resolve(resolver);
                symbol.typing.clone()
            }
        };
    }

    fn is_instance(&self) -> bool {
        self.typing.kind.is_structure() || self.typing.kind.is_union()
    }
}
