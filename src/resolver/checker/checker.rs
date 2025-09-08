use {
    crate::{
        resolver::{
            ResolveError, Resolver,
            checker::{
                CheckError,
                types::{Type, TypeKind},
            },
        },
        scanner::{PunctuationKind, Token, TokenKind},
        schema::*,
        data::Str,
        internal::hash::Map,
        parser::{Element, ElementKind, Symbol, SymbolKind},
    },
};

pub trait Checkable<'checkable> {
    fn infer(&self) -> Type<'checkable>;
}

impl<'resolver> Resolver<'resolver> {
    pub fn check(&mut self, target: Type<'resolver>, source: Type<'resolver>) {
        if target != source {
            let error = ResolveError::new(
                crate::resolver::ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::Mismatch(target, source.clone()),
                        source.span,
                    ),
                },
                source.span,
            );

            self.errors.push(error);
        }
    }
}
