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
            Symbol, Element, ElementKind,
        },
        scanner::{
            OperatorKind,
            Token, TokenKind,
        },
    }
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(&self, resolver: &mut Resolver<'element>) -> Option<Symbol<'element>> {
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

        match &self.kind {
            ElementKind::Delimited(delimited) => {
                resolver.enter();
                delimited.items.iter().for_each(|item| {
                    item.resolve(resolver);
                });
                resolver.exit();

                None
            }

            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                resolver.get(&self)
            }

            ElementKind::Construct(construct) => {
                resolver.get(&self)
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                for argument in &invoke.members {
                    argument.resolve(resolver);
                }

                resolver.get(&self)
            }

            ElementKind::Index(index) => {
                index.target.resolve(resolver);

                index.members.iter().for_each(|member| {
                    member.resolve(resolver);
                });

                resolver.get(&self)
            }

            ElementKind::Binary(binary) => {
                match binary.operator {
                    Token { kind: TokenKind::Operator(OperatorKind::Dot), .. } => {
                        if let Some(symbol) = binary.left.resolve(resolver) {
                            resolver.enter_scope(symbol.scope);

                            let resolved = binary.right.resolve(resolver);

                            resolver.exit();

                            resolved
                        } else {
                            None
                        }
                    }

                    _ => {
                        binary.left.resolve(resolver);
                        binary.right.resolve(resolver);

                        None
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

                None
            }

            ElementKind::Symbolize(symbol) => {
                symbol.resolve(resolver)
            }

            ElementKind::Literal(_) => {
                None
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