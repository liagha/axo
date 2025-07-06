use {
    super::{
        Element, ElementKind,
        Symbol, SymbolKind
    },

    crate::{
        memory::discriminant,
        
        hash::{
            Hash, Hasher
        },
    },
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
                discriminant(self).hash(state);
                token_kind.hash(state);
            }
            ElementKind::Identifier(name) => {
                discriminant(self).hash(state);
                name.hash(state);
            }

            ElementKind::Procedural(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }

            // Composite
            ElementKind::Group(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Sequence(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Collection(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Series(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Bundle(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Constructor { name, body } => {
                discriminant(self).hash(state);
                name.hash(state);
                body.hash(state);
            }

            // Operations
            ElementKind::Binary { left, operator, right } => {
                discriminant(self).hash(state);
                left.hash(state);
                operator.kind.hash(state); // Only hash the kind of the token, not its span
                right.hash(state);
            }
            ElementKind::Unary { operator, operand } => {
                discriminant(self).hash(state);
                operator.kind.hash(state); // Only hash the kind of the token, not its span
                operand.hash(state);
            }

            // Access Expressions
            ElementKind::Labeled { label, element } => {
                discriminant(self).hash(state);
                label.hash(state);
                element.hash(state);
            }
            ElementKind::Index { element, index } => {
                discriminant(self).hash(state);
                element.hash(state);
                index.hash(state);
            }
            ElementKind::Invoke { target, parameters } => {
                discriminant(self).hash(state);
                target.hash(state);
                parameters.hash(state);
            }
            ElementKind::Path { tree } => {
                discriminant(self).hash(state);
                tree.hash(state);
            }
            ElementKind::Member { object, member } => {
                discriminant(self).hash(state);
                object.hash(state);
                member.hash(state);
            }

            // Control Flow
            ElementKind::Scope(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Match { target, body } => {
                discriminant(self).hash(state);
                target.hash(state);
                body.hash(state);
            }
            ElementKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                discriminant(self).hash(state);
                condition.hash(state);
                then_branch.hash(state);
                else_branch.hash(state);
            }
            ElementKind::Cycle { condition, body } => {
                discriminant(self).hash(state);
                condition.hash(state);
                body.hash(state);
            }
            ElementKind::Iterate { clause, body } => {
                discriminant(self).hash(state);
                clause.hash(state);
                body.hash(state);
            }

            // Declarations & Definitions
            ElementKind::Symbolization(symbol) => {
                discriminant(self).hash(state);
                symbol.hash(state);
            }
            ElementKind::Assignment { target, value } => {
                discriminant(self).hash(state);
                target.hash(state);
                value.hash(state);
            }

            // Flow Control Statements
            ElementKind::Return(expr_opt) => {
                discriminant(self).hash(state);
                expr_opt.hash(state);
            }
            ElementKind::Break(expr_opt) => {
                discriminant(self).hash(state);
                expr_opt.hash(state);
            }
            ElementKind::Skip(expr_opt) => {
                discriminant(self).hash(state);
                expr_opt.hash(state);
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
            (ElementKind::Cycle { condition: a_cond, body: a_body },
                ElementKind::Cycle { condition: b_cond, body: b_body }) => a_cond == b_cond && a_body == b_body,
            (ElementKind::Iterate { clause: a_clause, body: a_body },
                ElementKind::Iterate { clause: b_clause, body: b_body }) => a_clause == b_clause && a_body == b_body,

            // Declarations & Definitions
            (ElementKind::Symbolization(a), ElementKind::Symbolization(b)) => a == b,
            (ElementKind::Assignment { target: a_target, value: a_value },
                ElementKind::Assignment { target: b_target, value: b_value }) => {
                a_target == b_target && a_value == b_value
            },

            // Flow Control Statements
            (ElementKind::Return(a), ElementKind::Return(b)) => a == b,
            (ElementKind::Break(a), ElementKind::Break(b)) => a == b,
            (ElementKind::Skip(a), ElementKind::Skip(b)) => a == b,

            // Different variants are never equal
            _ => false,
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl PartialEq for SymbolKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SymbolKind::Inclusion { target: t1 }, SymbolKind::Inclusion { target: t2 }) => t1 == t2,
            (
                SymbolKind::Implementation { element: e1, body: b1 },
                SymbolKind::Implementation { element: e2, body: b2 },
            ) => e1 == e2 && b1 == b2,
            (SymbolKind::Formation { identifier: i1, .. }, SymbolKind::Formation { identifier: i2, .. }) => {
                i1 == i2
            },
            (
                SymbolKind::Interface { name: n1, body: b1 },
                SymbolKind::Interface { name: n2, body: b2 },
            ) => n1 == n2 && b1 == b2,
            (
                SymbolKind::Binding {
                    target: t1,
                    value: _v1,
                    ty: _ty1,
                    mutable: _m1,
                },
                SymbolKind::Binding {
                    target: t2,
                    value: _v2,
                    ty: _ty2,
                    mutable: _m2,
                },
            ) => {
                t1 == t2
            },
            (
                SymbolKind::Structure {
                    name: n1,
                    fields: f1,
                },
                SymbolKind::Structure {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (
                SymbolKind::Enumeration { name: n1, variants: v1 },
                SymbolKind::Enumeration { name: n2, variants: v2 },
            ) => n1 == n2 && v1 == v2,
            (
                SymbolKind::Function {
                    name: n1,
                    parameters: p1,
                    body: b1,
                },
                SymbolKind::Function {
                    name: n2,
                    parameters: p2,
                    body: b2,
                },
            ) => n1 == n2 && p1 == p2 && b1 == b2,
            _ => false,
        }
    }
}

impl Hash for SymbolKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SymbolKind::Inclusion { target } => {
                discriminant(self).hash(state);
                target.hash(state);
            }
            SymbolKind::Formation { identifier, .. } => {
                discriminant(self).hash(state);
                identifier.hash(state);
            }
            SymbolKind::Implementation { element, body } => {
                discriminant(self).hash(state);
                element.hash(state);
                body.hash(state);
            }
            SymbolKind::Interface { name, body } => {
                discriminant(self).hash(state);
                name.hash(state);
                body.hash(state);
            }
            SymbolKind::Binding { target, .. } => {
                discriminant(self).hash(state);
                target.hash(state);
            }
            SymbolKind::Structure { name, fields } => {
                discriminant(self).hash(state);
                name.hash(state);
                fields.hash(state);
            }
            SymbolKind::Enumeration { name, variants } => {
                discriminant(self).hash(state);
                name.hash(state);
                variants.hash(state);
            }
            SymbolKind::Function {
                name,
                parameters,
                body,
            } => {
                discriminant(self).hash(state);
                name.hash(state);
                parameters.hash(state);
                body.hash(state);
            }
        }
    }
}

impl Clone for SymbolKind {
    fn clone(&self) -> Self {
        match self {
            SymbolKind::Inclusion { target } => SymbolKind::Inclusion { target: target.clone() },
            SymbolKind::Formation { identifier, form } => SymbolKind::Formation {
                identifier: identifier.clone(),
                form: form.clone(),
            },
            SymbolKind::Implementation { element, body } => SymbolKind::Implementation {
                element: element.clone(),
                body: body.clone(),
            },
            SymbolKind::Interface { name, body } => SymbolKind::Interface {
                name: name.clone(),
                body: body.clone(),
            },
            SymbolKind::Binding { target, value, ty, mutable } => SymbolKind::Binding {
                target: target.clone(),
                value: value.clone(),
                ty: ty.clone(),
                mutable: *mutable,
            },
            SymbolKind::Structure { name, fields } => SymbolKind::Structure {
                name: name.clone(),
                fields: fields.clone(),
            },
            SymbolKind::Enumeration { name, variants } => SymbolKind::Enumeration {
                name: name.clone(),
                variants: variants.clone(),
            },
            SymbolKind::Function { name, parameters, body } => SymbolKind::Function {
                name: name.clone(),
                parameters: parameters.clone(),
                body: body.clone(),
            },
        }
    }
}

impl Eq for SymbolKind {}

impl Eq for Symbol {}
