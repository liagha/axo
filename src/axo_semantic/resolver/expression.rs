use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind};
use crate::axo_semantic::Resolver;
use crate::axo_semantic::symbol::{Symbol, SymbolKind};

pub trait Expression {
    fn resolve_invoke(&mut self, target: Expr, parameters: Vec<Expr>) -> Symbol;
    fn resolve_member(&mut self, target: Expr, member: Expr) -> Symbol;
    fn resolve_struct(&mut self, name: Expr, body: Expr) -> Symbol;
}

impl Expression for Resolver {
    fn resolve_invoke(&mut self, target: Expr, parameters: Vec<Expr>) -> Symbol {
        let symbol = Symbol {
            kind: SymbolKind::Function {
                name: target,
                parameters,
                body: Expr::dummy(),
                return_type: None,
            },
            span: Span::zero()
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_member(&mut self, target: Expr, member: Expr) -> Symbol {
        let symbol = Symbol {
            kind: SymbolKind::Variable {
                name: target,
                value: None,
                mutable: false,
                ty: None,
            },
            span: Span::zero()
        };

        let found = self.lookup(&symbol);

        found
    }

    fn resolve_struct(&mut self, name: Expr, body: Expr) -> Symbol {
        let symbol = Symbol {
            kind: SymbolKind::Struct {
                name,
                fields: {
                    let mut fields = Vec::new();

                    match body {
                        Expr { kind: ExprKind::Block(block), .. } => {
                            for expr in block {
                                fields.push(self.resolve_expr(expr));
                            }
                        }
                        _ => fields.push(self.resolve_expr(body))
                    }

                    fields
                }
            },
            span: Span::zero()
        };

        let found = self.lookup(&symbol);

        found
    }
}