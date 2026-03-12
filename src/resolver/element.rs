use {
    super::{
        Resolvable, Resolver,
    },
    crate::{
        parser::{Element, ElementKind},
        scanner::{OperatorKind, Token, TokenKind},
    },
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(
        &mut self,
        resolver: &mut Resolver<'element>,
    ) {
        match &mut self.kind {
            ElementKind::Literal(
                Token {
                    kind: TokenKind::Identifier(_),
                    ..
                })
            => {
                match resolver.scope.lookup(&self) {
                    Ok(symbol) => {
                        self.reference = Some(symbol.identity);
                    }

                    Err(errors) => {
                        resolver.errors.extend(errors);
                    }
                }
            }

            ElementKind::Literal(_) => {}

            ElementKind::Delimited(delimited) => {
                resolver.enter();

                delimited.members.iter_mut().for_each(|item| {
                    item.resolve(resolver);
                });

                resolver.exit();
            }

            ElementKind::Construct(construct) => {
                for member in construct.members.iter_mut() {
                    member.resolve(resolver);
                }

                match resolver.scope.lookup(&self) {
                    Ok(symbol) => {
                        self.reference = Some(symbol.identity);
                    }

                    Err(errors) => {
                        resolver.errors.extend(errors);
                    }
                }
            }

            ElementKind::Invoke(invoke) => {
                for member in invoke.members.iter_mut() {
                    member.resolve(resolver);
                }

                match resolver.scope.lookup(&self) {
                    Ok(symbol) => {
                        self.reference = Some(symbol.identity);
                    }

                    Err(errors) => {
                        resolver.errors.extend(errors);
                    }
                }
            },

            ElementKind::Index(index) => {
                index.target.resolve(resolver);

                index.members.iter_mut().for_each(|member| member.resolve(resolver));
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                match binary.operator.kind {
                    TokenKind::Operator(OperatorKind::Dot) => {
                        if let Some(reference) = binary.left.reference {
                            if let Some(symbol) = resolver.scope.get_identity(reference) {
                                resolver.enter_scope(symbol.scope.clone());

                                binary.right.resolve(resolver);

                                resolver.exit();
                            }
                        }

                        self.reference = binary.right.reference;
                    }

                    _ => {
                        binary.right.resolve(resolver);
                    }
                }
            },

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);
            },

            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);

                resolver.add(symbol.clone());

                symbol.resolve(resolver);
            },
        }
    }
}
