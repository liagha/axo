use crate::{
    data::Str,
    internal::hash::Map,
    parser::{
        Element, ElementKind, Symbol,
    },
};
use crate::resolver::checker::types::TypeKind;
use crate::parser::SymbolKind;
use crate::resolver::checker::{
    types::Type,
    CheckError,
};
use crate::resolver::{ResolveError, Resolver};
use crate::scanner::{PunctuationKind, Token, TokenKind};
use crate::schema::{Index, Invoke, Structure};

pub trait Checkable<'checkable> {
    fn infer(&self) -> Type<'checkable>;
}

impl<'resolver> Resolver<'resolver> {
    pub fn check(&mut self, target: Type<'resolver>, source: Type<'resolver>) {
        if target != source {
            let error = ResolveError::new(
                crate::resolver::ErrorKind::Check { 
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::Mismatch(
                            target, source.clone()
                        ),
                        source.span
                    ),
                },
                source.span
            );
            
            self.errors.push(error);
        }    
    }
}