use {
    super::{
        Resolvable, Resolver,
    },
    crate::{
        parser::{Symbol},
    },
};
use crate::parser::SymbolKind;

impl<'symbol> Resolvable<'symbol> for Symbol<'symbol> {
    fn resolve(
        &mut self,
        resolver: &mut Resolver<'symbol>,
    ) {
        // Register the symbol initially so it is accessible recursively if needed
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

                // Capture the populated scope before exiting
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();
            }
            SymbolKind::Union(union) => {
                resolver.enter();

                for member in union.members.iter_mut() {
                    member.resolve(resolver);
                }

                // Capture the populated scope before exiting
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

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

                // Capture the populated scope before exiting
                let mut scope = resolver.scope.clone();
                scope.parent = None;
                self.scope = scope;

                resolver.exit();
            }
            SymbolKind::Module(_) => {}
        }

        // Re-add the finalized symbol to the resolver so that outer scopes
        // receive the version with the fully populated `self.scope`.
        resolver.add(self.clone());
    }
}
