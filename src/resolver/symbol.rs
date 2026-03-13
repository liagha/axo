use {
    super::{Resolvable, Resolver},
    crate::parser::{Symbol, SymbolKind},
};

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(&mut self, resolver: &mut Resolver<'symbol>) {
        resolver.add(self.clone());

        match &mut self.kind {
            SymbolKind::Binding(binding) => {
                if let Some(annotation) = &mut binding.annotation {
                    annotation.resolve(resolver);
                }

                if let Some(value) = &mut binding.value {
                    value.resolve(resolver);
                }
            }
            SymbolKind::Structure(structure) => {
                resolver.enter();

                for member in structure.members.iter_mut() {
                    member.resolve(resolver);
                }

                let mut local = resolver.scope.clone();
                local.parent = None;
                self.scope = local;

                resolver.exit();
            }
            SymbolKind::Union(union) => {
                resolver.enter();

                for member in union.members.iter_mut() {
                    member.resolve(resolver);
                }

                let mut local = resolver.scope.clone();
                local.parent = None;
                self.scope = local;

                resolver.exit();
            }
            SymbolKind::Function(function) => {
                resolver.enter();

                for member in function.members.iter_mut() {
                    member.resolve(resolver);
                }

                if let Some(output) = &mut function.output {
                    output.resolve(resolver);
                }

                if let Some(body) = &mut function.body {
                    body.resolve(resolver);
                }

                let local = resolver.scope.clone();
                self.scope = local;

                resolver.exit();
            }
            SymbolKind::Module(_) => {}
        }

        resolver.add(self.clone());
    }
}
