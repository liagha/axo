use {
    crate::{
        axo_parser::{
            Element, ElementKind, Item, ItemKind
        },

        axo_resolver::{
            Resolver,
            error::ErrorKind,
        }
    }
};

impl Resolver {
    pub fn validate(&mut self, element: &Element, item: &Item) {
        match (&element.kind, &item.kind) {
            (ElementKind::Invoke { parameters: found, .. }, ItemKind::Function { parameters: expected, .. }) => {
                if found != expected {
                    self.error(ErrorKind::ParameterMismatch {
                        found: found.len(),
                        expected: expected.len(),
                    }, element.span.clone());
                }
            },
            (ElementKind::Invoke { parameters: found, .. }, ItemKind::Macro { parameters: expected, .. }) => {
                if found != expected {
                    self.error(ErrorKind::ParameterMismatch {
                        found: found.len(),
                        expected: expected.len(),
                    }, element.span.clone());
                }
            },
            (ElementKind::Constructor { body, .. }, ItemKind::Structure { fields: expected, .. }) => {
                if let ElementKind::Bundle(found) = &body.kind {
                    if found != expected {
                        self.error(ErrorKind::FieldCountMismatch {
                            found: found.len(),
                            expected: expected.len(),
                        }, element.span.clone());
                    }
                }
            },
            (ElementKind::Identifier(_), ItemKind::Variable { .. }) => {
            },
            (expr_kind, item_kind) => {
                self.error(ErrorKind::TypeMismatch {
                    expected: format!("{:?}", expr_kind),
                    found: format!("{:?}", item_kind),
                }, element.span.clone());
            },
        }
    }
}