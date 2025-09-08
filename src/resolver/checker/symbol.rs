use {
    crate::{
        data::Str,
        parser::{Symbol, SymbolKind},
        resolver::{
            checker::{
                Checkable,
                Type, TypeKind,
            },
        },
        schema::*,
    },
};

impl<'symbol> Checkable<'symbol> for Symbol<'symbol> {
    fn infer(&self) -> Type<'symbol> {
        match &self.kind {
            SymbolKind::Inclusion(_) => {
                Type::unit(self.span)
            }
            SymbolKind::Extension(_) => {
                Type::unit(self.span)
            }
            SymbolKind::Binding(binding) => {
                if let Some(annotation) = &binding.annotation {
                    Type::unit(self.span)
                } else if let Some(value) = &binding.value {
                    value.infer()
                } else {
                    Type::unit(self.span)
                }
            }
            SymbolKind::Structure(structure) => {
                let structure = Structure::new(
                    Str::from(structure.target.brand().unwrap().to_string()),
                    structure.members
                        .iter()
                        .map(|field| {
                            Box::new(field.clone().infer())
                        })
                        .collect::<Vec<_>>(),
                );

                Type::new(
                    TypeKind::Structure(
                        structure,
                    ),
                    self.span
                )
            }
            SymbolKind::Enumeration(_) => {
                Type::unit(self.span)
            }
            SymbolKind::Method(_) => {
                Type::unit(self.span)
            }
            SymbolKind::Module(_) => {
                Type::unit(self.span)
            }
            SymbolKind::Preference(_) => {
                Type::unit(self.span)
            }
        }
    }
}