use {
    super::{Resolvable, Resolver},
    crate::{
        parser::{Element, ElementKind, SymbolKind},
        scanner::{OperatorKind, Token, TokenKind},
    },
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(&mut self, resolver: &mut Resolver<'element>) {
        match &mut self.kind {
            ElementKind::Literal(_) => {}

            ElementKind::Delimited(delimited) => {
                resolver.enter();

                delimited.members.iter_mut().for_each(|member| {
                    member.resolve(resolver);
                });

                resolver.exit();
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);

                for member in construct.members.iter_mut() {
                    member.resolve(resolver);
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                for member in invoke.members.iter_mut() {
                    member.resolve(resolver);
                }
            }

            ElementKind::Index(index) => {
                index.target.resolve(resolver);

                index.members.iter_mut().for_each(|member| member.resolve(resolver));
            }

            ElementKind::Binary(binary) => {
                binary.left.resolve(resolver);

                match binary.operator.kind {
                    TokenKind::Operator(OperatorKind::Dot) => {
                        let mut static_access = false;

                        if let Some(id) = binary.left.reference {
                            if let Some(symbol) = resolver.scope.get_identity(id) {
                                if matches!(
                                    symbol.kind,
                                    SymbolKind::Module(_)
                                        | SymbolKind::Structure(_)
                                        | SymbolKind::Union(_)
                                ) {
                                    static_access = true;

                                    resolver.enter_scope(symbol.scope.clone());
                                    binary.right.resolve(resolver);
                                    resolver.exit();
                                }
                            }
                        }

                        if static_access {
                            self.reference = binary.right.reference;
                        }
                    }

                    _ => {
                        binary.right.resolve(resolver);
                    }
                }
            }

            ElementKind::Unary(unary) => {
                unary.operand.resolve(resolver);
            }

            ElementKind::Symbolize(symbol) => {
                self.reference = Some(symbol.identity);

                symbol.resolve(resolver);
            }
        }

        let mut identity = None;

        match &self.kind {
            ElementKind::Literal(Token {
                                     kind: TokenKind::Identifier(_),
                                     ..
                                 })
            | ElementKind::Construct(_)
            | ElementKind::Invoke(_) => match resolver.scope.lookup(self) {
                Ok(symbol) => {
                    identity = Some(symbol.identity);
                }

                Err(errors) => {
                    resolver.errors.extend(errors);
                }
            },
            _ => {}
        }

        if let Some(id) = identity {
            self.reference = Some(id);

            match &mut self.kind {
                ElementKind::Construct(construct) => {
                    construct.target.reference = Some(id);
                }
                ElementKind::Invoke(invoke) => {
                    invoke.target.reference = Some(id);
                }
                _ => {}
            }
        }
    }
}
