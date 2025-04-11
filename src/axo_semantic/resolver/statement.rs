use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind};
use crate::axo_semantic::resolver::entity::Entity;
use crate::axo_semantic::resolver::error::ResolverError;
use crate::axo_semantic::resolver::scope::Scope;
use crate::axo_semantic::Resolver;

/// Trait for resolving control flow expressions
pub trait ControlFlowResolver {
    fn resolve_block(&mut self, exprs: Vec<Expr>, span: Span) -> Result<Entity, ResolverError>;

    fn resolve_conditional(
        &mut self,
        condition: Expr,
        then_block: Expr,
        else_block: Option<Expr>,
        span: Span,
    ) -> Result<Entity, ResolverError>;

    fn resolve_while(
        &mut self,
        condition: Expr,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError>;

    fn resolve_for(
        &mut self,
        iterator: Expr,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError>;

    fn resolve_match(
        &mut self,
        target: Expr,
        cases: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError>;

    fn resolve_closure(
        &mut self,
        params: Vec<Expr>,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError>;
}

impl ControlFlowResolver for Resolver {
    fn resolve_block(&mut self, exprs: Vec<Expr>, span: Span) -> Result<Entity, ResolverError> {
        let current_scope = self.scope.clone();

        let old_scope = std::mem::replace(
            &mut self.scope,
            Scope::with_parent(current_scope.clone()),
        );

        for expr in &exprs {
            let _ = self.resolve_expr(expr.clone())?;
        }

        self.scope = old_scope;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Block(exprs),
            span,
        }))
    }

    fn resolve_conditional(
        &mut self,
        condition: Expr,
        then_branch: Expr,
        else_branch: Option<Expr>,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let _ = self.resolve_expr(condition.clone())?;
        let _ = self.resolve_expr(then_branch.clone())?;

        let else_branch = if let Some(else_expr) = &else_branch {
            let _ = self.resolve_expr(else_expr.clone())?;
            Some(else_expr.clone().into())
        } else {
            None
        };

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Conditional {
                condition: condition.into(),
                then_branch: then_branch.into(),
                else_branch,
            },
            span,
        }))
    }

    fn resolve_while(
        &mut self,
        condition: Expr,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let _ = self.resolve_expr(condition.clone())?;
        let _ = self.resolve_expr(body.clone())?;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::While {
                condition: condition.into(),
                body: body.into()
            },
            span,
        }))
    }

    fn resolve_for(
        &mut self,
        clause: Expr,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let _ = self.resolve_expr(clause.clone())?;
        let _ = self.resolve_expr(body.clone())?;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::For {
                clause: clause.into(),
                body: body.into(),
            },
            span,
        }))
    }

    fn resolve_match(
        &mut self,
        target: Expr,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let _ = self.resolve_expr(target.clone())?;

        match &body.kind {
            ExprKind::Block(case_exprs) => {
                for case_expr in case_exprs {
                    let _ = self.resolve_expr(case_expr.clone())?;
                }
            }
            _ => {
                return Err(ResolverError::InvalidExpression(
                    "Expected block for match cases".to_string(),
                ))
            }
        }

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Match {
                target: target.into(),
                body: body.into()
            },
            span,
        }))
    }

    fn resolve_closure(
        &mut self,
        parameters: Vec<Expr>,
        body: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let current_scope = self.scope.clone();
        let old_scope = std::mem::replace(
            &mut self.scope,
            Scope::with_parent(current_scope.clone()),
        );

        for param in &parameters {
            if let ExprKind::Identifier(_) = &param.kind {
                let param_entity = Entity::Variable {
                    name: param.clone(),
                    value: None,
                    mutable: false,
                    type_annotation: None,
                };
                self.scope.insert(param_entity)?;
            } else if let ExprKind::Typed { expr, ty } = &param.kind {
                if let ExprKind::Identifier(_) = expr.kind {
                    let param_entity = Entity::Variable {
                        name: *expr.clone(),
                        value: None,
                        mutable: false,
                        type_annotation: Some(Box::new(*ty.clone())),
                    };
                    self.scope.insert(param_entity)?;
                }
            }
        }

        let _ = self.resolve_expr(body.clone())?;

        self.scope = old_scope;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Closure {
                parameters,
                body: body.into()
            },
            span,
        }))
    }
}