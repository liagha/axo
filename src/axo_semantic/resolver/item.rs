use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, ItemKind};
use crate::axo_semantic::symbol::SymbolKind;
use crate::axo_semantic::{ResolveError, Resolver};
use crate::axo_semantic::error::ErrorKind;
use crate::axo_semantic::resolver::symbol::Symbol;

pub trait ItemResolver {
    fn resolve_item(&mut self, item: ItemKind, span: Span) -> Symbol;
    fn resolve_field(&mut self, expr: Expr) -> Symbol;
    fn resolve_definition(
        &mut self,
        span: Span,
        target: Expr,
        value: Option<Box<Expr>>,
    ) -> Symbol;
    fn resolve_assignment(&mut self, target: Expr, value: Expr, span: Span) -> Symbol;
}

impl ItemResolver for Resolver {
    fn resolve_item(&mut self, item: ItemKind, span: Span) -> Symbol {
        match item {
            ItemKind::Struct { name, body } => self.resolve_struct_item(name, body, span),
            ItemKind::Enum { name, body } => self.resolve_enum_item(name, body, span),
            ItemKind::Function { name, parameters, body } => self.resolve_function_item(name, parameters, body, span),
            ItemKind::Macro { name, parameters, body } => self.resolve_macro_item(name, parameters, body, span),
            ItemKind::Trait(name, body) => self.resolve_trait_item(name, body, span),
            ItemKind::Implement(trait_, target) => self.resolve_impl_item(trait_, target, span),
            ItemKind::Use(path_expr) => self.create_expr_symbol(ExprKind::Item(ItemKind::Use(path_expr)), span),
        }
    }

    fn resolve_field(&mut self, expr: Expr) -> Symbol {
        let Expr { kind, span } = expr.clone();

        match kind {
            ExprKind::Identifier(_) => {
                let kind = SymbolKind::Field {
                    name: expr,
                    field_type: None,
                    default: None,
                };

                Symbol {
                    kind,
                    span,
                }
            }
            ExprKind::Labeled { label: expr, expr: ty } => {
                let kind = SymbolKind::Field {
                    name: *expr,
                    field_type: Some(*ty),
                    default: None,
                };

                Symbol {
                    kind,
                    span,
                }
            }
            ExprKind::Assignment { target, value } => {
                if let Expr { kind: ExprKind::Labeled { label: expr, expr: ty }, .. } = *target {
                    let kind = SymbolKind::Field {
                        name: *expr,
                        field_type: Some(*ty),
                        default: Some(*value),
                    };

                    Symbol {
                        kind,
                        span,
                    }
                } else {
                    let kind = SymbolKind::Field {
                        name: *target,
                        field_type: None,
                        default: Some(*value),
                    };

                    Symbol {
                        kind,
                        span,
                    }
                }
            }
            _ => {
                let error = ResolveError {
                    kind: ErrorKind::InvalidStructField(format!(
                        "{:?}",
                        kind
                    )),
                    span: span.clone(),
                    context: None,
                    help: None,
                    hints: vec![],
                };

                let kind = SymbolKind::Error(error);

                Symbol {
                    kind,
                    span,
                }
            }
        }
    }

    fn resolve_definition(
        &mut self,
        span: Span,
        target: Expr,
        value: Option<Box<Expr>>,
    ) -> Symbol {
        let value = if let Some(val_expr) = value {
            self.resolve_expr(*val_expr.clone());
            Some(*val_expr)
        } else {
            None
        };

        let kind = SymbolKind::Variable {
            name: target,
            value,
            mutable: false,
            ty: None,
        };

        let symbol = Symbol {
            kind,
            span,
        };

        self.insert(symbol.clone());

        symbol
    }

    fn resolve_assignment(
        &mut self,
        target: Expr,
        value: Expr,
        span: Span,
    ) -> Symbol {
        self.validate_assignment_target(&target);

        self.resolve_expr(value.clone());

        self.create_expr_symbol(
            ExprKind::Assignment { target: target.into(), value: value.into() },
            span,
        )
    }
}

impl Resolver {
    fn resolve_struct_item(
        &mut self,
        name: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    ) -> Symbol {
        let mut fields = Vec::new();

        match body.kind {
            ExprKind::Block(block_exprs) => {
                for field_expr in block_exprs {
                    fields.push(self.resolve_field(field_expr));
                }
            }
            _ => fields.push(self.resolve_field(*body))
        };

        let kind = SymbolKind::Struct {
            name: *name,
            fields,
        };

        let symbol = Symbol {
            kind,
            span,
        };

        self.insert(symbol.clone());

        symbol
    }

    fn resolve_enum_item(
        &mut self,
        name: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    ) -> Symbol {
        let mut variants = Vec::new();

        self.process_enum_variants(*body, &mut variants);

        let kind = SymbolKind::Enum {
            name: *name,
            variants,
        };

        let symbol = Symbol {
            kind,
            span
        };

        self.insert(symbol.clone());

        symbol
    }

    fn resolve_function_like_item(
        &mut self,
        span: Span,
        name: Box<Expr>,
        parameters: Vec<Expr>,
        body: Box<Expr>,
        is_macro: bool,
    ) -> Symbol {
        self.with_new_scope(|resolver| {
            for param in &parameters {
                let symbol = resolver.resolve_field(param.clone());
                resolver.insert(symbol);
            }

            resolver.resolve_expr(*body.clone())
        });

        let kind = if is_macro {
            SymbolKind::Macro {
                name: *name,
                parameters,
                body: *body,
            }
        } else {
            SymbolKind::Function {
                name: *name,
                parameters,
                body: *body,
                return_type: None,
            }
        };

        let symbol = Symbol {
            kind,
            span
        };

        self.insert(symbol.clone());

        symbol
    }

    fn resolve_function_item(
        &mut self,
        name: Box<Expr>,
        parameters: Vec<Expr>,
        body: Box<Expr>,
        span: Span,
    ) -> Symbol {
        self.resolve_function_like_item(span, name, parameters, body, false)
    }

    fn resolve_macro_item(
        &mut self,
        name: Box<Expr>,
        parameters: Vec<Expr>,
        body: Box<Expr>,
        span: Span,
    ) -> Symbol {
        self.resolve_function_like_item(span, name, parameters, body, true)
    }

    fn resolve_trait_item(
        &mut self,
        name: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    ) -> Symbol {
        let kind = SymbolKind::Trait {
            name: *name,
            body: *body,
            generic_params: Vec::new(),
        };

        let symbol = Symbol {
            kind,
            span
        };

        self.insert(symbol.clone());

        symbol
    }

    fn resolve_impl_item(
        &mut self,
        trait_: Box<Expr>,
        target: Box<Expr>,
        span: Span,
    ) -> Symbol {
        let trait_symbol = if let Expr { kind: ExprKind::Identifier(trait_name), span } = *trait_.clone() {
            let variable = Symbol {
                kind: SymbolKind::Variable { name: *trait_, value: None, mutable: false, ty: None },
                span,
            };

            let symbol = self.lookup(&variable);

            let Symbol { kind, span } = symbol.clone();

            match kind {
                SymbolKind::Trait { .. } => Some(symbol.clone().into()),
                _ => {
                    return self.error(ErrorKind::TypeMismatch(
                        "Trait".to_string(),
                        "Not a trait".to_string(),
                    ), span)
                }
            }
        } else {
            None
        };

        let kind = SymbolKind::Impl {
            trait_: trait_symbol,
            target: *target,
            body: Expr {
                kind: ExprKind::Block(Vec::new()),
                span: span.clone(),
            },
        };

        let symbol = Symbol {
            kind,
            span
        };

        self.insert(symbol.clone());

        symbol
    }

    fn process_enum_variants(
        &mut self,
        body: Expr,
        variants: &mut Vec<Symbol>,
    ) {
        let Expr { kind, span } = body.clone();

        match kind {
            ExprKind::Block(block_exprs) => {
                for variant_expr in block_exprs {
                    self.resolve_expr(variant_expr);
                }
            }
            ExprKind::Identifier(_) | ExprKind::Invoke { .. } | ExprKind::Struct { .. } => {
                self.resolve_expr(body);
            }
            _ => {
                self.error(ErrorKind::InvalidEnumVariant(format!(
                    "{:?}",
                    body.kind
                )), span);
            }
        }
    }

    fn validate_assignment_target(&mut self, target: &Expr) {
        let Expr { kind, span } = target.clone();

        match kind {
            ExprKind::Identifier(name) => {
                let variable = Symbol {
                    kind: SymbolKind::Variable { name: target.clone(), value: None, mutable: false, ty: None },
                    span,
                };

                let symbol = self.lookup(&variable);

                let Symbol { kind, span } = symbol.clone();

                match kind {
                    SymbolKind::Variable { mutable, .. } => {
                        if !mutable {
                            self.error(ErrorKind::InvalidAssignment, span);
                        }
                    }
                    _ => {
                        self.error(ErrorKind::InvalidAssignment, span);
                    },
                }
            }
            ExprKind::Member { object, member } => {
                self.resolve_expr(*object);
                self.resolve_expr(*member);
            }
            ExprKind::Index { expr, index } => {
                self.resolve_expr(*expr);
                self.resolve_expr(*index);
            }
            _ => {
                self.error(ErrorKind::InvalidAssignment, span);
            },
        }
    }
}