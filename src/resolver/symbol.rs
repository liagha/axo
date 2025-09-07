use {
    super::{
        Resolver, Resolvable,
    },
    crate::{
        parser::{
            Symbol, SymbolKind,
        },
    }
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(&self, resolver: &mut Resolver<'symbol>) {
        let mut symbol = self.clone();
        symbol.id = resolver.next_id();

        match symbol.kind {
            SymbolKind::Inclusion(_) => {}
            SymbolKind::Preference(_) => {}
            SymbolKind::Extension(extension) => {
                let candidates = resolver.scope.all();

                if let Some(mut target) = resolver.lookup(&*extension.target, &candidates) {
                    if let Some(extension) = extension.extension {
                        if let Some(found) = resolver.lookup(&*extension, &candidates) {
                            if let SymbolKind::Structure(structure) = found.kind {
                                resolver.scope.remove(&target);
                                target.scope.symbols.extend(structure.members.iter().cloned());
                                resolver.scope.add(target);
                            }
                        }
                    } else {
                        resolver.scope.remove(&target);
                        target.scope.symbols.extend(extension.members.iter().cloned());
                        resolver.scope.add(target);
                    }
                }
            }
            _ => {
                resolver.scope.add(symbol);
            }
        }
    }
}