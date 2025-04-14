use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, Item, ItemKind};
use crate::axo_semantic::Resolver;

pub trait Expression {
    fn resolve_assignment(
        &mut self,
        target: Expr,
        value: Expr,
        span: Span,
    ) -> Item;
    fn resolve_invoke(&mut self, target: Expr, parameters: Vec<Expr>) -> Item;
    fn resolve_member(&mut self, target: Expr, member: Expr) -> Item;
    fn resolve_struct(&mut self, name: Expr, body: Expr) -> Item;
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
                name: target.into(),
                parameters,
                body: Expr::dummy().into(),
            },
            span: Span::zero()
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_member(&mut self, target: Expr, member: Expr) -> Item {
        let symbol = Item {
            kind: ItemKind::Variable {
                target: target.into(),
                value: None,
                mutable: false,
                ty: None,
            },
            span: Span::zero()
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_struct(&mut self, name: Expr, body: Expr) -> Item {
        let symbol = Item {
            kind: ItemKind::Struct {
                name: name.into(),
                body: body.into()
            },
            span: Span::zero()
        };

        let found = self.lookup(&symbol);

        found
    }
}