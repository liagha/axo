use {
    crate::{
        axo_parser::{
            Expr, ExprKind, Item, ItemKind
        },

        axo_resolver::{
            error::ErrorKind,
        }
    }
};

use crate::axo_resolver::Resolver;

impl Resolver {
    /// Validates an expression against a resolved item
    pub fn validate(&mut self, expr: &Expr, item: &Item) {
        match (&expr.kind, &item.kind) {
            (ExprKind::Invoke { parameters, .. }, ItemKind::Function { parameters: func_params, .. }) => {
                self.error(ErrorKind::ParameterMismatch {
                    found: parameters.len(),
                    expected: func_params.len(),
                }, expr.span.clone());
            },
            (ExprKind::Invoke { parameters, .. }, ItemKind::Macro { parameters: macro_params, .. }) => {
                self.error(ErrorKind::ParameterMismatch {
                    found: parameters.len(),
                    expected: macro_params.len(),
                }, expr.span.clone());
            },
            (ExprKind::Constructor { body, .. }, ItemKind::Structure { fields, .. }) => {
                if let ExprKind::Bundle(exprs) = &body.kind {
                    self.error(ErrorKind::FieldCountMismatch {
                        found: exprs.len(),
                        expected: fields.len(),
                    }, expr.span.clone());
                }
            },
            (ExprKind::Constructor { .. }, ItemKind::Enum { .. }) => {
                // Enums don't require field validation as they may have variants
            },
            (ExprKind::Identifier(_), ItemKind::Variable { .. }) => {
                // Variables don't need additional validation beyond type checking
            },
            (expr_kind, item_kind) => {
                self.error(ErrorKind::TypeMismatch {
                    expected: format!("{:?}", expr_kind),
                    found: format!("{:?}", item_kind),
                }, expr.span.clone());
            },
        }
    }
}