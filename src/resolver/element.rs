use {
    super::{
        ErrorKind, ResolveError, Resolver,
        scope::Scope,
        checker::Checkable,
        resolver::Resolvable,
    },
    crate::{
        data::{
            memory::replace,
        },
        parser::{
            Element, ElementKind,
        },
        scanner::{
            OperatorKind,
            Token, TokenKind,
        },
    }
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(&self, resolver: &mut Resolver<'element>) {
        let Element { kind, .. } = self.clone();

        match kind {
            ElementKind::Delimited(delimited) => {
                resolver.enter();
                delimited.items.iter().for_each(|item| item.resolve(resolver));
                resolver.exit();
            }

            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                resolver.get(&self);
            }

            ElementKind::Construct { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. } => {
                resolver.get(&self);
            }

            ElementKind::Binary(binary) => {
                match binary.operator.kind {
                    TokenKind::Operator(OperatorKind::Equal) => {
                        if let Some(symbol) = resolver.get(&*binary.left) {
                            binary.right.resolve(resolver);

                            let target = symbol.clone().infer();
                            let value = binary.right.infer();

                            resolver.check(target, value);
                        }
                    }

                    TokenKind::Operator(OperatorKind::Dot) => {
                        let candidates = resolver.scope.all();
                        let target = resolver.lookup(&*binary.left, &candidates);

                        if let Some(target) = target {
                            let members = target.scope.all();
                            let member = resolver.lookup(&*binary.right, &members);
                        }
                    }

                    _ => {
                        binary.left.resolve(resolver);
                        binary.right.resolve(resolver);
                    }
                }
            }

            ElementKind::Unary(unary) => unary.operand.resolve(resolver),

            ElementKind::Conditional(conditioned) => {
                conditioned.guard.resolve(resolver);
                resolver.enter();
                conditioned.then.resolve(resolver);
                resolver.exit();

                if let Some(alternate) = conditioned.alternate {
                    resolver.enter();
                    alternate.resolve(resolver);
                    resolver.exit();
                }
            }

            ElementKind::While(repeat) => {
                if let Some(condition) = repeat.guard {
                    condition.resolve(resolver);
                }
                resolver.enter();
                repeat.body.resolve(resolver);
                resolver.exit();
            }

            ElementKind::Cycle(walk) => {
                walk.guard.resolve(resolver);

                let parent = replace(&mut resolver.scope, Scope::new());
                resolver.scope.attach(parent);

                resolver.enter();
                walk.body.resolve(resolver);
                resolver.exit();
            }

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Continue(value) => {
                if let Some(value) = value {
                    value.resolve(resolver);
                }
            }

            ElementKind::Symbolize(_)
            | ElementKind::Literal(_)
            | ElementKind::Procedural(_) => {}
        }

        let analysis = resolver.analyze(self.clone());

        match analysis {
            Ok(analysis) => {
                resolver.output.push(analysis);
            }
            Err(error) => {
                let error = ResolveError::new(ErrorKind::Analyze { error: error.clone() }, error.span);

                resolver.errors.push(error);
            }
        }
    }
}