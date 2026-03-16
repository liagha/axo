use crate::{
    data::{Aggregate, Scale},
    format::Show,
    parser::{Element, ElementKind, SymbolKind},
    resolver::{Error, ErrorKind, Resolvable, Resolver, Type, TypeKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
};

fn lvalue(node: &Element) -> bool {
    match &node.kind {
        ElementKind::Literal(token) => matches!(token.kind, TokenKind::Identifier(_)),
        ElementKind::Index(index) => lvalue(&index.target),
        ElementKind::Binary(binary) => {
            if let TokenKind::Operator(op) = &binary.operator.kind {
                op.as_slice() == [OperatorKind::Dot] && lvalue(&binary.left)
            } else {
                false
            }
        }
        ElementKind::Unary(unary) => {
            if let TokenKind::Operator(op) = &unary.operator.kind {
                op.as_slice() == [OperatorKind::Star]
            } else {
                false
            }
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
            ElementKind::Construct(construct) => {
                construct.target.declare(resolver);
                
                for member in &mut construct.members {
                    member.declare(resolver);
                }
            }
            ElementKind::Binary(binary) => {
                binary.left.declare(resolver);
                binary.right.declare(resolver);
            }
            ElementKind::Delimited(delimited) => {
                for member in delimited.members.iter_mut() {
                    member.declare(resolver);
                }
            }
            _ => {}
        }
    }

    fn resolve(&mut self, resolver: &mut Resolver<'element>) {
        let span = self.span;
        let mut id = None;

        if matches!(
            &self.kind,
            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. })
                | ElementKind::Construct(_)
                | ElementKind::Invoke(_)
        ) {
            match resolver.scope.lookup(self) {
                Ok(sym) => id = Some(sym.identity),
                Err(errs) => resolver.errors.extend(errs),
            }
        }

        if let Some(ref_id) = id {
            self.reference = Some(ref_id);
            match &mut self.kind {
                ElementKind::Construct(cons) => cons.target.reference = Some(ref_id),
                ElementKind::Invoke(inv) => inv.target.reference = Some(ref_id),
                _ => {}
            }
        }

        let typing = match &mut self.kind {
            ElementKind::Literal(lit) => match lit.kind {
                TokenKind::Integer(_) => Type::new(TypeKind::Integer { size: 64, signed: true }, lit.span),
                TokenKind::Float(_) => Type::new(TypeKind::Float { size: 64 }, lit.span),
                TokenKind::Boolean(_) => Type::new(TypeKind::Boolean, lit.span),
                TokenKind::String(_) => Type::new(TypeKind::String, lit.span),
                TokenKind::Character(_) => Type::new(TypeKind::Character, lit.span),
                TokenKind::Identifier(_) => {
                    if let Some(ref_id) = self.reference {
                        resolver.lookup(ref_id, lit.span)
                    } else {
                        resolver.fresh(lit.span)
                    }
                }
                _ => Type::void(lit.span),
            },

            ElementKind::Delimited(delim) => match (
                &delim.start.kind,
                delim.separator.as_ref().map(|tok| &tok.kind),
                &delim.end.kind,
            ) {
                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    None | Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delim.separator.is_none() && delim.members.len() == 1 {
                        delim.members[0].resolve(resolver);
                        delim.members[0].typing.clone()
                    } else {
                        let mut members = Vec::with_capacity(delim.members.len());
                        for member in &mut delim.members {
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
                    let last = delim.members.len().saturating_sub(1);
                    for (idx, member) in delim.members.iter_mut().enumerate() {
                        member.resolve(resolver);
                        if idx == last {
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
                    for member in &mut delim.members {
                        member.resolve(resolver);
                        inner = resolver.unify(member.span, &inner, &member.typing);
                    }
                    Type::new(
                        TypeKind::Array {
                            member: Box::new(inner),
                            size: delim.members.len() as Scale,
                        },
                        span,
                    )
                }

                _ => Type::void(span),
            },

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);

                match &unary.operator.kind {
                    TokenKind::Operator(op) => match op.as_slice() {
                        [OperatorKind::Exclamation] => resolver.unify(span, &unary.operand.typing, &Type::new(TypeKind::Boolean, span)),
                        [OperatorKind::Tilde] => {
                            let expect = resolver.reify(&unary.operand.typing);
                            if !matches!(expect.kind, TypeKind::Integer { .. } | TypeKind::Variable(_)) {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            }
                            unary.operand.typing.clone()
                        }
                        [OperatorKind::Plus] | [OperatorKind::Minus] => unary.operand.typing.clone(),
                        [OperatorKind::Ampersand] => {
                            if lvalue(&unary.operand) {
                                Type::new(TypeKind::Pointer { target: Box::new(unary.operand.typing.clone()) }, span)
                            } else {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                                resolver.fresh(span)
                            }
                        }
                        [OperatorKind::Star] => {
                            let target = resolver.fresh(span);
                            let ptr = Type::new(TypeKind::Pointer { target: Box::new(target.clone()) }, span);
                            resolver.unify(span, &unary.operand.typing, &ptr);
                            target
                        }
                        _ => {
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                            resolver.fresh(span)
                        }
                    },
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(unary.operator.clone()), unary.operator.span));
                        resolver.fresh(span)
                    }
                }
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                match &binary.operator.kind {
                    TokenKind::Operator(op) => match op.as_slice() {
                        [OperatorKind::Dot] => {
                            let mut scope = None;
                            let mut inst = false;

                            if let Some(sym) = binary.left.reference.and_then(|ref_id| resolver.scope.find(ref_id)) {
                                match &sym.kind {
                                    SymbolKind::Module(_) | SymbolKind::Structure(_) | SymbolKind::Union(_) => {
                                        scope = Some(sym.scope.clone());
                                    }
                                    SymbolKind::Enumeration(_) => {
                                        scope = Some(sym.scope.clone());
                                        inst = true;
                                    }
                                    _ => {}
                                }
                            }

                            if scope.is_none() {
                                let mut left = resolver.reify(&binary.left.typing);

                                loop {
                                    match left.kind {
                                        TypeKind::Pointer { target } => {
                                            left = *target;
                                        }
                                        _ => break,
                                    }
                                }

                                match left.kind {
                                    TypeKind::Structure(ref_id, _) | TypeKind::Union(ref_id, _) | TypeKind::Enumeration(ref_id, _) => {
                                        if let Some(sym) = resolver.scope.find(ref_id) {
                                            scope = Some(sym.scope.clone());
                                            inst = true;
                                        }
                                    }
                                    TypeKind::Constructor(ref_id, _) => {
                                        if let Some(sym) = resolver.scope.find(ref_id) {
                                            scope = Some(sym.scope.clone());
                                            if let SymbolKind::Enumeration(_) = sym.kind {
                                                inst = true;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(env) = scope {
                                resolver.enter_scope(env);
                                binary.right.resolve(resolver);

                                if let Some(member) = binary.right.reference.and_then(|ref_id| resolver.scope.find(ref_id)) {
                                    if !inst && member.is_instance() {
                                        resolver.errors.push(Error::new(
                                            ErrorKind::InvalidOperation(binary.operator.clone()),
                                            binary.right.span,
                                        ));
                                        binary.right.typing = resolver.fresh(binary.right.span);
                                    }
                                }

                                resolver.exit();
                            } else {
                                binary.right.resolve(resolver);
                            }

                            self.reference = binary.right.reference;
                            binary.right.typing.clone()
                        }
                        ops => {
                            binary.right.resolve(resolver);

                            match ops {
                                [OperatorKind::Equal] => resolver.unify(span, &binary.left.typing, &binary.right.typing),
                                [OperatorKind::Plus]
                                | [OperatorKind::Minus]
                                | [OperatorKind::Star]
                                | [OperatorKind::Slash]
                                | [OperatorKind::Percent] => {
                                    let left = resolver.reify(&binary.left.typing);
                                    let right = resolver.reify(&binary.right.typing);
                                    let valid = |t: &Type| {
                                        matches!(
                                            t.kind,
                                            TypeKind::Integer { .. }
                                                | TypeKind::Float { .. }
                                                | TypeKind::Pointer { .. }
                                                | TypeKind::Variable(_)
                                                | TypeKind::Unknown
                                        )
                                    };
                                    if valid(&left) && valid(&right) {
                                        resolver.unify(span, &left, &right)
                                    } else {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                        resolver.fresh(span)
                                    }
                                }
                                [OperatorKind::LeftAngle, OperatorKind::LeftAngle] | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                                    let left = resolver.reify(&binary.left.typing);
                                    let right = resolver.reify(&binary.right.typing);
                                    let valid = |t: &Type| matches!(t.kind, TypeKind::Integer { .. } | TypeKind::Variable(_) | TypeKind::Unknown);

                                    if valid(&left) && valid(&right) {
                                        resolver.unify(span, &left, &right)
                                    } else {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                        resolver.fresh(span)
                                    }
                                }
                                [OperatorKind::Ampersand] | [OperatorKind::Pipe] | [OperatorKind::Caret] => {
                                    let left = resolver.reify(&binary.left.typing);
                                    let right = resolver.reify(&binary.right.typing);
                                    let valid = |t: &Type| matches!(t.kind, TypeKind::Integer { .. } | TypeKind::Boolean | TypeKind::Variable(_) | TypeKind::Unknown);

                                    if valid(&left) && valid(&right) {
                                        resolver.unify(span, &binary.left.typing, &binary.right.typing)
                                    } else {
                                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), span));
                                        resolver.fresh(span)
                                    }
                                }
                                [OperatorKind::Ampersand, OperatorKind::Ampersand] | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                                    let boolean = Type::new(TypeKind::Boolean, span);
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
                                    let valid = matches!(
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
                    },
                    _ => {
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(binary.operator.clone()), binary.operator.span));
                        resolver.fresh(span)
                    }
                }
            }

            ElementKind::Index(idx) => {
                if idx.members.is_empty() {
                    resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                    resolver.fresh(span)
                } else {
                    idx.target.resolve(resolver);
                    idx.members[0].resolve(resolver);

                    let target = resolver.reify(&idx.target.typing);
                    let param = resolver.reify(&idx.members[0].typing);

                    match target.kind {
                        TypeKind::Pointer { target: base } => {
                            let expect = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                            resolver.unify(span, &param, &expect);
                            if let TypeKind::Array { member, .. } = base.kind {
                                *member
                            } else {
                                *base
                            }
                        }
                        TypeKind::Array { member, .. } => {
                            let expect = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                            resolver.unify(span, &param, &expect);
                            *member
                        }
                        TypeKind::Tuple { members } => {
                            let mut val = None;
                            if let ElementKind::Literal(Token { kind: TokenKind::Integer(lit), .. }) = idx.members[0].kind {
                                val = usize::try_from(lit).ok();
                            }
                            if let Some(pos) = val.filter(|&p| p < members.len()) {
                                members[pos].clone()
                            } else {
                                resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                                resolver.fresh(span)
                            }
                        }
                        TypeKind::Variable(_) => {
                            let expect = Type::new(TypeKind::Integer { size: 64, signed: true }, span);
                            resolver.unify(span, &param, &expect);
                            let elem = resolver.fresh(span);
                            let ptr = Type::new(TypeKind::Pointer { target: Box::new(elem.clone()) }, span);
                            resolver.unify(span, &target, &ptr);
                            elem
                        }
                        TypeKind::Unknown => Type::new(TypeKind::Unknown, span),
                        _ => {
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(Token::new(TokenKind::Punctuation(PunctuationKind::LeftBracket), span)), span));
                            resolver.fresh(span)
                        }
                    }
                }
            }

            ElementKind::Invoke(inv) => {
                inv.target.resolve(resolver);

                let prim = inv.target.brand().and_then(|brand| match &brand.kind {
                    TokenKind::Identifier(name) => Some(name),
                    _ => None,
                }).and_then(|name| name.as_str());

                match prim {
                    Some("if") => {
                        if inv.members.len() < 2 {
                            Type::void(span)
                        } else {
                            resolver.enter();
                            inv.members[0].resolve(resolver);
                            let boolean = Type::new(TypeKind::Boolean, span);
                            resolver.unify(inv.members[0].span, &inv.members[0].typing, &boolean);

                            inv.members[1].resolve(resolver);
                            let then = inv.members[1].typing.clone();

                            let typing = if inv.members.len() == 3 {
                                inv.members[2].resolve(resolver);
                                resolver.unify(span, &then, &inv.members[2].typing)
                            } else {
                                let void = Type::new(TypeKind::Void, span);
                                resolver.unify(inv.members[1].span, &then, &void);
                                void
                            };

                            resolver.exit();
                            typing
                        }
                    }
                    Some("while") => {
                        if !inv.members.is_empty() {
                            inv.members[0].resolve(resolver);
                            let boolean = Type::new(TypeKind::Boolean, span);
                            resolver.unify(inv.members[0].span, &inv.members[0].typing, &boolean);
                        }
                        if inv.members.len() > 1 {
                            resolver.enter();
                            inv.members[1].resolve(resolver);
                            resolver.exit();
                        }
                        Type::void(span)
                    }
                    Some("return") => {
                        if !inv.members.is_empty() {
                            inv.members[0].resolve(resolver);
                        }

                        let val = inv.members.first().map_or_else(|| Type::new(TypeKind::Void, span), |m| m.typing.clone());

                        if let Some(expect) = resolver.returns.last().cloned() {
                            resolver.unify(span, &expect, &val);
                        } else {
                            let token = inv.target.brand().unwrap().clone();
                            resolver.errors.push(Error::new(ErrorKind::InvalidOperation(token), span));
                        }
                        Type::new(TypeKind::Unknown, span)
                    }
                    Some("continue") | Some("break") => {
                        Type::new(TypeKind::Unknown, span)
                    }
                    _ => {
                        for member in &mut inv.members {
                            member.resolve(resolver);
                        }

                        let out = resolver.fresh(span);
                        let mut args = Vec::new();

                        if let ElementKind::Binary(bin) = &inv.target.kind {
                            if let TokenKind::Operator(op) = &bin.operator.kind {
                                if op.as_slice() == [OperatorKind::Dot] {
                                    let recv = resolver.reify(&bin.left.typing);
                                    if !matches!(recv.kind, TypeKind::Void | TypeKind::Constructor(_, _)) {
                                        args.push(bin.left.typing.clone());
                                    }
                                }
                            }
                        }

                        args.extend(inv.members.iter().map(|member| member.typing.clone()));

                        let func = Type::new(TypeKind::Function(crate::data::Str::default(), args, Some(Box::new(out.clone()))), span);
                        let merged = resolver.unify(span, &inv.target.typing, &func);

                        match merged.kind {
                            TypeKind::Function(_, _, Some(kind)) => *kind,
                            TypeKind::Function(_, _, None) => Type::new(TypeKind::Void, span),
                            _ => out,
                        }
                    }
                }
            }

            ElementKind::Construct(cons) => {
                cons.target.resolve(resolver);

                let mut comp = 0;
                let mut scope = None;
                let mut is_union = false;
                let mut is_enum = false;

                if let TypeKind::Constructor(id, _) = &cons.target.typing.kind {
                    if let Some(sym) = resolver.scope.find(*id) {
                        if let SymbolKind::Enumeration(_) = &sym.kind {
                            is_enum = true;
                            comp = *id;
                        }
                    }
                }

                let mut layout = Vec::new();

                if let Some(ref_id) = cons.target.reference {
                    if !is_enum {
                        comp = ref_id;
                    }
                    if let Some(sym) = resolver.scope.find(ref_id) {
                        match &sym.kind {
                            SymbolKind::Structure(struc) => {
                                scope = Some(sym.scope.clone());
                                for member in &struc.members {
                                    if member.is_instance() {
                                        layout.push(member.typing.clone());
                                    }
                                }
                            }
                            SymbolKind::Union(un) => {
                                scope = Some(sym.scope.clone());
                                is_union = true;
                                for member in &un.members {
                                    if member.is_instance() {
                                        layout.push(member.typing.clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(env) = scope {
                    resolver.enter_scope(env);
                } else {
                    resolver.enter();
                }

                if is_union {
                    if cons.members.len() != 1 {
                        let tok = cons.target.brand().cloned().unwrap_or_else(|| Token::new(TokenKind::Identifier(crate::data::Str::default()), span));
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(tok), span));
                    }
                    for member in &mut cons.members {
                        member.resolve(resolver);
                    }
                    if let Some(member) = cons.members.first() {
                        let mut valid = false;
                        let actual = resolver.reify(&member.typing);
                        for expect in &layout {
                            let check = resolver.reify(expect);
                            if actual == check {
                                resolver.unify(member.span, &member.typing, expect);
                                valid = true;
                                break;
                            }
                        }
                        if !valid && !layout.is_empty() {
                            resolver.unify(member.span, &member.typing, &layout[0]);
                        }
                    }
                } else if is_enum {
                    for member in &mut cons.members {
                        member.resolve(resolver);
                    }
                } else {
                    if cons.members.len() != layout.len() {
                        let tok = cons.target.brand().cloned().unwrap_or_else(|| Token::new(TokenKind::Identifier(crate::data::Str::default()), span));
                        resolver.errors.push(Error::new(ErrorKind::InvalidOperation(tok), span));
                    }
                    for (idx, member) in cons.members.iter_mut().enumerate() {
                        member.resolve(resolver);
                        if let Some(expect) = layout.get(idx) {
                            resolver.unify(member.span, &member.typing, expect);
                        }
                    }
                }

                resolver.exit();

                let head = cons.target.brand().map_or_else(crate::data::Str::default, |brand| brand.format(0).into());
                let struc = Aggregate::new(head, layout);

                if is_enum {
                    Type::new(TypeKind::Enumeration(comp, struc), span)
                } else if is_union {
                    Type::new(TypeKind::Union(comp, struc), span)
                } else {
                    Type::new(TypeKind::Structure(comp, struc), span)
                }
            }

            ElementKind::Symbolize(sym) => {
                self.reference = Some(sym.identity);
                sym.resolve(resolver);
                sym.typing.clone()
            }
        };

        self.typing = typing;
    }

    fn reify(&mut self, resolver: &mut Resolver<'element>) {
        self.typing = resolver.reify(&self.typing);

        match &mut self.kind {
            ElementKind::Literal(_) => {}
            ElementKind::Delimited(delim) => {
                for member in &mut delim.members {
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
            ElementKind::Index(idx) => {
                idx.target.reify(resolver);
                for member in &mut idx.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Invoke(inv) => {
                inv.target.reify(resolver);
                for member in &mut inv.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Construct(cons) => {
                cons.target.reify(resolver);
                for member in &mut cons.members {
                    member.reify(resolver);
                }
            }
            ElementKind::Symbolize(sym) => {
                sym.reify(resolver);
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
