use {
    super::{
        Resolvable, Resolver,
    },
    crate::{
        parser::{Element, ElementKind, SymbolKind},
        scanner::{OperatorKind, Token, TokenKind},
    },
};

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(
        &mut self,
        resolver: &mut Resolver<'element>,
    ) {
        match &mut self.kind {
            ElementKind::Literal(_) => {}

            ElementKind::Delimited(delimited) => {
                resolver.enter();

                delimited.members.iter_mut().for_each(|item| {
                    item.resolve(resolver);
                });

                resolver.exit();
            }

            ElementKind::Construct(construct) => {
                construct.target.resolve(resolver);

                let mut entered = false;

                if let Some(reference) = construct.target.reference {
                    if let Some(symbol) = resolver.scope.get_identity(reference) {
                        if matches!(symbol.kind, SymbolKind::Structure(_) | SymbolKind::Union(_)) {
                            let mut scope = symbol.scope.clone();
                            scope.parent = Some(Box::new(resolver.scope.clone()));
                            resolver.enter_scope(scope);
                            entered = true;
                        }
                    }
                }

                for member in construct.members.iter_mut() {
                    member.resolve(resolver);
                }

                if entered {
                    resolver.exit();
                }
            }

            ElementKind::Invoke(invoke) => {
                invoke.target.resolve(resolver);

                let mut entered = false;

                // Temporarily inject the function's scope to resolve named arguments.
                if let Some(reference) = invoke.target.reference {
                    if let Some(symbol) = resolver.scope.get_identity(reference) {
                        if matches!(symbol.kind, SymbolKind::Function(_)) {
                            let mut scope = symbol.scope.clone();
                            scope.parent = Some(Box::new(resolver.scope.clone()));
                            resolver.enter_scope(scope);
                            entered = true;
                        }
                    }
                }

                for member in invoke.members.iter_mut() {
                    member.resolve(resolver);
                }

                if entered {
                    resolver.exit();
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
                        let mut is_namespace = false;

                        if let Some(reference) = binary.left.reference {
                            if let Some(symbol) = resolver.scope.get_identity(reference) {
                                match &symbol.kind {
                                    SymbolKind::Module(_) | SymbolKind::Structure(_) | SymbolKind::Union(_) => {
                                        is_namespace = true;

                                        resolver.enter_scope(symbol.scope.clone());

                                        binary.right.resolve(resolver);

                                        resolver.exit();
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if is_namespace {
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
            | ElementKind::Invoke(_) => {
                match resolver.scope.lookup(self) {
                    Ok(symbol) => {
                        identity = Some(symbol.identity);
                    }

                    Err(errors) => {
                        resolver.errors.extend(errors);
                    }
                }
            }
            _ => {}
        }

        if let Some(identity) = identity {
            self.reference = Some(identity);

            match &mut self.kind {
                ElementKind::Construct(construct) => {
                    construct.target.reference = Some(identity);
                }
                ElementKind::Invoke(invoke) => {
                    invoke.target.reference = Some(identity);
                }
                _ => {}
            }
        }
    }
}
