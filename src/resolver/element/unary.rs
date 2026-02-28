use crate::{
    parser::Element,
    resolver::{
        checker::{CheckError, Checkable, Type, TypeKind},
        ErrorKind, Resolution, Resolvable, ResolveError, Resolver,
    },
    scanner::TokenKind,
    schema::Unary,
};

pub(super) fn resolve_unary<'element>(
    element: &Element<'element>,
    unary: &Unary<crate::scanner::Token<'element>, Box<Element<'element>>>,
    resolver: &mut Resolver<'element>,
    analysis: crate::resolver::analyzer::Analysis<'element>,
) -> Result<Resolution<'element>, Vec<ResolveError<'element>>> {
    let operand = unary.operand.resolve(resolver)?;
    let operator = match &unary.operator.kind {
        TokenKind::Operator(operator) => operator,
        _ => {
            return Err(vec![ResolveError::new(
                ErrorKind::Check {
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::InvalidOperation(
                            unary.operator.clone(),
                        ),
                        unary.operator.span,
                    ),
                },
                unary.operator.span,
            )]);
        }
    };

    let typ = match operator.as_slice() {
        [crate::scanner::OperatorKind::Ampersand] => {
            if !unary.operand.is_addressable() {
                return Err(vec![ResolveError::new(
                    ErrorKind::Check {
                        error: CheckError::new(
                            crate::resolver::checker::ErrorKind::InvalidOperation(
                                unary.operator.clone(),
                            ),
                            unary.operator.span,
                        ),
                    },
                    unary.operator.span,
                )]);
            }

            Type::pointer(operand.typed, element.span)
        }
        [crate::scanner::OperatorKind::Star] => match operand.typed.kind {
            TypeKind::Pointer { to } => *to,
            TypeKind::Infer => Type::new(TypeKind::Infer, element.span),
            _ => {
                return Err(vec![ResolveError::new(
                    ErrorKind::Check {
                        error: CheckError::new(
                            crate::resolver::checker::ErrorKind::Mismatch(
                                Type::pointer(
                                    Type::new(TypeKind::Infer, element.span),
                                    element.span,
                                ),
                                operand.typed,
                            ),
                            element.span,
                        ),
                    },
                    element.span,
                )]);
            }
        },
        _ => element.infer().map_err(|error| {
            vec![ResolveError::new(
                ErrorKind::Check {
                    error: error.clone(),
                },
                error.span,
            )]
        })?,
    };

    Ok(Resolution::new(None, typ, analysis))
}
