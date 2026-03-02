mod binary;
mod invoke;
mod unary;

use {
    super::{
        ErrorKind, Resolution, Resolvable, ResolveError, Resolver,
    },
    crate::{
        data::Boolean,
        parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
        scanner::{OperatorKind, Token, TokenKind},
    },
};
use crate::analyzer::{Analyzable, AnalyzeError, ErrorKind as AnalyzeErrorKind};
use crate::checker::{Checkable, Type};
use crate::data::schema::*;
use self::{binary::resolve_binary, invoke::resolve_invoke, unary::resolve_unary};

impl<'element> Element<'element> {
    pub(crate) fn resolve_path(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Symbol<'element>, Vec<ResolveError<'element>>> {
        match &self.kind {
            ElementKind::Literal(Token {
                kind: TokenKind::Identifier(_),
                ..
            })
            | ElementKind::Construct(_) => resolver.scope.try_get(self),
            ElementKind::Binary(binary) => {
                let is_namespace = match &binary.operator.kind {
                    TokenKind::Operator(operator) => {
                        matches!(operator.as_slice(), [OperatorKind::Dot])
                    }
                    _ => false,
                };

                if !is_namespace {
                    return Err(vec![ResolveError::new(
                        ErrorKind::Analyze {
                            error: AnalyzeError::new(
                                AnalyzeErrorKind::InvalidOperation(binary.operator.clone()),
                                binary.operator.span,
                            ),
                        },
                        binary.operator.span,
                    )]);
                }

                let left = binary.left.resolve_path(resolver)?;
                resolver.enter_scope(left.scope.clone());
                let right = binary.right.resolve_path(resolver);
                resolver.exit();
                if matches!(left.kind, SymbolKind::Module(_)) {
                    if let Ok(ref member) = right {
                        if matches!(member.specifier.visibility, Visibility::Private) {
                            let token = member.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(crate::data::Str::from("<private>")),
                                member.span,
                            ));
                            return Err(vec![ResolveError::new(
                                ErrorKind::PrivateSymbol { symbol: token },
                                member.span,
                            )]);
                        }
                    }
                }
                right
            }
            _ => Err(vec![ResolveError::new(
                ErrorKind::Analyze {
                    error: AnalyzeError::new(
                        AnalyzeErrorKind::InvalidOperation(Token::new(
                            TokenKind::Operator(OperatorKind::Dot),
                            self.span,
                        )),
                        self.span,
                    ),
                },
                self.span,
            )]),
        }
    }

    fn filter_context(
        errors: Vec<ResolveError<'element>>,
    ) -> Result<(), Vec<ResolveError<'element>>> {
        let filtered: Vec<ResolveError<'element>> = errors
            .into_iter()
            .filter(|error| {
                !matches!(
                    &error.kind,
                    ErrorKind::Analyze { error }
                        if matches!(&error.kind, AnalyzeErrorKind::InvalidPrimitiveContext { .. })
                )
            })
            .collect();

        if filtered.is_empty() {
            Ok(())
        } else {
            Err(filtered)
        }
    }

    fn is_addressable(&self) -> bool {
        match &self.kind {
            ElementKind::Literal(Token {
                kind: TokenKind::Identifier(_),
                ..
            }) => true,
            ElementKind::Index(_) => true,
            ElementKind::Binary(binary) => {
                matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot))
            }
            ElementKind::Unary(unary) => {
                matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star))
            }
            _ => false,
        }
    }

    fn inference(&self, resolver: &Resolver<'element>) -> Option<Type<'element>> {
        let token = self.brand()?;

        resolver
            .symbols
            .iter()
            .rev()
            .find_map(|(symbol, inference)| {
                if symbol.brand() == Some(token.clone()) {
                    inference.as_ref().and_then(|item| item.inferred.clone())
                } else {
                    None
                }
            })
    }
}

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(
        &self,
        resolver: &mut Resolver<'element>,
    ) -> Result<Resolution<'element>, Vec<ResolveError<'element>>> {
        let analysis = self.analyze(resolver).map_err(|error| {
            vec![ResolveError::new(
                ErrorKind::Analyze {
                    error: error.clone(),
                },
                error.span,
            )]
        })?;

        match &self.kind {
            ElementKind::Delimited(delimited) => {
                resolver.enter();
                delimited.members.iter().for_each(|item| {
                    item.resolve(resolver);
                });
                resolver.exit();

                let typ = delimited.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(None, typ, analysis))
            }

            ElementKind::Literal(Token {
                kind: TokenKind::Identifier(_),
                ..
            }) => {
                if let Some(typ) = self.inference(resolver) {
                    return Ok(Resolution::new(None, typ, analysis));
                }

                let symbol = resolver.scope.try_get(&self)?;

                let typ = symbol.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(Some(symbol.id), typ, analysis))
            }

            ElementKind::Construct(construct) => {
                let symbol = resolver.scope.try_get(&self)?;

                let typ = symbol.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(Some(symbol.id), typ, analysis))
            }

            ElementKind::Invoke(invoke) => resolve_invoke(self, invoke, resolver, analysis),

            ElementKind::Index(index) => {
                let symbol = resolver.scope.try_get(&self)?;

                let typ = symbol.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(Some(symbol.id), typ, analysis))
            }

            ElementKind::Binary(binary) => resolve_binary(self, binary, resolver, analysis),

            ElementKind::Unary(unary) => resolve_unary(self, unary, resolver, analysis),

            ElementKind::Symbolize(symbol) => symbol.resolve(resolver),

            ElementKind::Literal(_) => {
                let typ = self.infer().map_err(|error| {
                    vec![ResolveError::new(
                        ErrorKind::Check {
                            error: error.clone(),
                        },
                        error.span,
                    )]
                })?;

                Ok(Resolution::new(None, typ, analysis))
            }
        }
    }

    fn is_instance(&self, resolver: &mut Resolver<'element>) -> Boolean {
        match &self.kind {
            ElementKind::Literal(Token {
                kind: TokenKind::Identifier(_),
                ..
            }) => false,
            ElementKind::Unary(Unary { operand, .. }) => operand.is_instance(resolver),
            ElementKind::Binary(Binary {
                left,
                operator:
                    Token {
                        kind: TokenKind::Operator(OperatorKind::Dot),
                        ..
                    },
                ..
            }) => left.is_instance(resolver),
            ElementKind::Symbolize(symbol) => symbol.is_instance(resolver),
            _ => true,
        }
    }
}
