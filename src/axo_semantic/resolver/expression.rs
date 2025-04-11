use crate::axo_lexer::{Span, Token};
use crate::axo_parser::{Expr, ExprKind};
use crate::axo_semantic::resolver::entity::Entity;
use crate::axo_semantic::resolver::error::ResolverError;
use crate::axo_semantic::Resolver;

/// Trait for resolving basic expressions like identifiers, literals, and operators
pub trait ExpressionResolver {
    fn resolve_identifier(&self, name: String) -> Result<Entity, ResolverError>;
    fn resolve_binary(&mut self, left: Expr, op: Token, right: Expr, span: Span) -> Result<Entity, ResolverError>;
    fn resolve_unary(&mut self, op: Token, operand: Expr, span: Span) -> Result<Entity, ResolverError>;
    fn resolve_invoke(&mut self, func: Expr, args: Vec<Expr>, span: Span) -> Result<Entity, ResolverError>;
    fn resolve_member(&mut self, obj: Expr, member: Expr, span: Span) -> Result<Entity, ResolverError>;
    fn resolve_struct_instantiation(&mut self, name: Expr, fields: Expr, span: Span) -> Result<Entity, ResolverError>;
    fn resolve_expression(&mut self, expr: Expr) -> Result<Entity, ResolverError>;
}

impl ExpressionResolver for Resolver {
    fn resolve_identifier(&self, name: String) -> Result<Entity, ResolverError> {
        use crate::axo_data::{MatchType, SmartMatcher};

        if let Some(entity) = self.scope.lookup(&name) {
            Ok(entity.clone())
        } else {
            let matcher = SmartMatcher::default();
            let candidate_names: Vec<String> = self
                .scope
                .entities
                .iter()
                .filter_map(|sym| sym.get_name())
                .collect();

            let suggestion = matcher
                .find_best_match(&name, &candidate_names)
                .map(|m| m.name);

            Err(ResolverError::UndefinedSymbol(name, suggestion))
        }
    }

    fn resolve_binary(
        &mut self,
        left: Expr,
        operator: Token,
        right: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let _ = self.resolve_expr(left.clone())?;
        let _ = self.resolve_expr(right.clone())?;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Binary { left: left.into(), operator, right: right.into() },
            span,
        }))
    }

    fn resolve_unary(
        &mut self,
        operator: Token,
        operand: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let _ = self.resolve_expr(operand.clone())?;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Unary { operator, operand: operand.into() },
            span,
        }))
    }

    fn resolve_invoke(
        &mut self,
        func: Expr,
        args: Vec<Expr>,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let func_entity = self.resolve_expr(func.clone())?;

        match &func_entity {
            Entity::Function { parameters, .. } => {
                if args.len() != parameters.len() {
                    return Err(ResolverError::InvalidExpression(format!(
                        "Expected {} arguments, got {}",
                        parameters.len(),
                        args.len()
                    )));
                }
            }
            Entity::Macro { parameters, .. } => {
                if args.len() != parameters.len() {
                    return Err(ResolverError::InvalidExpression(format!(
                        "Expected {} arguments, got {}",
                        parameters.len(),
                        args.len()
                    )));
                }
            }
            _ => {
            }
        }

        for arg in &args {
            let _ = self.resolve_expr(arg.clone())?;
        }

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Invoke { target: func.into(), parameters: args },
            span,
        }))
    }

    fn resolve_member(
        &mut self,
        object: Expr,
        member: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let obj_entity = self.resolve_expr(object.clone())?;

        match &obj_entity {
            Entity::Struct { fields, .. } => {
                if let ExprKind::Identifier(member_name) = &member.kind {
                    let field_exists = fields.iter().any(|field| {
                        if let Some(field_name) = field.get_name() {
                            field_name == *member_name
                        } else {
                            false
                        }
                    });

                    if !field_exists {
                        return Err(ResolverError::InvalidExpression(format!(
                            "Field '{}' does not exist",
                            member_name
                        )));
                    }
                }
            }
            Entity::Enum { variants, .. } => {
                if let ExprKind::Identifier(variant_name) = &member.kind {
                    let variant_exists = variants.iter().any(|variant| {
                        if let Some(var_name) = variant.get_name() {
                            var_name == *variant_name
                        } else {
                            false
                        }
                    });

                    if !variant_exists {
                        return Err(ResolverError::InvalidExpression(format!(
                            "Variant '{}' does not exist",
                            variant_name
                        )));
                    }
                }
            }
            _ => {
            }
        }

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Member { object: object.into(), member: member.into() },
            span,
        }))
    }

    fn resolve_struct_instantiation(
        &mut self,
        name: Expr,
        fields: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        let name_entity = self.resolve_expr(name.clone())?;

        let struct_entity = match name_entity {
            Entity::Struct { .. } => name_entity,
            _ => {
                return Err(ResolverError::TypeMismatch(
                    "Struct".to_string(),
                    "Not a struct".to_string(),
                ))
            }
        };

        if let ExprKind::Block(field_exprs) = fields.kind.clone() {
            for field_expr in field_exprs {
                let _ = self.resolve_expr(field_expr)?;
            }
        } else {
            return Err(ResolverError::InvalidExpression(
                "Expected block for struct fields".to_string(),
            ));
        }

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Struct { name: name.into(), body: fields.into() },
            span,
        }))
    }

    fn resolve_expression(&mut self, expr: Expr) -> Result<Entity, ResolverError> {
        Ok(
            Entity::Expression(expr)
        )
    }
}