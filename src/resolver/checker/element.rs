use crate::data::Str;
use crate::parser::{Element, ElementKind};
use crate::resolver::checker::checker::Checkable;
use crate::resolver::checker::{Type, TypeKind};
use crate::scanner::{PunctuationKind, TokenKind};
use crate::schema::Structure;

impl<'element> Checkable<'element> for Element<'element> {
    fn infer(&self) -> Type<'element> {
        match self.kind.clone() {
            ElementKind::Literal(literal) => {
                Type::unit(literal.span)
            }
            ElementKind::Procedural(_) => {
                Type::unit(self.span)
            }
            ElementKind::Delimited(delimited) => {
                match (delimited.start.kind, delimited.separator.map(|token| token.kind), delimited.end.kind) {
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) | (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        Type::new(
                            TypeKind::Tuple {
                                items: delimited.items
                                    .iter()
                                    .map(|item| {
                                        item.infer()
                                    })
                                    .collect::<Vec<_>>()
                            },
                            self.span,
                        )
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) | (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) => {
                        if let Some(item) = delimited.items.last() {
                            item.infer()
                        } else {
                            Type::unit(self.span)
                        }
                    }

                    _ => {
                        Type::unit(self.span)
                    }
                }
            }
            ElementKind::Unary(unary) => {
                (&*unary.operand).infer()
            }
            ElementKind::Binary(binary) => {
                (&*binary.left).infer()
            }
            ElementKind::Index(_) => {
                Type::unit(self.span)
            }
            ElementKind::Invoke(_) => {
                Type::unit(self.span)
            }
            ElementKind::Construct(construct) => {
                let structure = Structure::new(
                    Str::from(construct.target.brand().unwrap().to_string()),
                    construct.members.iter().map(|field| Box::new(field.infer())).collect::<Vec<_>>(),
                );

                Type::new(TypeKind::Structure(structure), self.span)
            }
            ElementKind::Conditional(_) => {
                Type::unit(self.span)
            }
            ElementKind::While(_) => {
                Type::unit(self.span)
            }
            ElementKind::Cycle(_) => {
                Type::unit(self.span)
            }
            ElementKind::Symbolize(_) => {
                Type::unit(self.span)
            }
            ElementKind::Return(_) => {
                Type::unit(self.span)
            }
            ElementKind::Break(_) => {
                Type::unit(self.span)
            }
            ElementKind::Continue(_) => {
                Type::unit(self.span)
            }
        }
    }
}