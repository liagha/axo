use crate::axo_parser::{Expr, ExprKind, Item, ItemKind};
use crate::axo_semantic::Resolver;
use crate::axo_span::Span;

pub trait Expression {
    fn resolve_assignment(
        &mut self,
        target: Expr,
        value: Expr,
        span: Span,
    ) -> Item;
    fn resolve_invoke(&mut self, target: Expr, parameters: Vec<Expr>) -> Item;
    fn resolve_constructor(&mut self, name: Expr, fields: Box<Expr>) -> Item;
    fn resolve_member(&mut self, target: Expr, member: Expr) -> Item;
    fn resolve_struct(&mut self, name: Expr, fields: Vec<Item>) -> Item;
}

impl Expression for Resolver {
    fn resolve_assignment(
        &mut self,
        target: Expr,
        value: Expr,
        span: Span,
    ) -> Item {
        self.resolve_expr(target.clone());
        self.resolve_expr(value.clone());

        self.create_expr_symbol(
            ExprKind::Assignment { target: target.into(), value: value.into() },
            span,
        )
    }

    fn resolve_invoke(&mut self, target: Expr, parameters: Vec<Expr>) -> Item {
        let symbol = Item {
            kind: ItemKind::Function {
                name: target.clone().into(),
                parameters,
                body: Expr {
                    kind: ExprKind::Block(Vec::new()),
                    span: target.span.clone(),
                }.into(),
            },
            span: target.span,
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_constructor(&mut self, name: Expr, body: Box<Expr>) -> Item {
        let fields = match *body {
            Expr { kind: ExprKind::Block(fields), .. } => {
                fields.iter().map(
                    |field| Item {
                        kind: ItemKind::Expression(field.clone().into()),
                        span: field.span.clone()
                    }
                ).collect::<Vec<_>>()
            },

            _ => {
                let field = Item {
                    kind: ItemKind::Expression(body.clone().into()),
                    span: body.span.clone()
                };

                vec![field]
            },
        };

        let symbol = Item {
            kind: ItemKind::Structure {
                name: name.clone().into(),
                fields
            },
            span: name.span,
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_member(&mut self, target: Expr, _member: Expr) -> Item {
        let symbol = Item {
            kind: ItemKind::Variable {
                target: target.clone().into(),
                value: None,
                mutable: false,
                ty: None,
            },
            span: target.span,
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_struct(&mut self, name: Expr, fields: Vec<Item>) -> Item {
        let symbol = Item {
            kind: ItemKind::Structure {
                name: name.clone().into(),
                fields
            },
            span: name.span,
        };

        let found = self.lookup(&symbol);

        found
    }
}