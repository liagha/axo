use {
    core::hash::{
        Hash, Hasher
    },

    crate::axo_parser::{
        Expr, ExprKind,
        Item, ItemKind
    }
};

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Hash for ExprKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            // Primary Expressions
            ExprKind::Literal(token_kind) => {
                core::mem::discriminant(self).hash(state);
                token_kind.hash(state);
            }
            ExprKind::Identifier(name) => {
                core::mem::discriminant(self).hash(state);
                name.hash(state);
            }

            // Composite
            ExprKind::Group(exprs) => {
                core::mem::discriminant(self).hash(state);
                exprs.hash(state);
            }
            ExprKind::Sequence(exprs) => {
                core::mem::discriminant(self).hash(state);
                exprs.hash(state);
            }
            ExprKind::Collection(exprs) => {
                core::mem::discriminant(self).hash(state);
                exprs.hash(state);
            }
            ExprKind::Series(exprs) => {
                core::mem::discriminant(self).hash(state);
                exprs.hash(state);
            }
            ExprKind::Bundle(exprs) => {
                core::mem::discriminant(self).hash(state);
                exprs.hash(state);
            }
            ExprKind::Constructor { name, body } => {
                core::mem::discriminant(self).hash(state);
                name.hash(state);
                body.hash(state);
            }

            // Operations
            ExprKind::Binary { left, operator, right } => {
                core::mem::discriminant(self).hash(state);
                left.hash(state);
                operator.kind.hash(state); // Only hash the kind of the token, not its span
                right.hash(state);
            }
            ExprKind::Unary { operator, operand } => {
                core::mem::discriminant(self).hash(state);
                operator.kind.hash(state); // Only hash the kind of the token, not its span
                operand.hash(state);
            }

            // Access Expressions
            ExprKind::Bind { key, value } => {
                core::mem::discriminant(self).hash(state);
                key.hash(state);
                value.hash(state);
            }
            ExprKind::Labeled { label, expr } => {
                core::mem::discriminant(self).hash(state);
                label.hash(state);
                expr.hash(state);
            }
            ExprKind::Index { expr, index } => {
                core::mem::discriminant(self).hash(state);
                expr.hash(state);
                index.hash(state);
            }
            ExprKind::Invoke { target, parameters } => {
                core::mem::discriminant(self).hash(state);
                target.hash(state);
                parameters.hash(state);
            }
            ExprKind::Path { tree } => {
                core::mem::discriminant(self).hash(state);
                tree.hash(state);
            }
            ExprKind::Member { object, member } => {
                core::mem::discriminant(self).hash(state);
                object.hash(state);
                member.hash(state);
            }

            ExprKind::Closure { parameters, body } => {
                core::mem::discriminant(self).hash(state);
                parameters.hash(state);
                body.hash(state);
            }

            // Control Flow
            ExprKind::Block(exprs) => {
                core::mem::discriminant(self).hash(state);
                exprs.hash(state);
            }
            ExprKind::Match { target, body } => {
                core::mem::discriminant(self).hash(state);
                target.hash(state);
                body.hash(state);
            }
            ExprKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                core::mem::discriminant(self).hash(state);
                condition.hash(state);
                then_branch.hash(state);
                else_branch.hash(state);
            }
            ExprKind::Loop { condition, body } => {
                core::mem::discriminant(self).hash(state);
                condition.hash(state);
                body.hash(state);
            }
            ExprKind::Iterate { clause, body } => {
                core::mem::discriminant(self).hash(state);
                clause.hash(state);
                body.hash(state);
            }

            // Declarations & Definitions
            ExprKind::Item(item_kind) => {
                core::mem::discriminant(self).hash(state);
                item_kind.hash(state);
            }
            ExprKind::Assignment { target, value } => {
                core::mem::discriminant(self).hash(state);
                target.hash(state);
                value.hash(state);
            }

            // Flow Control Statements
            ExprKind::Return(expr_opt) => {
                core::mem::discriminant(self).hash(state);
                expr_opt.hash(state);
            }
            ExprKind::Break(expr_opt) => {
                core::mem::discriminant(self).hash(state);
                expr_opt.hash(state);
            }
            ExprKind::Continue(expr_opt) => {
                core::mem::discriminant(self).hash(state);
                expr_opt.hash(state);
            }

            ExprKind::Error(_) => {
                core::mem::discriminant(self).hash(state);
                // Note: You'll need to implement Hash for ParseError if it doesn't already implement it
                // error.hash(state);
            }
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl PartialEq for ExprKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Primary Expressions
            (ExprKind::Literal(a), ExprKind::Literal(b)) => a == b,
            (ExprKind::Identifier(a), ExprKind::Identifier(b)) => a == b,

            // Composite
            (ExprKind::Group(a), ExprKind::Group(b)) => a == b,
            (ExprKind::Collection(a), ExprKind::Collection(b)) => a == b,
            (ExprKind::Bundle(a), ExprKind::Bundle(b)) => a == b,
            (ExprKind::Constructor { name: a_name, body: a_body },
                ExprKind::Constructor { name: b_name, body: b_body }) => a_name == b_name && a_body == b_body,

            // Operations
            (ExprKind::Binary { left: a_left, operator: a_op, right: a_right },
                ExprKind::Binary { left: b_left, operator: b_op, right: b_right }) => {
                a_left == b_left && a_op == b_op && a_right == b_right
            },
            (ExprKind::Unary { operator: a_op, operand: a_operand },
                ExprKind::Unary { operator: b_op, operand: b_operand }) => {
                a_op == b_op && a_operand == b_operand
            },

            // Access Expressions
            (ExprKind::Bind { key: a_key, value: a_value },
                ExprKind::Bind { key: b_key, value: b_value }) => a_key == b_key && a_value == b_value,
            (ExprKind::Labeled { label: a_label, expr: a_expr },
                ExprKind::Labeled { label: b_label, expr: b_expr }) => a_label == b_label && a_expr == b_expr,
            (ExprKind::Index { expr: a_expr, index: a_index },
                ExprKind::Index { expr: b_expr, index: b_index }) => a_expr == b_expr && a_index == b_index,
            (ExprKind::Invoke { target: a_target, parameters: a_params },
                ExprKind::Invoke { target: b_target, parameters: b_params }) => {
                a_target == b_target && a_params == b_params
            },
            (ExprKind::Path { tree: a_tree }, ExprKind::Path { tree: b_tree }) => a_tree == b_tree,
            (ExprKind::Member { object: a_obj, member: a_mem },
                ExprKind::Member { object: b_obj, member: b_mem }) => a_obj == b_obj && a_mem == b_mem,

            (ExprKind::Closure { parameters: a_params, body: a_body },
                ExprKind::Closure { parameters: b_params, body: b_body }) => {
                a_params == b_params && a_body == b_body
            },

            // Control Flow
            (ExprKind::Block(a), ExprKind::Block(b)) => a == b,
            (ExprKind::Match { target: a_target, body: a_body },
                ExprKind::Match { target: b_target, body: b_body }) => a_target == b_target && a_body == b_body,
            (ExprKind::Conditional { condition: a_cond, then: a_then, alternate: a_else },
                ExprKind::Conditional { condition: b_cond, then: b_then, alternate: b_else }) => {
                a_cond == b_cond && a_then == b_then && a_else == b_else
            },
            (ExprKind::Loop { condition: a_cond, body: a_body },
                ExprKind::Loop { condition: b_cond, body: b_body }) => a_cond == b_cond && a_body == b_body,
            (ExprKind::Iterate { clause: a_clause, body: a_body },
                ExprKind::Iterate { clause: b_clause, body: b_body }) => a_clause == b_clause && a_body == b_body,

            // Declarations & Definitions
            (ExprKind::Item(a), ExprKind::Item(b)) => a == b,
            (ExprKind::Assignment { target: a_target, value: a_value },
                ExprKind::Assignment { target: b_target, value: b_value }) => {
                a_target == b_target && a_value == b_value
            },

            // Flow Control Statements
            (ExprKind::Return(a), ExprKind::Return(b)) => a == b,
            (ExprKind::Break(a), ExprKind::Break(b)) => a == b,
            (ExprKind::Continue(a), ExprKind::Continue(b)) => a == b,

            (ExprKind::Error(a), ExprKind::Error(b)) => a == b,

            // Different variants are never equal
            _ => false,
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl PartialEq for ItemKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ItemKind::Use(e1), ItemKind::Use(e2)) => e1 == e2,
            (ItemKind::Expression(e1), ItemKind::Expression(e2)) => e1 == e2,
            (
                ItemKind::Implement { expr: e1, body: b1 },
                ItemKind::Implement { expr: e2, body: b2 },
            ) => e1 == e2 && b1 == b2,
            (
                ItemKind::Trait { name: n1, body: b1 },
                ItemKind::Trait { name: n2, body: b2 },
            ) => n1 == n2 && b1 == b2,
            (
                ItemKind::Variable {
                    target: t1,
                    value: _v1,
                    ty: _ty1,
                    mutable: _m1,
                },
                ItemKind::Variable {
                    target: t2,
                    value: _v2,
                    ty: _ty2,
                    mutable: _m2,
                },
            ) => {
                t1 == t2
            },
            (
                ItemKind::Field {
                    name: n1,
                    value: v1,
                    ty: ty1,
                },
                ItemKind::Field {
                    name: n2,
                    value: v2,
                    ty: ty2,
                },
            ) => n1 == n2 && v1 == v2 && ty1 == ty2,
            (
                ItemKind::Structure {
                    name: n1,
                    fields: f1,
                },
                ItemKind::Structure {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (
                ItemKind::Enum { name: n1, body: b1 },
                ItemKind::Enum { name: n2, body: b2 },
            ) => n1 == n2 && b1 == b2,
            (
                ItemKind::Macro {
                    name: n1,
                    parameters: p1,
                    body: b1,
                },
                ItemKind::Macro {
                    name: n2,
                    parameters: p2,
                    body: b2,
                },
            ) => n1 == n2 && p1 == p2 && b1 == b2,
            (
                ItemKind::Function {
                    name: n1,
                    parameters: p1,
                    body: b1,
                },
                ItemKind::Function {
                    name: n2,
                    parameters: p2,
                    body: b2,
                },
            ) => n1 == n2 && p1 == p2 && b1 == b2,
            (ItemKind::Unit, ItemKind::Unit) => true,
            _ => false,
        }
    }
}

impl Hash for ItemKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ItemKind::Use(e) => {
                e.hash(state);
            }
            ItemKind::Expression(e) => {
                e.hash(state);
            }
            ItemKind::Implement { expr, body } => {
                expr.hash(state);
                body.hash(state);
            }
            ItemKind::Trait { name, body } => {
                name.hash(state);
                body.hash(state);
            }
            ItemKind::Variable { target, .. } => {
                target.hash(state);
            }
            ItemKind::Field { name, value, ty } => {
                name.hash(state);
                value.hash(state);
                ty.hash(state);
            }
            ItemKind::Structure { name, fields } => {
                name.hash(state);
                fields.hash(state);
            }
            ItemKind::Enum { name, body } => {
                name.hash(state);
                body.hash(state);
            }
            ItemKind::Macro {
                name,
                parameters,
                body,
            } => {
                name.hash(state);
                parameters.hash(state);
                body.hash(state);
            }
            ItemKind::Function {
                name,
                parameters,
                body,
            } => {
                name.hash(state);
                parameters.hash(state);
                body.hash(state);
            }
            ItemKind::Unit => {
                self.hash(state);
            }
        }
    }
}

