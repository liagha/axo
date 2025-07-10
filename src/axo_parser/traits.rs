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
            ElementKind::Literal(kind) => {
                discriminant(self).hash(state);
                kind.hash(state);
            }
            ElementKind::Identifier(name) => {
                discriminant(self).hash(state);
                name.hash(state);
            }
            ElementKind::Procedural(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }

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
            ElementKind::Scope(elements) => {
                discriminant(self).hash(state);
                elements.hash(state);
            }
            ElementKind::Constructor { name, fields } => {
                discriminant(self).hash(state);
                name.hash(state);
                fields.hash(state);
            }

            ElementKind::Binary { left, operator, right } => {
                discriminant(self).hash(state);
                left.hash(state);
                operator.kind.hash(state);
                right.hash(state);
            }
            ElementKind::Unary { operator, operand } => {
                discriminant(self).hash(state);
                operator.kind.hash(state);
                operand.hash(state);
            }

            ElementKind::Labeled { label, element } => {
                discriminant(self).hash(state);
                label.hash(state);
                element.hash(state);
            }
            ElementKind::Member { object, member } => {
                discriminant(self).hash(state);
                object.hash(state);
                member.hash(state);
            }
            ElementKind::Index { target, indexes } => {
                discriminant(self).hash(state);
                target.hash(state);
                indexes.hash(state);
            }
            ElementKind::Invoke { target, arguments } => {
                discriminant(self).hash(state);
                target.hash(state);
                arguments.hash(state);
            }
            ElementKind::Path { tree } => {
                discriminant(self).hash(state);
                tree.hash(state);
            }

            ElementKind::Conditional { condition, then, alternate } => {
                discriminant(self).hash(state);
                condition.hash(state);
                then.hash(state);
                alternate.hash(state);
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
            ElementKind::Match { target, body } => {
                discriminant(self).hash(state);
                target.hash(state);
                body.hash(state);
            }

            ElementKind::Symbolization(symbol) => {
                discriminant(self).hash(state);
                symbol.hash(state);
            }
            ElementKind::Assignment { target, value } => {
                discriminant(self).hash(state);
                target.hash(state);
                value.hash(state);
            }

            ElementKind::Return(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
            ElementKind::Break(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
            ElementKind::Skip(element) => {
                discriminant(self).hash(state);
                element.hash(state);
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
            (ElementKind::Literal(a), ElementKind::Literal(b)) => a == b,
            (ElementKind::Identifier(a), ElementKind::Identifier(b)) => a == b,
            (ElementKind::Procedural(a), ElementKind::Procedural(b)) => a == b,

            (ElementKind::Group(a), ElementKind::Group(b)) => a == b,
            (ElementKind::Sequence(a), ElementKind::Sequence(b)) => a == b,
            (ElementKind::Collection(a), ElementKind::Collection(b)) => a == b,
            (ElementKind::Series(a), ElementKind::Series(b)) => a == b,
            (ElementKind::Bundle(a), ElementKind::Bundle(b)) => a == b,
            (ElementKind::Scope(a), ElementKind::Scope(b)) => a == b,

            (
                ElementKind::Constructor { name: a_name, fields: a_fields },
                ElementKind::Constructor { name: b_name, fields: b_fields }
            ) => {
                a_name == b_name && a_fields == b_fields
            },

            (
                ElementKind::Binary { left: a_left, operator: a_op, right: a_right },
                ElementKind::Binary { left: b_left, operator: b_op, right: b_right }
            ) => {
                a_left == b_left && a_op == b_op && a_right == b_right
            },
            (
                ElementKind::Unary { operator: a_operator, operand: a_operand },
                ElementKind::Unary { operator: b_operator, operand: b_operand }
            ) => {
                a_operator == b_operator && a_operand == b_operand
            },

            (
                ElementKind::Labeled { label: a_label, element: a_element },
                ElementKind::Labeled { label: b_label, element: b_element }
            ) => {
                a_label == b_label && a_element == b_element
            },
            (
                ElementKind::Member { object: a_object, member: a_member },
                ElementKind::Member { object: b_object, member: b_member }
            ) => {
                a_object == b_object && a_member == b_member
            },
            (
                ElementKind::Index { target: a_target, indexes: a_indexes },
                ElementKind::Index { target: b_target, indexes: b_indexes }
            ) => {
                a_target == b_target && a_indexes == b_indexes
            },
            (
                ElementKind::Invoke { target: a_target, arguments: a_arguments },
                ElementKind::Invoke { target: b_target, arguments: b_arguments }
            ) => {
                a_target == b_target && a_arguments == b_arguments
            },
            (ElementKind::Path { tree: a_tree }, ElementKind::Path { tree: b_tree }) => {
                a_tree == b_tree
            },

            (
                ElementKind::Conditional { condition: a_condition, then: a_then, alternate: a_alternate },
                ElementKind::Conditional { condition: b_condition, then: b_then, alternate: b_alternate }
            ) => {
                a_condition == b_condition && a_then == b_then && a_alternate == b_alternate
            },
            (
                ElementKind::Cycle { condition: a_condition, body: a_body },
                ElementKind::Cycle { condition: b_condition, body: b_body }
            ) => {
                a_condition == b_condition && a_body == b_body
            },
            (
                ElementKind::Iterate { clause: a_clause, body: a_body },
                ElementKind::Iterate { clause: b_clause, body: b_body }
            ) => {
                a_clause == b_clause && a_body == b_body
            },
            (
                ElementKind::Match { target: a_target, body: a_body },
                ElementKind::Match { target: b_target, body: b_body }
            ) => {
                a_target == b_target && a_body == b_body
            },

            (ElementKind::Symbolization(a), ElementKind::Symbolization(b)) => a == b,
            (
                ElementKind::Assignment { target: a_target, value: a_value },
                ElementKind::Assignment { target: b_target, value: b_value }
            ) => {
                a_target == b_target && a_value == b_value
            },

            (ElementKind::Return(a), ElementKind::Return(b)) => a == b,
            (ElementKind::Break(a), ElementKind::Break(b)) => a == b,
            (ElementKind::Skip(a), ElementKind::Skip(b)) => a == b,

            _ => false,
        }
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Element {
            kind: self.kind.clone(),
            span: self.span.clone(),
        }
    }
}

impl Clone for ElementKind {
    fn clone(&self) -> Self {
        match self {
            ElementKind::Literal(kind) => ElementKind::Literal(kind.clone()),
            ElementKind::Identifier(name) => ElementKind::Identifier(name.clone()),
            ElementKind::Procedural(element) => ElementKind::Procedural(element.clone()),

            ElementKind::Group(elements) => ElementKind::Group(elements.clone()),
            ElementKind::Sequence(elements) => ElementKind::Sequence(elements.clone()),
            ElementKind::Collection(elements) => ElementKind::Collection(elements.clone()),
            ElementKind::Series(elements) => ElementKind::Series(elements.clone()),
            ElementKind::Bundle(elements) => ElementKind::Bundle(elements.clone()),
            ElementKind::Scope(elements) => ElementKind::Scope(elements.clone()),

            ElementKind::Constructor { name, fields } => ElementKind::Constructor {
                name: name.clone(),
                fields: fields.clone(),
            },

            ElementKind::Binary { left, operator, right } => ElementKind::Binary {
                left: left.clone(),
                operator: operator.clone(),
                right: right.clone(),
            },
            ElementKind::Unary { operator, operand } => ElementKind::Unary {
                operator: operator.clone(),
                operand: operand.clone(),
            },

            ElementKind::Labeled { label, element } => ElementKind::Labeled {
                label: label.clone(),
                element: element.clone(),
            },
            ElementKind::Member { object, member } => ElementKind::Member {
                object: object.clone(),
                member: member.clone(),
            },
            ElementKind::Index { target, indexes } => ElementKind::Index {
                target: target.clone(),
                indexes: indexes.clone(),
            },
            ElementKind::Invoke { target, arguments } => ElementKind::Invoke {
                target: target.clone(),
                arguments: arguments.clone(),
            },
            ElementKind::Path { tree } => ElementKind::Path {
                tree: tree.clone(),
            },

            ElementKind::Conditional { condition, then, alternate } => ElementKind::Conditional {
                condition: condition.clone(),
                then: then.clone(),
                alternate: alternate.clone(),
            },
            ElementKind::Cycle { condition, body } => ElementKind::Cycle {
                condition: condition.clone(),
                body: body.clone(),
            },
            ElementKind::Iterate { clause, body } => ElementKind::Iterate {
                clause: clause.clone(),
                body: body.clone(),
            },
            ElementKind::Match { target, body } => ElementKind::Match {
                target: target.clone(),
                body: body.clone(),
            },

            ElementKind::Symbolization(symbol) => ElementKind::Symbolization(symbol.clone()),
            ElementKind::Assignment { target, value } => ElementKind::Assignment {
                target: target.clone(),
                value: value.clone(),
            },

            ElementKind::Return(element) => ElementKind::Return(element.clone()),
            ElementKind::Break(element) => ElementKind::Break(element.clone()),
            ElementKind::Skip(element) => ElementKind::Skip(element.clone()),
        }
    }
}

impl Eq for Element {}

impl Eq for ElementKind {}

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

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Symbol {
            kind: self.kind.clone(),
            span: self.span.clone(),
        }
    }
}

impl PartialEq for SymbolKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SymbolKind::Inclusion { target: t1 }, SymbolKind::Inclusion { target: t2 }) => t1 == t2,
            (
                SymbolKind::Formation { identifier: i1, form: f1 },
                SymbolKind::Formation { identifier: i2, form: f2 },
            ) => i1 == i2 && f1 == f2,
            (
                SymbolKind::Implementation { element: e1, body: b1 },
                SymbolKind::Implementation { element: e2, body: b2 },
            ) => e1 == e2 && b1 == b2,
            (
                SymbolKind::Interface { name: n1, body: b1 },
                SymbolKind::Interface { name: n2, body: b2 },
            ) => n1 == n2 && b1 == b2,
            (
                SymbolKind::Slot { target: t1, value: v1, ty: ty1 },
                SymbolKind::Slot { target: t2, value: v2, ty: ty2 },
            ) => t1 == t2 && v1 == v2 && ty1 == ty2,
            (
                SymbolKind::Binding { target: t1, value: v1, ty: ty1, mutable: m1 },
                SymbolKind::Binding { target: t2, value: v2, ty: ty2, mutable: m2 },
            ) => t1 == t2 && v1 == v2 && ty1 == ty2 && m1 == m2,
            (
                SymbolKind::Structure { name: n1, entries: f1 },
                SymbolKind::Structure { name: n2, entries: f2 },
            ) => n1 == n2 && f1 == f2,
            (
                SymbolKind::Enumeration { name: n1, variants: v1 },
                SymbolKind::Enumeration { name: n2, variants: v2 },
            ) => n1 == n2 && v1 == v2,
            (
                SymbolKind::Function { name: n1, parameters: p1, body: b1 },
                SymbolKind::Function { name: n2, parameters: p2, body: b2 },
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
            SymbolKind::Formation { identifier, form } => {
                discriminant(self).hash(state);
                identifier.hash(state);
                form.hash(state);
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
            SymbolKind::Slot { target, value, ty } => {
                discriminant(self).hash(state);
                target.hash(state);
                value.hash(state);
                ty.hash(state);
            }
            SymbolKind::Binding { target, value, ty, mutable } => {
                discriminant(self).hash(state);
                target.hash(state);
                value.hash(state);
                ty.hash(state);
                mutable.hash(state);
            }
            SymbolKind::Structure { name, entries: fields } => {
                discriminant(self).hash(state);
                name.hash(state);
                fields.hash(state);
            }
            SymbolKind::Enumeration { name, variants } => {
                discriminant(self).hash(state);
                name.hash(state);
                variants.hash(state);
            }
            SymbolKind::Function { name, parameters, body } => {
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
            SymbolKind::Slot { target, value, ty } => SymbolKind::Slot {
                target: target.clone(),
                value: value.clone(),
                ty: ty.clone(),
            },
            SymbolKind::Binding { target, value, ty, mutable } => SymbolKind::Binding {
                target: target.clone(),
                value: value.clone(),
                ty: ty.clone(),
                mutable: *mutable,
            },
            SymbolKind::Structure { name, entries: fields } => SymbolKind::Structure {
                name: name.clone(),
                entries: fields.clone(),
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

impl Eq for Symbol {}

impl Eq for SymbolKind {}