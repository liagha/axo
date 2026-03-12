use crate::checker::types::Type;
use crate::checker::CheckError;
use crate::data::Identity;
use crate::parser::Element;
use crate::tracker::Span;

pub struct Checker<'check, 'source> {
    pub input: &'check mut Vec<Element<'source>>,
    pub errors: Vec<CheckError<'source>>,
    pub vars: Vec<Option<Type<'source>>>,
}

pub trait Checkable<'source> {
    fn check(&mut self, checker: &mut Checker<'_, 'source>);
}

impl<'check, 'source> Checker<'check, 'source> {
    pub fn new(input: &'check mut Vec<Element<'source>>) -> Self {
        Self {
            input,
            errors: vec![],
            vars: vec![],
        }
    }

    pub fn check(&mut self) {
        let mut elements = std::mem::take(self.input);
        for element in &mut elements {
            element.check(self);
        }
        *self.input = elements;
    }

    pub fn fresh(&mut self, span: Span<'source>) -> Type<'source> {
        let id = self.vars.len();
        self.vars.push(None);
        Type::new(crate::checker::TypeKind::Variable(id), span)
    }

    pub fn resolve(&mut self, ty: &Type<'source>) -> Type<'source> {
        if let crate::checker::TypeKind::Variable(id) = ty.kind {
            if let Some(resolved) = self.vars[id].clone() {
                let deep = self.resolve(&resolved);
                self.vars[id] = Some(deep.clone());
                return deep;
            }
        }
        ty.clone()
    }

    fn occurs(&mut self, variable: Identity, ty: &Type<'source>) -> bool {
        let resolved = self.resolve(ty);

        match resolved.kind {
            crate::checker::TypeKind::Variable(id) => id == variable,

            crate::checker::TypeKind::Pointer { ref target } => {
                self.occurs(variable, target)
            }

            crate::checker::TypeKind::Array { ref member, .. } => {
                self.occurs(variable, member)
            }

            crate::checker::TypeKind::Tuple { ref members } => {
                members.iter().any(|member| self.occurs(variable, member))
            }

            crate::checker::TypeKind::Function(_, ref parameters, ref output) => {
                if parameters.iter().any(|param| self.occurs(variable, param)) {
                    return true;
                }

                if let Some(out) = output {
                    return self.occurs(variable, out);
                }

                false
            }

            _ => false,
        }
    }

    pub fn unify(&mut self, span: Span<'source>, left: &Type<'source>, right: &Type<'source>) -> Type<'source> {
        let left = self.resolve(left);
        let right = self.resolve(right);

        if left == right {
            return left;
        }

        use crate::checker::TypeKind::*;

        match (left.kind.clone(), right.kind.clone()) {

            (Variable(id), _) => {
                if self.occurs(id, &right) {
                    self.errors.push(CheckError::new(
                        crate::checker::ErrorKind::Mismatch(left.clone(), right.clone()),
                        span,
                    ));
                    return left;
                }

                self.vars[id] = Some(right.clone());
                right
            }

            (_, Variable(id)) => {
                if self.occurs(id, &left) {
                    self.errors.push(CheckError::new(
                        crate::checker::ErrorKind::Mismatch(left.clone(), right.clone()),
                        span,
                    ));
                    return left;
                }

                self.vars[id] = Some(left.clone());
                left
            }

            (Pointer { target: left_target }, Pointer { target: right_target }) => {
                let unified_target = self.unify(span, &left_target, &right_target);
                Type::new(Pointer { target: Box::new(unified_target) }, left.span)
            }

            (
                Array { member: left_member, size: left_size },
                Array { member: right_member, size: right_size }
            ) if left_size == right_size => {

                let unified_member = self.unify(span, &left_member, &right_member);

                Type::new(
                    Array {
                        member: Box::new(unified_member),
                        size: left_size,
                    },
                    left.span,
                )
            }

            (
                Tuple { members: left_members },
                Tuple { members: right_members }
            ) if left_members.len() == right_members.len() => {

                let mut unified_members = Vec::with_capacity(left_members.len());

                for (left_member, right_member) in left_members.iter().zip(right_members.iter()) {
                    unified_members.push(self.unify(span, left_member, right_member));
                }

                Type::new(Tuple { members: unified_members }, left.span)
            }

            (
                Integer { size: left_size, signed: left_signed },
                Integer { size: right_size, signed: right_signed }
            ) => {
                Type::new(
                    Integer {
                        size: left_size.max(right_size),
                        signed: left_signed || right_signed,
                    },
                    left.span,
                )
            }

            (Float { size: left_size }, Float { size: right_size }) => {
                Type::new(Float { size: left_size.max(right_size) }, left.span)
            }

            (
                Function(name, left_parameters, left_output),
                Function(_, right_parameters, right_output)
            ) if left_parameters.len() == right_parameters.len() => {

                let mut unified_parameters = Vec::with_capacity(left_parameters.len());

                for (left_param, right_param) in left_parameters.iter().zip(right_parameters.iter()) {
                    unified_parameters.push(self.unify(span, left_param, right_param));
                }

                let unified_output = match (left_output, right_output) {
                    (Some(left_out), Some(right_out)) => {
                        Some(Box::new(self.unify(span, &left_out, &right_out)))
                    }

                    (Some(left_out), None) => Some(left_out.clone()),
                    (None, Some(right_out)) => Some(right_out.clone()),
                    (None, None) => None,
                };

                Type::new(
                    Function(name, unified_parameters, unified_output),
                    left.span,
                )
            }

            _ => {
                self.errors.push(CheckError::new(
                    crate::checker::ErrorKind::Mismatch(left.clone(), right.clone()),
                    span,
                ));

                left
            }
        }
    }
}
