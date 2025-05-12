use std::path::PathBuf;
use std::sync::Arc;
use crate::axo_form::{Form, Former, Pattern};
use crate::{is_alphabetic, is_numeric, Lexer, Peekable, Token, TokenKind};
use crate::axo_lexer::{OperatorLexer, PunctuationLexer};
use crate::float::FloatLiteral;

fn extract_chars(form: &Form<char, Token>) -> String {
    match form {
        Form::Raw(c) => c.to_string(),
        Form::Single(_) => String::new(), 
        Form::Multiple(items) => {
            let mut result = String::new();
            for item in items {
                result.push_str(&extract_chars(&item.form));
            }
            result
        }
    }
}

fn line_comment() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Sequence(vec![
            Pattern::Precise('/'),
            Pattern::Precise('/'),
            Pattern::Repeat {
                pattern: Box::new(Pattern::Predicate(Arc::new(|c| *c != '\n'))),
                minimum: 0,
                maximum: None,
            },
        ])),
        transform: Arc::new(|chars, span| {
            let mut content = String::new();
            
            let end_idx = if chars.len() > 3 && extract_chars(&chars[chars.len()-1].form) == "\n" {
                chars.len() - 1
            } else {
                chars.len()
            };

            for formed in &chars[2..end_idx] {
                content.push_str(&extract_chars(&formed.form));
            }
            
            Ok(Token::new(TokenKind::Comment(content), span))
        }),
    }
}

fn multiline_comment() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Sequence(vec![
            Pattern::Precise('/'),
            Pattern::Precise('*'),
            Pattern::Repeat {
                pattern: Box::new(Pattern::OneOf(vec![
                    Pattern::Predicate(Arc::new(|c| *c != '*')),
                    
                    Pattern::Sequence(vec![
                        Pattern::Precise('*'),
                        Pattern::Lookup(Box::new(Pattern::Negate(Box::new(Pattern::Precise('/'))))),
                    ]),
                ])),
                minimum: 0,
                maximum: None,
            },
            Pattern::Precise('*'),
            Pattern::Precise('/'),
        ])),
        transform: Arc::new(|chars, span| {
            let mut content = String::new();
            
            for formed in &chars[2..chars.len()-2] {
                content.push_str(&extract_chars(&formed.form));
            }
            Ok(Token::new(TokenKind::Comment(content), span))
        }),
    }
}

fn number() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::OneOf(vec![
            Pattern::Sequence(vec![
                Pattern::Predicate(Arc::new(|c| is_numeric(*c))),
                Pattern::Repeat {
                    pattern: Box::new(Pattern::OneOf(vec![
                        Pattern::Predicate(Arc::new(|c| is_numeric(*c))),
                        Pattern::Precise('_'),
                    ])),
                    minimum: 0,
                    maximum: None,
                },
                Pattern::Optional(Box::new(Pattern::Sequence(vec![
                    Pattern::Precise('.'),
                    Pattern::Repeat {
                        pattern: Box::new(Pattern::OneOf(vec![
                            Pattern::Predicate(Arc::new(|c| is_numeric(*c))),
                            Pattern::Precise('_'),
                        ])),
                        minimum: 0,
                        maximum: None,
                    },
                ]))),
                Pattern::Optional(Box::new(Pattern::Sequence(vec![
                    Pattern::Predicate(Arc::new(|c| *c == 'e' || *c == 'E')),
                    Pattern::Optional(Box::new(Pattern::Predicate(Arc::new(|c| *c == '+' || *c == '-')))),
                    Pattern::Repeat {
                        pattern: Box::new(Pattern::Predicate(Arc::new(|c| is_numeric(*c)))),
                        minimum: 1,
                        maximum: None,
                    },
                ]))),
            ]),
            Pattern::Sequence(vec![
                Pattern::Precise('0'),
                Pattern::Predicate(Arc::new(|c| matches!(*c, 'x' | 'X' | 'o' | 'O' | 'b' | 'B'))),
                Pattern::Repeat {
                    pattern: Box::new(Pattern::OneOf(vec![
                        Pattern::Predicate(Arc::new(|c| {
                            let c = *c;
                            c.is_digit(16) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
                        })),
                        Pattern::Precise('_'),
                    ])),
                    minimum: 1,
                    maximum: None,
                },
            ]),
        ])),
        transform: Arc::new(|chars, span| {
            let mut number = String::new();
            for formed in &chars {
                let chars_str = extract_chars(&formed.form);
                for c in chars_str.chars() {
                    if c != '_' {
                        number.push(c);
                    }
                }
            }

            if number.contains('.') || number.to_lowercase().contains('e') {
                let parser = crate::axo_rune::parser::<f64>();
                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Float(FloatLiteral::from(num)), span)),
                    Err(_) => Err('0'),
                }
            } else {
                let parser = crate::axo_rune::parser::<i128>();
                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), span)),
                    Err(_) => Err('0'),
                }
            }
        }),
    }
}

fn identifier() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Sequence(vec![
            Pattern::Predicate(Arc::new(|c| is_alphabetic(*c) || *c == '_')),
            Pattern::Repeat {
                pattern: Box::new(Pattern::Predicate(Arc::new(|c| is_alphabetic(*c) || is_numeric(*c) || *c == '_'))),
                minimum: 0,
                maximum: None,
            },
        ])),
        transform: Arc::new(|chars, span| {
            let mut ident = String::new();

            for formed in &chars {
                ident.push_str(&extract_chars(&formed.form));
            }

            Ok(Token::new(
                TokenKind::from_str(&ident).unwrap_or(TokenKind::Identifier(ident)),
                span,
            ))
        }),
    }
}

fn quoted_string() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Sequence(vec![
            Pattern::Precise('"'),
            Pattern::Repeat {
                pattern: Box::new(Pattern::OneOf(vec![
                    Pattern::Sequence(vec![
                        Pattern::Precise('\\'),
                        Pattern::Predicate(Arc::new(|_| true)),
                    ]),
                    Pattern::Predicate(Arc::new(|c| *c != '"' && *c != '\\')),
                ])),
                minimum: 0,
                maximum: None,
            },
            Pattern::Precise('"'),
        ])),
        transform: Arc::new(|chars, span| {
            let mut content = String::new();
            let mut i = 1;

            let mut flat_chars = Vec::new();
            for formed in &chars {
                let s = extract_chars(&formed.form);
                flat_chars.extend(s.chars());
            }

            while i < flat_chars.len() - 1 {
                let c = flat_chars[i];
                if c == '\\' {
                    i += 1;
                    if i < flat_chars.len() - 1 {
                        let escaped_c = flat_chars[i];
                        content.push(match escaped_c {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            '0' => '\0',
                            'x' => {
                                i += 1;
                                let mut hex = String::new();
                                for _ in 0..2 {
                                    if i < flat_chars.len() - 1 {
                                        let hex_c = flat_chars[i];
                                        if hex_c.is_digit(16) {
                                            hex.push(hex_c);
                                            i += 1;
                                        } else {
                                            return Err('0');
                                        }
                                    } else {
                                        return Err('0');
                                    }
                                }
                                i -= 1;
                                u32::from_str_radix(&hex, 16)
                                    .ok()
                                    .and_then(core::char::from_u32)
                                    .unwrap_or('\0')
                            }
                            'u' => {
                                i += 1;
                                if i < flat_chars.len() - 1 {
                                    if flat_chars[i] == '{' {
                                        i += 1;
                                        let mut hex = String::new();
                                        while i < flat_chars.len() - 1 {
                                            let hex_c = flat_chars[i];
                                            if hex_c == '}' {
                                                break;
                                            }
                                            hex.push(hex_c);
                                            i += 1;
                                        }
                                        if i < flat_chars.len() - 1 {
                                            if flat_chars[i] == '}' {
                                                u32::from_str_radix(&hex, 16)
                                                    .ok()
                                                    .and_then(core::char::from_u32)
                                                    .unwrap_or('\0')
                                            } else {
                                                return Err('0');
                                            }
                                        } else {
                                            return Err('0');
                                        }
                                    } else {
                                        return Err('0');
                                    }
                                } else {
                                    return Err('0');
                                }
                            }
                            _ => escaped_c,
                        });
                    }
                } else {
                    content.push(c);
                }
                i += 1;
            }
            Ok(Token::new(TokenKind::String(content), span))
        }),
    }
}

fn backtick_string() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Sequence(vec![
            Pattern::Precise('`'),
            Pattern::Repeat {
                pattern: Box::new(Pattern::Predicate(Arc::new(|c| *c != '`'))),
                minimum: 0,
                maximum: None,
            },
            Pattern::Precise('`'),
        ])),
        transform: Arc::new(|chars, span| {
            let mut content = String::new();

            for formed in &chars[1..chars.len() - 1] {
                content.push_str(&extract_chars(&formed.form));
            }

            Ok(Token::new(TokenKind::String(content), span))
        }),
    }
}

fn character() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Sequence(vec![
            Pattern::Precise('\''),
            Pattern::OneOf(vec![
                Pattern::Sequence(vec![
                    Pattern::Precise('\\'),
                    Pattern::Predicate(Arc::new(|_| true)),
                ]),
                Pattern::Predicate(Arc::new(|c| *c != '\'' && *c != '\\')),
            ]),
            Pattern::Precise('\''),
        ])),
        transform: Arc::new(|chars, span| {
            let mut flat_chars = Vec::new();
            for formed in &chars {
                let s = extract_chars(&formed.form);
                flat_chars.extend(s.chars());
            }

            if flat_chars.len() < 3 {
                return Err('0');
            }

            let ch = if flat_chars[1] == '\\' {
                if flat_chars.len() < 4 {
                    return Err('0');
                }
                let escaped_c = flat_chars[2];
                match escaped_c {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '0' => '\0',
                    'x' => {
                        if flat_chars.len() < 6 {
                            return Err('0');
                        }
                        let h1 = flat_chars[3];
                        let h2 = flat_chars[4];
                        if h1.is_digit(16) && h2.is_digit(16) {
                            let hex = format!("{}{}", h1, h2);
                            u32::from_str_radix(&hex, 16)
                                .ok()
                                .and_then(core::char::from_u32)
                                .unwrap_or('\0')
                        } else {
                            return Err('0');
                        }
                    }
                    'u' => {
                        if flat_chars.len() < 5 || flat_chars[3] != '{' {
                            return Err('0');
                        }
                        let mut i = 4;
                        let mut hex = String::new();
                        while i < flat_chars.len() && flat_chars[i] != '}' {
                            hex.push(flat_chars[i]);
                            i += 1;
                        }
                        if i >= flat_chars.len() || flat_chars[i] != '}' {
                            return Err('0');
                        }
                        u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(core::char::from_u32)
                            .unwrap_or('\0')
                    }
                    _ => escaped_c,
                }
            } else {
                flat_chars[1]
            };

            Ok(Token::new(TokenKind::Character(ch), span))
        }),
    }
}

fn operator() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Repeat {
            pattern: Box::new(Pattern::Predicate(Arc::new(|c| {
                c.is_operator()
            }))),
            minimum: 1,
            maximum: None,
        }),
        transform: Arc::new(|chars, span| {
            let mut op = String::new();
            for formed in &chars {
                op.push_str(&extract_chars(&formed.form));
            }
            Ok(Token::new(TokenKind::Operator(op.to_operator()), span))
        }),
    }
}

fn punctuation() -> Pattern<char, Token> {
    Pattern::Transform {
        pattern: Box::new(Pattern::Predicate(Arc::new(|c| {
            c.is_punctuation()
        }))),
        transform: Arc::new(|chars, span| {
            let ch_str = extract_chars(&chars[0].form);
            if let Some(ch) = ch_str.chars().next() {
                Ok(Token::new(TokenKind::Punctuation(ch.to_punctuation()), span))
            } else {
                Err('0')
            }
        }),
    }
}

pub fn pattern() -> Pattern<char, Token> {
    Pattern::Repeat {
        pattern: Box::new(
            Pattern::OneOf(vec![
                line_comment(),
                multiline_comment(),
                number(),
                identifier(),
                quoted_string(),
                backtick_string(),
                character(),
                operator(),
                punctuation(),
            ])
        ),
        minimum: 0,
        maximum: None,
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(input.to_string(), PathBuf::new());

    while lexer.peek().is_some() {
        match lexer.form(pattern()) {
            Ok(formed) => {
                match formed.form {
                    Form::Single(token) => {
                        tokens.push(token);
                    },
                    Form::Multiple(multi) => {
                        for item in multi {
                            match item.form {
                                Form::Single(token) => {
                                    tokens.push(token);
                                },
                                Form::Multiple(sub_multi) => {
                                    for sub_item in sub_multi {
                                        if let Form::Single(token) = sub_item.form {
                                            tokens.push(token);
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    Form::Raw(_) => {
                    }
                }
            },
            Err(error) => {
                return if let Some(c) = lexer.peek().cloned() {
                    lexer.next();

                    Err(format!(
                        "Error at position {:?}: character '{}' - {:?}",
                        lexer.position(), c, error
                    ))
                } else {
                    Err("Unexpected end of input".to_string())
                }
            }
        }
    }

    Ok(tokens)
}