use {
    crate::{
        resolver::{
            ResolveError, Resolver,
            checker::{
                CheckError,
                types::{Type},
            },
        },
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
