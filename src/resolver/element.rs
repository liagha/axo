use {
    super::{
        ErrorKind, ResolveError, Resolver,
        scope::Scope,
        checker::Checkable,
        resolver::Resolvable,
        analyzer::Analyzable,
    },
    crate::{
        data::{
            Boolean,
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
        match &self.kind {
            ElementKind::Delimited(delimited) => {
                resolver.enter();
                delimited.items.iter().for_each(|item| item.resolve(resolver));
                resolver.exit();
            }

            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                resolver.get(&self);
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);

                for member in &construct.members {
                    member.resolve(resolver);
                }

                resolver.get(&self);
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                for argument in &invoke.members {
                    argument.resolve(resolver);
                }

                resolver.get(&self);
            }

            ElementKind::Index(index) => {
                index.target.resolve(resolver);

                index.members.iter().for_each(|member| member.resolve(resolver));

                resolver.get(&self);
            }

            ElementKind::Binary(binary) => {
                match binary.operator {
                    Token { kind: TokenKind::Operator(OperatorKind::Dot), .. } => {
                        let scope = binary.left.scope(resolver);

                        resolver.lookup(&binary.right, &scope);
                    }

                    _ => {
                        binary.left.resolve(resolver);
                        binary.right.resolve(resolver);
                    }
                }
            }

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver)
            },

            ElementKind::Closure(closure) => {
                resolver.enter();

                for parameter in &closure.members {
                    parameter.resolve(resolver);
                }

                closure.body.resolve(resolver);

                resolver.exit();
            }

            ElementKind::Symbolize(symbol) => {
                symbol.resolve(resolver);
            }

            ElementKind::Literal(_) => {}
        }

        let analysis = self.analyze(resolver);

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

    fn scope(&self, resolver: &mut Resolver<'element>) -> Scope<'element> {
        match &self.kind {
            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                if let Some(symbol) = resolver.get(self) {
                    symbol.scope(resolver)
                } else {
                    Scope::new()
                }
            }
            ElementKind::Literal(_) => {
                Scope::new()
            }
            ElementKind::Delimited(_) => {
                Scope::new()
            }
            ElementKind::Unary(_) => {
                Scope::new()
            }
            ElementKind::Binary(binary) => {
                match &binary.operator {
                    Token { kind: TokenKind::Operator(OperatorKind::Dot), .. } => {
                        if !binary.left.is_instance(resolver) {
                            if let Some(symbol) = resolver.get(&binary.left) {
                                let scope = symbol.scope(resolver);

                                if let Some(symbol) = resolver.lookup(&binary.right, &scope) {
                                    symbol.scope(resolver)
                                } else {
                                    Scope::new()
                                }
                            } else {
                                Scope::new()
                            }
                        } else {
                            Scope::new()
                        }
                    }

                    _ => {
                        Scope::new()
                    }
                }
            }
            ElementKind::Closure(_) => {
                Scope::new()
            }
            ElementKind::Index(_) => {
                Scope::new()
            }
            ElementKind::Invoke(_) => {
                Scope::new()
            }
            ElementKind::Construct(_) => {
                Scope::new()
            }
            ElementKind::Symbolize(_) => {
                Scope::new()
            }
        }
    }

    fn is_instance(&self, resolver: &mut Resolver<'element>) -> Boolean {
        match &self.kind {
            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                if let Some(symbol) = resolver.get(&self) {
                    symbol.is_instance(resolver)
                } else {
                    false
                }
            }
            ElementKind::Literal(_) => {
                true
            }
            ElementKind::Delimited(_) => {
                true
            }
            ElementKind::Unary(unary) => {
                unary.operand.is_instance(resolver)
            }
            ElementKind::Binary(binary) => {
                match binary.operator {
                    Token { kind: TokenKind::Operator(OperatorKind::Dot), .. } => {
                        binary.left.is_instance(resolver)
                    }
                    _ => {
                        true
                    }
                }
            }
            ElementKind::Closure(_) => {
                true
            }
            ElementKind::Index(_) => {
                true
            }
            ElementKind::Invoke(_) => {
                true
            }
            ElementKind::Construct(_) => {
                true
            }
            ElementKind::Symbolize(symbol) => {
                symbol.is_instance(resolver)
            }
        }
    }
}