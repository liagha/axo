use {
    core::hash::{
        Hash, Hasher
    },

    crate::axo_parser::{
        Element, ElementKind,
        Item, ItemKind
    }
};

impl Hash for Element {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Hash for ElementKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            // Primary Expressions
            ElementKind::Literal(token_kind) => {
                core::mem::discriminant(self).hash(state);
                token_kind.hash(state);
            }
            ElementKind::Identifier(name) => {
                core::mem::discriminant(self).hash(state);
                name.hash(state);
            }
            
            ElementKind::Procedural(element) => {
                core::mem::discriminant(self).hash(state);
                element.hash(state);
            }

            // Composite
            ElementKind::Group(elements) => {
                core::mem::discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Sequence(elements) => {
                core::mem::discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Collection(elements) => {
                core::mem::discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Series(elements) => {
                core::mem::discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Bundle(elements) => {
                core::mem::discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Constructor { name, body } => {
                core::mem::discriminant(self).hash(state);
                name.hash(state);
                body.hash(state);
            }

            // Operations
            ElementKind::Binary { left, operator, right } => {
                core::mem::discriminant(self).hash(state);
                left.hash(state);
                operator.kind.hash(state); // Only hash the kind of the token, not its span
                right.hash(state);
            }
            ElementKind::Unary { operator, operand } => {
                core::mem::discriminant(self).hash(state);
                operator.kind.hash(state); // Only hash the kind of the token, not its span
                operand.hash(state);
            }

            // Access Expressions
            ElementKind::Bind { key, value } => {
                core::mem::discriminant(self).hash(state);
                key.hash(state);
                value.hash(state);
            }
            ElementKind::Labeled { label, element } => {
                core::mem::discriminant(self).hash(state);
                label.hash(state);
                element.hash(state);
            }
            ElementKind::Index { element, index } => {
                core::mem::discriminant(self).hash(state);
                element.hash(state);
                index.hash(state);
            }
            ElementKind::Invoke { target, parameters } => {
                core::mem::discriminant(self).hash(state);
                target.hash(state);
                parameters.hash(state);
            }
            ElementKind::Path { tree } => {
                core::mem::discriminant(self).hash(state);
                tree.hash(state);
            }
            ElementKind::Member { object, member } => {
                core::mem::discriminant(self).hash(state);
                object.hash(state);
                member.hash(state);
            }

            // Control Flow
            ElementKind::Scope(elements) => {
                core::mem::discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Match { target, body } => {
                core::mem::discriminant(self).hash(state);
                target.hash(state);
                body.hash(state);
            }
            ElementKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                core::mem::discriminant(self).hash(state);
                condition.hash(state);
                then_branch.hash(state);
                else_branch.hash(state);
            }
            ElementKind::Loop { condition, body } => {
                core::mem::discriminant(self).hash(state);
                condition.hash(state);
                body.hash(state);
            }
            ElementKind::Iterate { clause, body } => {
                core::mem::discriminant(self).hash(state);
                clause.hash(state);
                body.hash(state);
            }

            // Declarations & Definitions
            ElementKind::Item(item_kind) => {
                core::mem::discriminant(self).hash(state);
                item_kind.hash(state);
            }
            ElementKind::Assignment { target, value } => {
                core::mem::discriminant(self).hash(state);
                target.hash(state);
                value.hash(state);
            }

            // Flow Control Statements
            ElementKind::Return(expr_opt) => {
                core::mem::discriminant(self).hash(state);
                expr_opt.hash(state);
            }
            ElementKind::Break(expr_opt) => {
                core::mem::discriminant(self).hash(state);
                expr_opt.hash(state);
            }
            ElementKind::Skip(expr_opt) => {
                core::mem::discriminant(self).hash(state);
                expr_opt.hash(state);
            }

            ElementKind::Invalid(_) => {
                core::mem::discriminant(self).hash(state);
                // Note: You'll need to implement Hash for ParseError if it doesn't already implement it
                // error.hash(state);
            }
        }
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl PartialEq for ElementKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Primary Expressions
            (ElementKind::Literal(a), ElementKind::Literal(b)) => a == b,
            (ElementKind::Identifier(a), ElementKind::Identifier(b)) => a == b,

            // Composite
            (ElementKind::Group(a), ElementKind::Group(b)) => a == b,
            (ElementKind::Collection(a), ElementKind::Collection(b)) => a == b,
            (ElementKind::Bundle(a), ElementKind::Bundle(b)) => a == b,
            (ElementKind::Constructor { name: a_name, body: a_body },
                ElementKind::Constructor { name: b_name, body: b_body }) => a_name == b_name && a_body == b_body,

            // Operations
            (ElementKind::Binary { left: a_left, operator: a_op, right: a_right },
                ElementKind::Binary { left: b_left, operator: b_op, right: b_right }) => {
                a_left == b_left && a_op == b_op && a_right == b_right
            },
            (ElementKind::Unary { operator: a_op, operand: a_operand },
                ElementKind::Unary { operator: b_op, operand: b_operand }) => {
                a_op == b_op && a_operand == b_operand
            },

            // Access Expressions
            (ElementKind::Bind { key: a_key, value: a_value },
                ElementKind::Bind { key: b_key, value: b_value }) => a_key == b_key && a_value == b_value,
            (ElementKind::Labeled { label: a_label, element: a_expr },
                ElementKind::Labeled { label: b_label, element: b_expr }) => a_label == b_label && a_expr == b_expr,
            (ElementKind::Index { element: a_expr, index: a_index },
                ElementKind::Index { element: b_expr, index: b_index }) => a_expr == b_expr && a_index == b_index,
            (ElementKind::Invoke { target: a_target, parameters: a_params },
                ElementKind::Invoke { target: b_target, parameters: b_params }) => {
                a_target == b_target && a_params == b_params
            },
            (ElementKind::Path { tree: a_tree }, ElementKind::Path { tree: b_tree }) => a_tree == b_tree,
            (ElementKind::Member { object: a_obj, member: a_mem },
                ElementKind::Member { object: b_obj, member: b_mem }) => a_obj == b_obj && a_mem == b_mem,

            // Control Flow
            (ElementKind::Scope(a), ElementKind::Scope(b)) => a == b,
            (ElementKind::Match { target: a_target, body: a_body },
                ElementKind::Match { target: b_target, body: b_body }) => a_target == b_target && a_body == b_body,
            (ElementKind::Conditional { condition: a_cond, then: a_then, alternate: a_else },
                ElementKind::Conditional { condition: b_cond, then: b_then, alternate: b_else }) => {
                a_cond == b_cond && a_then == b_then && a_else == b_else
            },
            (ElementKind::Loop { condition: a_cond, body: a_body },
                ElementKind::Loop { condition: b_cond, body: b_body }) => a_cond == b_cond && a_body == b_body,
            (ElementKind::Iterate { clause: a_clause, body: a_body },
                ElementKind::Iterate { clause: b_clause, body: b_body }) => a_clause == b_clause && a_body == b_body,

            // Declarations & Definitions
            (ElementKind::Item(a), ElementKind::Item(b)) => a == b,
            (ElementKind::Assignment { target: a_target, value: a_value },
                ElementKind::Assignment { target: b_target, value: b_value }) => {
                a_target == b_target && a_value == b_value
            },

            // Flow Control Statements
            (ElementKind::Return(a), ElementKind::Return(b)) => a == b,
            (ElementKind::Break(a), ElementKind::Break(b)) => a == b,
            (ElementKind::Skip(a), ElementKind::Skip(b)) => a == b,

            (ElementKind::Invalid(a), ElementKind::Invalid(b)) => a == b,

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
            (
                ItemKind::Implement { element: e1, body: b1 },
                ItemKind::Implement { element: e2, body: b2 },
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
            ItemKind::Implement { element, body } => {
                element.hash(state);
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

