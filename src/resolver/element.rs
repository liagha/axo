use {
    super::{
        ErrorKind, ResolveError, Resolver,
        scope::Scope,
        checker::Checkable,
        resolver::Resolvable,
    },
    crate::{
        data::{
            memory::replace,
        },
        parser::{
            Element, ElementKind,
        },
        scanner::{
            OperatorKind,
            Token, TokenKind,
        },
    }
};
use crate::resolver::analyzer::Analyzable;

impl<'element> Resolvable<'element> for Element<'element> {
    fn resolve(&self, resolver: &mut Resolver<'element>) {
        let Element { kind, .. } = self.clone();

        match kind {
            ElementKind::Delimited(delimited) => {
                resolver.enter();
                delimited.items.iter().for_each(|item| item.resolve(resolver));
                resolver.exit();
            }

            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                resolver.get(&self);
            }

            ElementKind::Construct { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. } => {
                resolver.get(&self);
            }

            ElementKind::Binary(binary) => {
                if let Some(left) = resolver.get(&*binary.right) {
                    resolver.lookup(&*binary.right, &left.scope);    
                }
            }

            ElementKind::Unary(unary) => unary.operand.resolve(resolver),
            
            ElementKind::Closure(closure) => {
                closure.body.resolve(resolver);
            }

            ElementKind::Symbolize(_)
            | ElementKind::Literal(_) => {}
        }

        let analysis = self.analyze(resolver);

        match analysis {
            Ok(analysis) => {
                resolver.output.push(analysis);
            }
            Err(error) => {
                let error = ResolveError::new(ErrorKind::Analyze { error: error.clone() }, error.span);

                resolver.errors.push(error);
            }
        }
    }
}