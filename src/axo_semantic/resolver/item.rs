use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, ItemKind};
use crate::axo_semantic::Resolver;
use crate::axo_semantic::resolver::entity::Entity;
use crate::axo_semantic::resolver::error::ResolverError;
use crate::axo_semantic::resolver::scope::Scope;

pub trait ItemResolver {
    fn resolve_item(&mut self, item: ItemKind, span: Span) -> Result<Entity, ResolverError>;
    fn resolve_field(&mut self, expr: Expr) -> Result<Entity, ResolverError>;
    fn resolve_definition(
        &mut self,
        name_expr: Expr,
        value_opt: Option<Box<Expr>>,
    ) -> Result<Entity, ResolverError>;

    fn resolve_assignment(
        &mut self,
        target: Expr,
        value: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError>;
}

impl ItemResolver for Resolver {
    fn resolve_item(&mut self, item: ItemKind, span: Span) -> Result<Entity, ResolverError> {
        match item {
            ItemKind::Struct { name, body } => {
                let mut fields = Vec::new();
                let generic_params = Vec::new();

                match body.kind {
                    ExprKind::Block(block_exprs) => {
                        for field_expr in block_exprs {
                            fields.push(self.resolve_field(field_expr)?);
                        }
                    }
                    _ => fields.push(self.resolve_field(*body)?)
                };

                let struct_entity = Entity::Struct {
                    name: *name,
                    fields,
                    generic_params,
                };

                self.scope.insert(struct_entity.clone())?;
                Ok(struct_entity)
            }
            ItemKind::Enum { name, body } => {
                let mut variants = Vec::new();
                let generic_params = Vec::new();

                match body.kind {
                    ExprKind::Block(block_exprs) => {
                        for variant_expr in block_exprs {
                            match variant_expr.kind {
                                ExprKind::Identifier(_) => {
                                    let variant_entity = Entity::Variant { name: variant_expr };
                                    variants.push(variant_entity);
                                }
                                ExprKind::Invoke { .. } => {
                                    let variant_entity = Entity::Variant { name: variant_expr };
                                    variants.push(variant_entity);
                                }
                                ExprKind::Struct { .. } => {
                                    let variant_entity = Entity::Variant { name: variant_expr };
                                    variants.push(variant_entity);
                                }
                                _ => {
                                    return Err(ResolverError::InvalidEnumVariant(format!(
                                        "{:?}",
                                        variant_expr.kind
                                    )))
                                }
                            }
                        }
                    }
                    ExprKind::Identifier(_) => {
                        let variant_entity = Entity::Variant { name: *body };
                        variants.push(variant_entity);
                    }
                    ExprKind::Invoke { .. } => {
                        let variant_entity = Entity::Variant { name: *body };
                        variants.push(variant_entity);
                    }
                    ExprKind::Struct { .. } => {
                        let variant_entity = Entity::Variant { name: *body };
                        variants.push(variant_entity);
                    }
                    _ => {
                        return Err(ResolverError::InvalidEnumVariant(format!(
                            "{:?}",
                            body.kind
                        )))
                    }
                };

                let enum_entity = Entity::Enum {
                    name: *name,
                    variants,
                    generic_params,
                };

                self.scope.insert(enum_entity.clone())?;
                Ok(enum_entity)
            }
            ItemKind::Function { name, parameters, body } => {
                let current_scope = self.scope.clone();

                let old_scope = std::mem::replace(
                    &mut self.scope,
                    Scope::with_parent(current_scope.clone()),
                );

                for param in &parameters {
                    let entity = self.resolve_field(param.clone())?;

                    self.scope.insert(entity)?;
                }

                let _ = self.resolve_expr(*body.clone());

                self.scope = old_scope;

                let function_entity = Entity::Function {
                    name: *name,
                    parameters,
                    body: *body,
                    return_type: None,
                };

                self.scope.insert(function_entity.clone())?;
                Ok(function_entity)
            }
            ItemKind::Macro { name, parameters, body } => {
                let current_scope = self.scope.clone();

                let old_scope = std::mem::replace(
                    &mut self.scope,
                    Scope::with_parent(current_scope.clone()),
                );

                for param in &parameters {
                    let entity = self.resolve_field(param.clone())?;

                    self.scope.insert(entity)?;
                }

                let _ = self.resolve_expr(*body.clone());

                self.scope = old_scope;

                let function_entity = Entity::Macro {
                    name: *name,
                    parameters,
                    body: *body,
                };

                self.scope.insert(function_entity.clone())?;
                Ok(function_entity)
            }
            ItemKind::Trait(name, body) => {
                let trait_entity = Entity::Trait {
                    name: *name,
                    body: *body,
                    generic_params: Vec::new(),
                };

                self.scope.insert(trait_entity.clone())?;
                Ok(trait_entity)
            }
            ItemKind::Implement(trait_, target) => {
                let trait_entity = if let ExprKind::Identifier(trait_name) = &trait_.kind {
                    if let Some(entity) = self.scope.lookup(trait_name) {
                        match entity {
                            Entity::Trait { .. } => Some(Box::new(entity.clone())),
                            _ => {
                                return Err(ResolverError::TypeMismatch(
                                    "Trait".to_string(),
                                    "Not a trait".to_string(),
                                ))
                            }
                        }
                    } else {
                        return Err(ResolverError::UndefinedSymbol(trait_name.clone(), None));
                    }
                } else {
                    None
                };

                let impl_entity = Entity::Impl {
                    trait_: trait_entity,
                    target: *target,
                    body: Expr {
                        kind: ExprKind::Block(Vec::new()),
                        span,
                    },
                };

                self.scope.insert(impl_entity.clone())?;
                Ok(impl_entity)
            }
            ItemKind::Use(path_expr) => {
                Ok(Entity::Expression(Expr {
                    kind: ExprKind::Item(ItemKind::Use(path_expr)),
                    span,
                }))
            }
        }
    }

    fn resolve_field(&mut self, expr: Expr) -> Result<Entity, ResolverError> {
        match expr.kind {
            ExprKind::Identifier(_) => {
                let field_entity = Entity::Field {
                    name: expr,
                    field_type: None,
                    default: None,
                };

                Ok(field_entity)
            }
            ExprKind::Typed { expr, ty } => {
                let field_entity = Entity::Field {
                    name: *expr,
                    field_type: Some(*ty),
                    default: None,
                };

                Ok(field_entity)
            }
            ExprKind::Assignment { target, value } => {
                let field = if let Expr { kind: ExprKind::Typed { expr, ty }, .. } = *target {
                    Entity::Field {
                        name: *expr,
                        field_type: Some(*ty),
                        default: Some(*value),
                    }
                } else {
                    Entity::Field {
                        name: *target,
                        field_type: None,
                        default: Some(*value),
                    }
                };

                Ok(field)
            }
            _ => {
                Err(ResolverError::InvalidStructField(format!(
                    "{:?}",
                    expr.kind
                )))
            }
        }
    }

    fn resolve_definition(
        &mut self,
        name_expr: Expr,
        value_opt: Option<Box<Expr>>,
    ) -> Result<Entity, ResolverError> {
        let value = if let Some(val_expr) = value_opt {
            let _ = self.resolve_expr(*val_expr.clone());
            Some(*val_expr)
        } else {
            None
        };

        let variable_entity = Entity::Variable {
            name: name_expr,
            value,
            mutable: false,
            type_annotation: None,
        };

        self.scope.insert(variable_entity.clone())?;
        Ok(variable_entity)
    }

    fn resolve_assignment(
        &mut self,
        target: Expr,
        value: Expr,
        span: Span,
    ) -> Result<Entity, ResolverError> {
        match target.kind.clone() {
            ExprKind::Identifier(name) => {
                if let Some(entity) = self.scope.lookup(&name) {
                    match entity {
                        Entity::Variable { mutable, .. } => {
                            if !mutable {
                                return Err(ResolverError::InvalidAssignment);
                            }
                        }
                        _ => return Err(ResolverError::InvalidAssignment),
                    }
                } else {
                    return Err(ResolverError::UndefinedSymbol(name, None));
                }
            }
            ExprKind::Member { object, member } => {
                let _ = self.resolve_expr(*object)?;
                let _ = self.resolve_expr(*member)?;
            }
            ExprKind::Index { expr, index } => {
                let _ = self.resolve_expr(*expr)?;
                let _ = self.resolve_expr(*index)?;
            }
            _ => return Err(ResolverError::InvalidAssignment),
        }

        let _ = self.resolve_expr(value.clone())?;

        Ok(Entity::Expression(Expr {
            kind: ExprKind::Assignment { target: target.into(), value: value.into() },
            span,
        }))
    }
}