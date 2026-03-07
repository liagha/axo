use {
    super::{
        Resolvable, Resolver,
    },
    crate::{
        parser::{Symbol, SymbolKind},
    },
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(
        &mut self,
        resolver: &mut Resolver<'symbol>,
    ) {
        let mut symbol = self.clone();
        let generic = symbol.generic.clone();

        let id = resolver.next_identity();
        symbol.id = id.clone();

        match &mut symbol.kind {
            SymbolKind::Preference(_) => {}
            SymbolKind::Extension(extension) => {
                let mut scope = resolver.scope.clone();

                if let Ok(mut target) = scope.lookup(&*extension.target) {
                    if let Some(extension) = &extension.extension {
                        if let Ok(found) = scope.lookup(&*extension) {
                            if let SymbolKind::Structure(structure) = found.kind {
                                resolver.scope.remove(&target);
                                target
                                    .scope
                                    .symbols
                                    .extend(structure.members.iter().cloned());
                                target.generic.merge(&generic);
                                resolver.scope.add(target);
                            }
                        }
                    } else {
                        resolver.scope.remove(&target);
                        target
                            .scope
                            .symbols
                            .extend(extension.members.iter().cloned());
                        target.generic.merge(&generic);
                        resolver.scope.add(target);
                    }
                }
            }
            _ => {
            }
        }
    }
}
