use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, Item};
use crate::axo_semantic::error::ErrorKind;
use crate::axo_semantic::resolver::scope::Scope;
use crate::axo_semantic::Resolver;

pub trait ControlFlowResolver {
    fn resolve_block(&mut self, exprs: Vec<Expr>, span: Span) -> Item;
    fn resolve_conditional(&mut self, condition: Expr, then_block: Expr, else_block: Option<Expr>, span: Span) -> Item;
    fn resolve_while(&mut self, condition: Expr, body: Expr, span: Span) -> Item;
    fn resolve_for(&mut self, iterator: Expr, body: Expr, span: Span) -> Item;
    fn resolve_match(&mut self, target: Expr, cases: Expr, span: Span) -> Item;
    fn resolve_closure(&mut self, params: Vec<Expr>, body: Expr, span: Span) -> Item;
}

impl ControlFlowResolver for Resolver {
    fn resolve_block(&mut self, exprs: Vec<Expr>, span: Span) -> Item {
        self.with_new_scope(|resolver| {
            resolver.resolve_exprs(&exprs);
        });

        self.create_expr_symbol(ExprKind::Block(exprs), span)
    }

    fn resolve_conditional(
        &mut self,
        condition: Expr,
        then_branch: Expr,
        else_branch: Option<Expr>,
        span: Span,
    ) -> Item {
        self.resolve_expr(condition.clone());
        self.resolve_expr(then_branch.clone());

        let else_branch = if let Some(else_expr) = &else_branch {
            self.resolve_expr(else_expr.clone());
            Some(else_expr.clone().into())
        } else {
            None
        };

        self.create_expr_symbol(
            ExprKind::Conditional {
                condition: condition.into(),
                then_branch: then_branch.into(),
                else_branch,
            },
            span,
        )
    }

    fn resolve_while(&mut self, condition: Expr, body: Expr, span: Span) -> Item {
        self.resolve_expr(condition.clone());
        self.resolve_expr(body.clone());

        self.create_expr_symbol(
            ExprKind::While {
                condition: condition.into(),
                body: body.into()
            },
            span,
        )
    }

    fn resolve_for(&mut self, clause: Expr, body: Expr, span: Span) -> Item {
        self.resolve_expr(clause.clone());
        self.resolve_expr(body.clone());

        self.create_expr_symbol(
            ExprKind::For {
                clause: clause.into(),
                body: body.into(),
            },
            span,
        )
    }

    fn resolve_match(&mut self, target: Expr, body: Expr, span: Span) -> Item {
        self.resolve_expr(target.clone());

        match &body.kind {
            ExprKind::Block(case_exprs) => {
                self.resolve_exprs(case_exprs);
            }
            _ => {
                return self.error(ErrorKind::InvalidExpression(
                    "Expected block for match cases".to_string(),
                ), span)
            }
        }

        self.create_expr_symbol(
            ExprKind::Match {
                target: target.into(),
                body: body.into()
            },
            span,
        )
    }

    fn resolve_closure(&mut self, parameters: Vec<Expr>, body: Expr, span: Span) -> Item {
        self.with_new_scope(|resolver| {
            resolver.resolve_params(&parameters);

            resolver.resolve_expr(body.clone());
        });

        self.create_expr_symbol(
            ExprKind::Closure {
                parameters,
                body: body.into()
            },
            span,
        )
    }
}