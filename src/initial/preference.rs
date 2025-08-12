use {
    crate::{
        data::{
            any::{Any, TypeId},
            string::Str,
            memory,
        },
        internal::{
            hash::{Hash, Hasher, DefaultHasher},
        },
        formation::{
            classifier::Classifier,
            form::Form,
        },
        parser::Symbolic,
        scanner::{Token, TokenKind, OperatorKind},
        tracker::{Span, Spanned},
    },
    super::{Initializer, InitialError},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Preference<'preference> {
    pub target: Token<'preference>,
    pub value: Token<'preference>,
    pub span: Span<'preference>,
}

impl<'preference> Spanned<'preference> for Preference<'preference> {
    fn borrow_span(&self) -> Span<'preference> {
        self.span.clone()
    }

    fn span(self) -> Span<'preference> {
        self.span
    }
}

impl<'preference> Preference<'preference> {
    pub fn new(target: Token<'preference>, value: Token<'preference>) -> Self {
        let span = Span::merge(&target.borrow_span(), &value.borrow_span());

        Self {
            target,
            value,
            span
        }
    }
}

impl Symbolic for Preference<'static> {
    fn brand(&self) -> Option<Token<'static>> {
        Some(unsafe { memory::transmute(self.target.clone()) })
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(Self {
            target: self.target.clone(),
            value: self.value.clone(),
            span: self.span.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Symbolic) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'initializer> Initializer<'initializer> {
    pub fn verbosity() -> Classifier<'initializer, Token<'initializer>, Preference<'initializer>, InitialError<'initializer>> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Operator(operator) = &token.kind {
                    if *operator == OperatorKind::Minus {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }).with_ignore(),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    identifier == "v" || identifier == "verbose"
                } else {
                    false
                }
            }).with_transform(move |form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
                let identifier = form.collect_inputs()[0].clone();
                let span = identifier.span();

                Ok(Form::Input(Token::new(TokenKind::Identifier(Str::from("Verbosity")), span)))
            })
        ]).with_transform(move |form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
            let identifier: Token<'initializer> = form.collect_inputs()[0].clone();
            let span: Span<'initializer> = identifier.clone().span();

            Ok(Form::output(Preference::new(identifier, Token::new(TokenKind::Boolean(true), span))))
        })
    }

    pub fn path() -> Classifier<'initializer, Token<'initializer>, Preference<'initializer>, InitialError<'initializer>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        *operator == OperatorKind::Minus
                    } else {
                        false
                    }
                }).with_ignore(),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "p" || identifier == "path"
                    } else {
                        false
                    }
                }).with_transform(|form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    Ok(Form::Input(Token::new(TokenKind::Identifier(Str::from("Path")), span)))
                }),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        matches!(token.kind, TokenKind::Identifier(_))
                    }),
                    Classifier::repetition(
                        Classifier::sequence([
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Operator(OperatorKind::Slash))
                            }),
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Identifier(_))
                            }),
                        ]),
                        0,
                        None
                    ),
                    Classifier::sequence([
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Operator(OperatorKind::Dot))
                        }),
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Identifier(_))
                        }),
                    ]).as_optional()
                ])
            ]),
            |form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
                let inputs = form.collect_inputs();
                let identifier = inputs[0].clone();
                let span = identifier.clone().span();
                let mut path = String::new();

                for input in inputs.iter().skip(1) {
                    match &input.kind {
                        TokenKind::Identifier(identifier) => {
                            path.push_str(&identifier);
                        }
                        TokenKind::Operator(OperatorKind::Slash) => {
                            path.push('/');
                        }
                        TokenKind::Operator(OperatorKind::Dot) => {
                            path.push('.');
                        }
                        _ => {}
                    }
                }

                Ok(Form::output(Preference::new(identifier.clone(), Token::new(TokenKind::Identifier(Str::from(path)), span))))
            }
        )
    }
}
