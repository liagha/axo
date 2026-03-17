use crate::{
    data::{Aggregate, Delimited, Scale},
    format::{Show, Verbosity},
    parser::{Element, ElementKind, SymbolKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
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
            ElementKind::Delimited(
                Delimited {
                    start: Token { kind: TokenKind::Punctuation(PunctuationKind::LeftBrace), .. },
                    members,
                    separator: None | Some(Token { kind: TokenKind::Punctuation(PunctuationKind::Semicolon), .. }),
                    end: Token { kind: TokenKind::Punctuation(PunctuationKind::RightBrace), .. },
                }
            ) => {
                for member in members {
                    member.declare(resolver);
                }
            }
            _ => {}
        }
    }

    fn resolve(&mut self, resolver: &mut Resolver<'element>) {
        let span = self.span;

        if matches!(
            &self.kind,
            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. })
                | ElementKind::Construct(_)
                | ElementKind::Invoke(_)
        ) {
            match resolver.scope.lookup(self) {
                Ok(symbol) => {
                    self.reference = Some(symbol.identity);
                    match &mut self.kind {
                        ElementKind::Construct(construct) => construct.target.reference = Some(symbol.identity),
                        ElementKind::Invoke(invoke) => invoke.target.reference = Some(symbol.identity),
                        _ => {}
                    }
                }
                Err(errors) => resolver.errors.extend(errors),
            }
        }

        self.typing = match &mut self.kind {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Type::new(TypeKind::Integer { size: 64, signed: true }),
                TokenKind::Float(_) => Type::new(TypeKind::Float { size: 64 }),
                TokenKind::Boolean(_) => Type::new(TypeKind::Boolean),
                TokenKind::String(_) => Type::new(TypeKind::String),
                TokenKind::Character(_) => Type::new(TypeKind::Character),
                TokenKind::Identifier(_) => {
                    if let Some(reference) = self.reference {
                        resolver.lookup(reference)
                    } else {
                        resolver.fresh()
                    }
                }
                _ => Type::void(),
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

                        Type::new(TypeKind::Tuple { members })
                    }
                }

                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    resolver.enter();

                    let mut block = Type::new(TypeKind::Void);

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

                    Type::new(
                        TypeKind::Array {
                            member: Box::new(inner),
                            size: delimited.members.len() as Scale,
                        },
                    )
                }

                _ => Type::void(),
            },

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);

                match &unary.operator.kind {
                    TokenKind::Operator(operator) => match operator.as_slice() {
                        [OperatorKind::Exclamation] => {
                            resolver.unify(span, &unary.operand.typing, &Type::new(TypeKind::Boolean))
                        },
                        [OperatorKind::Tilde] => {
                            let expect = resolver.reify(&unary.operand.typing);

                            if !matches!(expect.kind, TypeKind::Integer { .. } | TypeKind::Variable(_)) {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            }

                            unary.operand.typing.clone()
                        }
                        [OperatorKind::Plus] | [OperatorKind::Minus] => unary.operand.typing.clone(),
                        [OperatorKind::Ampersand] => {
                            if assignable(&unary.operand) {
                                Type::new(TypeKind::Pointer { target: Box::new(unary.operand.typing.clone()) })
                            } else {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                                resolver.fresh()
                            }
                        }
                        [OperatorKind::Star] => {
                            let target = resolver.fresh();
                            let pointer = Type::new(TypeKind::Pointer { target: Box::new(target.clone()) });

                            resolver.unify(span, &unary.operand.typing, &pointer);

                            target
                        }
                        _ => {
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            resolver.fresh()
                        }
                    },
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                match &binary.operator.kind {
                    TokenKind::Operator(operator) => match operator.as_slice() {
                        [OperatorKind::Dot] => {
                            let mut scope = None;
                            let mut instance = false;
                            let mut enumeration = None;

                            if let Some(symbol) = binary.left.reference.and_then(|reference| resolver.scope.find(reference)) {
                                match &symbol.kind {
                                    SymbolKind::Module(_) | SymbolKind::Structure(_) | SymbolKind::Union(_) => {
                                        scope = Some(symbol.scope.clone());
                                    }
                                    SymbolKind::Enumeration(_) => {
                                        scope = Some(symbol.scope.clone());
                                        instance = true;
                                        enumeration = Some(symbol.identity);
                                    }
                                    _ => {}
                                }
                            }

                            if scope.is_none() {
                                let mut left = resolver.reify(&binary.left.typing);

                                while let TypeKind::Pointer { target } = left.kind {
                                    left = *target;
                                }

                                match left.kind {
                                    TypeKind::Structure(reference, _) | TypeKind::Union(reference, _) | TypeKind::Enumeration(reference, _) => {
                                        if let Some(symbol) = resolver.scope.find(reference) {
                                            scope = Some(symbol.scope.clone());
                                            instance = true;
                                            if matches!(symbol.kind, SymbolKind::Enumeration(_)) {
                                                enumeration = Some(reference);
                                            }
                                        }
                                    }
                                    TypeKind::Constructor(reference, _) => {
                                        if let Some(symbol) = resolver.scope.find(reference) {
                                            scope = Some(symbol.scope.clone());
                                            if matches!(symbol.kind, SymbolKind::Enumeration(_)) {
                                                instance = true;
                                                enumeration = Some(reference);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(environment) = scope {
                                resolver.enter_scope(environment);
                                binary.right.resolve(resolver);

                                if let Some(member) = binary.right.reference.and_then(|reference| resolver.scope.find(reference)) {
                                    if !instance && member.is_instance() {
                                        resolver.errors.push(Error::new(
                                            ErrorKind::InvalidOperation(binary.operator.clone()),
                                            binary.right.span,
                                        ));
                                        binary.right.typing = resolver.fresh();
                                    }
                                }

                                resolver.exit();
                            } else {
                                binary.right.resolve(resolver);
                            }

                            self.reference = binary.right.reference;

                            if let Some(reference) = enumeration {
                                if let Some(member) = binary.right.reference.and_then(|value| resolver.scope.find(value)) {
                                    if member.is_instance() {
                                        Type::new(TypeKind::Enumeration(reference, Aggregate::new(crate::data::Str::default(), Vec::new())))
                                    } else {
                                        binary.right.typing.clone()
                                    }
                                } else {
                                    binary.right.typing.clone()
                                }
                            } else {
                                binary.right.typing.clone()
                            }
                        }
                        operators => {
                            binary.right.resolve(resolver);

                            match operators {
                                [OperatorKind::Equal] => {
                                    resolver.unify(span, &binary.left.typing, &binary.right.typing)
                                },
                                [OperatorKind::Plus]
                                | [OperatorKind::Minus]
                                | [OperatorKind::Star]
                                | [OperatorKind::Slash]
                                | [OperatorKind::Percent] => {
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
                                        resolver.unify(span, &left, &right)
                                    } else {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                        resolver.fresh()
                                    }
                                }
                                [OperatorKind::LeftAngle, OperatorKind::LeftAngle] | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                                    let left = resolver.reify(&binary.left.typing);
                                    let right = resolver.reify(&binary.right.typing);

                                    let is_valid = |typing: &Type| matches!(typing.kind, TypeKind::Integer { .. } | TypeKind::Variable(_) | TypeKind::Unknown);

                                    if is_valid(&left) && is_valid(&right) {
                                        resolver.unify(span, &left, &right)
                                    } else {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                        resolver.fresh()
                                    }
                                }
                                [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] => {
                                    let left = resolver.reify(&binary.left.typing);
                                    let right = resolver.reify(&binary.right.typing);
                                    let is_valid = |typing: &Type| matches!(typing.kind, TypeKind::Integer { .. } | TypeKind::Boolean | TypeKind::Variable(_) | TypeKind::Unknown);

                                    if is_valid(&left) && is_valid(&right) {
                                        resolver.unify(span, &binary.left.typing, &binary.right.typing)
                                    } else {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                        resolver.fresh()
                                    }
                                }
                                [OperatorKind::Ampersand, OperatorKind::Ampersand] | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                                    let boolean = Type::new(TypeKind::Boolean);

                                    resolver.unify(span, &binary.left.typing, &boolean);
                                    resolver.unify(span, &binary.right.typing, &boolean);

                                    boolean
                                }
                                [OperatorKind::Equal, OperatorKind::Equal]
                                | [OperatorKind::Exclamation, OperatorKind::Equal]
                                | [OperatorKind::LeftAngle]
                                | [OperatorKind::LeftAngle, OperatorKind::Equal]
                                | [OperatorKind::RightAngle]
                                | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                                    let merged = resolver.unify(span, &binary.left.typing, &binary.right.typing);
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
                                            | TypeKind::Enumeration(_, _)
                                            | TypeKind::Unknown
                                    );

                                    if !is_valid {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                    }

                                    Type::new(TypeKind::Boolean)
                                }
                                _ => {
                                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                                    resolver.fresh()
                                }
                            }
                        }
                    },
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                        resolver.fresh()
                    }
                }
            }

            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                    resolver.fresh()
                } else {
                    index.target.resolve(resolver);
                    index.members[0].resolve(resolver);

                    let target = resolver.reify(&index.target.typing);
                    let member = resolver.reify(&index.members[0].typing);

                    let expect = Type::new(TypeKind::Integer { size: 64, signed: true });

                    resolver.unify(span, &member, &expect);

                    match target.kind {
                        TypeKind::Pointer { target: base } => {
                            if let TypeKind::Array { member, .. } = base.kind {
                                *member
                            } else {
                                *base
                            }
                        }
                        TypeKind::Array { member, .. } => {
                            *member
                        }
                        TypeKind::Tuple { members } => {
                            let mut value = None;

                            if let ElementKind::Literal(Token { kind: TokenKind::Integer(literal), .. }) = index.members[0].kind {
                                value = usize::try_from(literal).ok();
                            }

                            if let Some(position) = value.filter(|&position| position < members.len()) {
                                members[position].clone()
                            } else {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                                resolver.fresh()
                            }
                        }
                        TypeKind::Variable(_) => {
                            let element = resolver.fresh();
                            let pointer = Type::new(TypeKind::Pointer { target: Box::new(element.clone()) });

                            resolver.unify(span, &target, &pointer);

                            element
                        }
                        _ => {
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                            resolver.fresh()
                        }
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                let primitive = invoke.target.brand().and_then(|brand| match &brand.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                }).and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        if invoke.members.len() < 2 {
                            Type::void()
                        } else {
                            resolver.enter();
                            invoke.members[0].resolve(resolver);
                            let boolean = Type::new(TypeKind::Boolean);
                            resolver.unify(invoke.members[0].span, &invoke.members[0].typing, &boolean);

                            invoke.members[1].resolve(resolver);
                            let then = invoke.members[1].typing.clone();

                            let typing = if invoke.members.len() == 3 {
                                invoke.members[2].resolve(resolver);
                                resolver.unify(span, &then, &invoke.members[2].typing)
                            } else {
                                let void = Type::new(TypeKind::Void);
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
                            let boolean = Type::new(TypeKind::Boolean);
                            resolver.unify(invoke.members[0].span, &invoke.members[0].typing, &boolean);
                        }
                        if invoke.members.len() > 1 {
                            resolver.enter();
                            invoke.members[1].resolve(resolver);
                            resolver.exit();
                        }
                        Type::void()
                    }
                    Some("return") => {
                        if !invoke.members.is_empty() {
                            invoke.members[0].resolve(resolver);
                        }

                        let value = invoke.members.first().map_or_else(|| Type::new(TypeKind::Void), |member| member.typing.clone());

                        if let Some(expect) = resolver.returns.last().cloned() {
                            resolver.unify(span, &expect, &value);
                        } else {
                            let token = invoke.target.brand().unwrap().clone();
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(token), span));
                        }
                        Type::new(TypeKind::Unknown)
                    }
                    Some("continue") | Some("break") => {
                        Type::new(TypeKind::Unknown)
                    }
                    _ => {
                        for member in &mut invoke.members {
                            member.resolve(resolver);
                        }

                        let output = resolver.fresh();
                        let mut arguments = Vec::new();

                        if let ElementKind::Binary(binary) = &invoke.target.kind {
                            if let TokenKind::Operator(operator) = &binary.operator.kind {
                                if operator.as_slice() == [OperatorKind::Dot] {
                                    let receiver = resolver.reify(&binary.left.typing);
                                    if !matches!(receiver.kind, TypeKind::Void | TypeKind::Constructor(_, _)) {
                                        arguments.push(binary.left.typing.clone());
                                    }
                                }
                            }
                        }

                        arguments.extend(invoke.members.iter().map(|member| member.typing.clone()));

                        let function = Type::new(TypeKind::Function(crate::data::Str::default(), arguments, Some(Box::new(output.clone()))));
                        let merged = resolver.unify(span, &invoke.target.typing, &function);

                        match merged.kind {
                            TypeKind::Function(_, _, Some(kind)) => *kind,
                            TypeKind::Function(_, _, None) => Type::new(TypeKind::Void),
                            _ => output,
                        }
                    }
                }
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);

                let mut component = 0;
                let mut scope = None;
                let mut is_union = false;
                let mut is_enumeration = false;

                if let TypeKind::Constructor(identity, _) = &construct.target.typing.kind {
                    if let Some(symbol) = resolver.scope.find(*identity) {
                        if let SymbolKind::Enumeration(_) = &symbol.kind {
                            is_enumeration = true;
                            component = *identity;
                        }
                    }
                }

                let mut layout = Vec::new();

                if let Some(reference) = construct.target.reference {
                    if !is_enumeration {
                        component = reference;
                    }
                    if let Some(symbol) = resolver.scope.find(reference) {
                        match &symbol.kind {
                            SymbolKind::Structure(structure) => {
                                scope = Some(symbol.scope.clone());
                                for member in &structure.members {
                                    if member.is_instance() {
                                        layout.push(member.typing.clone());
                                    }
                                }
                            }
                            SymbolKind::Union(union_symbol) => {
                                scope = Some(symbol.scope.clone());
                                is_union = true;
                                for member in &union_symbol.members {
                                    if member.is_instance() {
                                        layout.push(member.typing.clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(environment) = scope {
                    resolver.enter_scope(environment);
                } else {
                    resolver.enter();
                }

                if is_union {
                    if construct.members.len() != 1 {
                        let token = construct.target.brand().cloned().unwrap_or_else(|| Token::new(TokenKind::Identifier(crate::data::Str::default()), span));
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(token), span));
                    }
                    for member in &mut construct.members {
                        member.resolve(resolver);
                    }
                    if let Some(member) = construct.members.first() {
                        let mut is_valid = false;
                        let actual = resolver.reify(&member.typing);
                        for expect in &layout {
                            let check = resolver.reify(expect);
                            if actual == check {
                                resolver.unify(member.span, &member.typing, expect);
                                is_valid = true;
                                break;
                            }
                        }
                        if !is_valid && !layout.is_empty() {
                            resolver.unify(member.span, &member.typing, &layout[0]);
                        }
                    }
                } else if is_enumeration {
                    for member in &mut construct.members {
                        member.resolve(resolver);
                    }
                } else {
                    if construct.members.len() != layout.len() {
                        let token = construct.target.brand().cloned().unwrap_or_else(|| Token::new(TokenKind::Identifier(crate::data::Str::default()), span));
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(token), span));
                    }
                    for (index, member) in construct.members.iter_mut().enumerate() {
                        member.resolve(resolver);
                        if let Some(expect) = layout.get(index) {
                            resolver.unify(member.span, &member.typing, expect);
                        }
                    }
                }

                resolver.exit();

                let head = construct.target.brand().map_or_else(crate::data::Str::default, |brand| brand.format(Verbosity::Minimal).into());
                let aggregate = Aggregate::new(head, layout);

                if is_enumeration {
                    Type::new(TypeKind::Enumeration(component, aggregate))
                } else if is_union {
                    Type::new(TypeKind::Union(component, aggregate))
                } else {
                    Type::new(TypeKind::Structure(component, aggregate))
                }
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
            TypeKind::Structure(_, _) | TypeKind::Union(_, _) | TypeKind::Enumeration(_, _)
        )
    }
}
