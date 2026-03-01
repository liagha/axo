use {
    super::{InitializeError, Initializer},
    crate::{
        data::Str,
        formation::{classifier::Classifier, form::Form},
        internal::hash::Hash,
        scanner::{OperatorKind, Token, TokenKind},
        tracker::{Span, Spanned},
    },
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Preference<'preference> {
    pub target: Token<'preference>,
    pub value: Token<'preference>,
    pub span: Span<'preference>,
}

impl<'preference> Preference<'preference> {
    pub fn new(target: Token<'preference>, value: Token<'preference>) -> Self {
        let span = Span::merge(&target.borrow_span(), &value.borrow_span());

        Self {
            target,
            value,
            span,
        }
    }
}

impl<'initializer> Initializer<'initializer> {
    fn path_string(tokens: Vec<Token<'initializer>>) -> String {
        let mut result = String::new();
        for input in tokens {
            match input.kind {
                TokenKind::Identifier(identifier) => result.push_str(&identifier),
                TokenKind::String(value) => result.push_str(value.as_str().unwrap_or("")),
                TokenKind::Integer(value) => result.push_str(&value.to_string()),
                TokenKind::Operator(OperatorKind::Slash) => result.push('/'),
                TokenKind::Operator(OperatorKind::Dot) => result.push('.'),
                TokenKind::Operator(OperatorKind::Backslash) => result.push('\\'),
                TokenKind::Operator(OperatorKind::Colon) => result.push(':'),
                _ => {}
            }
        }
        result
    }

    fn path_value() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::Identifier(_)
                        | TokenKind::String(_)
                        | TokenKind::Integer(_)
                        | TokenKind::Operator(OperatorKind::Slash)
                        | TokenKind::Operator(OperatorKind::Backslash)
                        | TokenKind::Operator(OperatorKind::Dot)
                )
            }),
            Classifier::repetition(
                Classifier::predicate(|token: &Token| {
                    matches!(
                        token.kind,
                        TokenKind::Identifier(_)
                            | TokenKind::String(_)
                            | TokenKind::Integer(_)
                            | TokenKind::Operator(OperatorKind::Slash)
                            | TokenKind::Operator(OperatorKind::Backslash)
                            | TokenKind::Operator(OperatorKind::Dot)
                            | TokenKind::Operator(OperatorKind::Colon)
                    )
                }),
                0,
                None,
            ),
        ])
    }

    fn path_preference(
        name: Str<'initializer>,
        matcher: fn(&Str<'initializer>) -> bool,
    ) -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
                })
                .with_ignore(),
                Classifier::predicate(move |token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        matcher(identifier)
                    } else {
                        false
                    }
                })
                .with_transform(
                    move |form: Form<
                        'initializer,
                        Token<'initializer>,
                        Preference,
                        InitializeError<'initializer>,
                    >| {
                        let identifier = form.collect_inputs()[0].clone();
                        let span = identifier.span();

                        Ok(Form::Input(Token::new(
                            TokenKind::Identifier(name.clone()),
                            span,
                        )))
                    },
                ),
                Self::path_value(),
            ]),
            |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let forms = form.as_forms();
                let identifier = forms[0].unwrap_input().clone();
                let span = identifier.clone().span();
                let result = Self::path_string(forms[1].collect_inputs());

                Ok(Form::output(Preference::new(
                    identifier,
                    Token::new(TokenKind::Identifier(Str::from(result)), span),
                )))
            },
        )
    }

    pub fn verbosity() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
            })
            .with_ignore(),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    identifier == "v" || identifier == "verbosity"
                } else {
                    false
                }
            })
            .with_transform(
                move |form: Form<
                    'initializer,
                    Token<'initializer>,
                    Preference,
                    InitializeError<'initializer>,
                >| {
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    Ok(Form::Input(Token::new(
                        TokenKind::Identifier(Str::from("Verbosity")),
                        span,
                    )))
                },
            ),
            Classifier::predicate(|token: &Token| matches!(token.kind, TokenKind::Integer(_))),
        ])
        .with_transform(
            move |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let identifier: Token<'initializer> = form.collect_inputs()[0].clone();
                let value: Token<'initializer> = form.collect_inputs()[1].clone();

                Ok(Form::output(Preference::new(identifier, value)))
            },
        )
    }

    pub fn code() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "o" || identifier == "output"
                    } else {
                        false
                    }
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(OperatorKind::Dot))
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "ir"
                    } else {
                        false
                    }
                })
                .with_transform(
                    |form: Form<
                        'initializer,
                        Token<'initializer>,
                        Preference,
                        InitializeError<'initializer>,
                    >| {
                        let identifier = form.collect_inputs()[0].clone();
                        let span = identifier.span();
                        Ok(Form::Input(Token::new(
                            TokenKind::Identifier(Str::from("OutputCode")),
                            span,
                        )))
                    },
                ),
                Self::path_value(),
            ]),
            |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let forms = form.as_forms();
                let identifier = forms[0].unwrap_input().clone();
                let span = identifier.clone().span();
                let result = Self::path_string(forms[1].collect_inputs());

                Ok(Form::output(Preference::new(
                    identifier,
                    Token::new(TokenKind::Identifier(Str::from(result)), span),
                )))
            },
        )
    }

    pub fn binary() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "o" || identifier == "output"
                    } else {
                        false
                    }
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(OperatorKind::Dot))
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "executable" || identifier == "exec"
                    } else {
                        false
                    }
                })
                .with_transform(
                    |form: Form<
                        'initializer,
                        Token<'initializer>,
                        Preference,
                        InitializeError<'initializer>,
                    >| {
                        let identifier = form.collect_inputs()[0].clone();
                        let span = identifier.span();
                        Ok(Form::Input(Token::new(
                            TokenKind::Identifier(Str::from("OutputBinary")),
                            span,
                        )))
                    },
                ),
                Self::path_value(),
            ]),
            |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let forms = form.as_forms();
                let identifier = forms[0].unwrap_input().clone();
                let span = identifier.clone().span();
                let result = Self::path_string(forms[1].collect_inputs());

                Ok(Form::output(Preference::new(
                    identifier,
                    Token::new(TokenKind::Identifier(Str::from(result)), span),
                )))
            },
        )
    }

    pub fn run() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
            })
            .with_ignore(),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    identifier == "r" || identifier == "run"
                } else {
                    false
                }
            })
            .with_transform(
                move |form: Form<
                    'initializer,
                    Token<'initializer>,
                    Preference,
                    InitializeError<'initializer>,
                >| {
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    Ok(Form::Input(Token::new(
                        TokenKind::Identifier(Str::from("Run")),
                        span,
                    )))
                },
            ),
        ])
        .with_transform(
            move |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let identifier: Token<'initializer> = form.collect_inputs()[0].clone();
                let span: Span<'initializer> = identifier.clone().span();

                Ok(Form::output(Preference::new(
                    identifier,
                    Token::new(TokenKind::Boolean(true), span),
                )))
            },
        )
    }

    pub fn bootstrap() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                matches!(token.kind, TokenKind::Operator(OperatorKind::Minus))
            })
            .with_ignore(),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    identifier == "bootstrap"
                } else {
                    false
                }
            })
            .with_transform(
                move |form: Form<
                    'initializer,
                    Token<'initializer>,
                    Preference,
                    InitializeError<'initializer>,
                >| {
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    Ok(Form::Input(Token::new(
                        TokenKind::Identifier(Str::from("Bootstrap")),
                        span,
                    )))
                },
            ),
        ])
        .with_transform(
            move |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let identifier: Token<'initializer> = form.collect_inputs()[0].clone();
                let span: Span<'initializer> = identifier.clone().span();

                Ok(Form::output(Preference::new(
                    identifier,
                    Token::new(TokenKind::Boolean(true), span),
                )))
            },
        )
    }

    pub fn input() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Self::path_preference(Str::from("Input"), |identifier| {
            identifier == "i" || identifier == "input"
        })
    }

    pub fn implicit_input() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::with_transform(
            Self::path_value(),
            |form: Form<
                'initializer,
                Token<'initializer>,
                Preference,
                InitializeError<'initializer>,
            >| {
                let inputs = form.collect_inputs();
                if inputs.is_empty() {
                    return Ok(Form::blank());
                }

                let span = inputs[0].clone().span();
                let result = Self::path_string(inputs);

                Ok(Form::output(Preference::new(
                    Token::new(TokenKind::Identifier(Str::from("Input")), span),
                    Token::new(TokenKind::Identifier(Str::from(result)), span),
                )))
            },
        )
    }

    pub fn output() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Self::path_preference(Str::from("Output"), |identifier| {
            identifier == "o" || identifier == "output"
        })
    }
}
