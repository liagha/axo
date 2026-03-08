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
        resolver: &mut Resolver<'symbol>,
    ) {
        let mut symbol = self.clone();
        let _generic = symbol.generic.clone();

        let id = resolver.next_identity();
        symbol.id = id.clone();
    }
}
