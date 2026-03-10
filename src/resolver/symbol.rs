use {
    super::{
        Resolvable, Resolver,
    },
    crate::{
        parser::{Symbol},
    },
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(
        &mut self,
        _resolver: &mut Resolver<'symbol>,
    ) {

    }
}
