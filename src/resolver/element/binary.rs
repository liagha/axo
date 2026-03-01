use crate::{
    parser::{Element, ElementKind, SymbolKind},
    resolver::{
        ErrorKind, Resolution, Resolvable, ResolveError, Resolver,
    },
    scanner::{OperatorKind, Token, TokenKind},
    schema::Binary,
    tracker::Span,
};
use crate::analyzer::{AnalyzeError, ErrorKind as AnalyzeErrorKind};
use crate::checker::{CheckError, Checkable};

pub(super) fn resolve_binary<'element>(
    element: &Element<'element>,
    binary: &Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>,
    resolver: &mut Resolver<'element>,
    analysis: crate::analyzer::Analysis<'element>,
) -> Result<Resolution<'element>, Vec<ResolveError<'element>>> {
    let is_namespace = match &binary.operator.kind {
        TokenKind::Operator(operator) => {
            matches!(operator.as_slice(), [OperatorKind::Dot])
        }
        _ => false,
    };

    if is_namespace {
        let left = binary.left.resolve(resolver)?;

        if let Some(id) = left.reference {
            let symbol = match resolver.scope.get_id(id).cloned() {
                Some(symbol) => symbol,
                None => binary.left.resolve_path(resolver)?,
            };

            resolver.enter_scope(symbol.scope.clone());

            let resolved = binary.right.resolve(resolver);

            resolver.exit();

            resolved
        } else {
            let field = match &binary.right.kind {
                ElementKind::Literal(Token {
                    kind: TokenKind::Identifier(name),
                    ..
                }) => name.clone(),
                _ => {
                    return Err(vec![ResolveError::new(
                        ErrorKind::Analyze {
                            error: AnalyzeError::new(
                                AnalyzeErrorKind::InvalidOperation(binary.operator.clone()),
                                binary.operator.span,
                            ),
                        },
                        binary.operator.span,
                    )]);
                }
            };

            let left_type = left.typed.clone();
            let field_type = match left_type.kind {
                crate::checker::TypeKind::Structure(structure) => {
                    let struct_name = structure.target.clone();
                    let identifier = Element::new(
                        ElementKind::Literal(Token::new(
                            TokenKind::Identifier(struct_name),
                            Span::void(),
                        )),
                        Span::void(),
                    );
                    let struct_symbol = resolver
                        .scope
                        .try_get(&identifier)
                        .map_err(|errors| errors)?;

                    let field_symbol = match struct_symbol.kind {
                        SymbolKind::Structure(structure) => structure
                            .members
                            .iter()
                            .find(|symbol| match &symbol.kind {
                                SymbolKind::Binding(binding) => binding
                                    .target
                                    .brand()
                                    .and_then(|token| match token.kind {
                                        TokenKind::Identifier(name) => Some(name == field),
                                        _ => None,
                                    })
                                    .unwrap_or(false),
                                _ => false,
                            })
                            .cloned(),
                        _ => None,
                    };

                    if let Some(field_symbol) = field_symbol {
                        let inferred: Result<
                            crate::checker::Type<'element>,
                            CheckError<'element>,
                        > = field_symbol.infer();
                        inferred.map_err(|error| {
                            vec![ResolveError::new(
                                ErrorKind::Check {
                                    error: error.clone(),
                                },
                                error.span,
                            )]
                        })?
                    } else {
                        return Err(vec![ResolveError::new(
                            ErrorKind::UndefinedSymbol {
                                query: Token::new(TokenKind::Identifier(field), binary.right.span),
                            },
                            binary.right.span,
                        )]);
                    }
                }
                _ => left_type.clone(),
            };

            Ok(Resolution::new(None, field_type, analysis))
        }
    } else {
        binary.left.resolve(resolver)?;
        binary.right.resolve(resolver)?;

        let typ = element.infer().map_err(|error| {
            vec![ResolveError::new(
                ErrorKind::Check {
                    error: error.clone(),
                },
                error.span,
            )]
        })?;

        Ok(Resolution::new(None, typ, analysis))
    }
}
