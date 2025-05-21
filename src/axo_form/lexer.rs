use std::sync::Arc;
use crate::axo_form::{Action, Form, Former, Pattern};
use crate::{is_alphabetic, is_numeric, Lexer, Peekable, Token, TokenKind};
use crate::axo_lexer::{LexError, OperatorLexer, PunctuationLexer};
use crate::axo_lexer::error::ErrorKind;
use crate::float::FloatLiteral;

fn extract(form: &Form<char, Token, LexError>) -> String {
    match form {
        Form::Empty | Form::Single(_) => String::new(),
        Form::Raw(c) => c.to_string(),
        Form::Multiple(items) => {
            let mut result = String::new();
            for item in items {
                result.push_str(&extract(&item.form));
            }
            result
        }
        
        Form::Error(e) => {
            println!("\nExtract Error: {:?}!\n", e);
            
            String::new()
        }
    }
}

fn line_comment() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(
                Pattern::sequence(
                    [
                        Pattern::literal('/'),
                        Pattern::literal('/'),
                    ]
                )
            ),
            Pattern::repeat(
                Pattern::predicate(Arc::new(|c| *c != '\n')),
                0,
                None,
            ),
        ]),
        Arc::new(|chars, span| {
            let mut content = String::new();

            for formed in &chars {
                content.push_str(&extract(&formed.form));
            }

            Ok(Token::new(TokenKind::Comment(content.to_string()), span))
        }),
    )
}

fn multiline_comment() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(
                Pattern::sequence(
                    [
                        Pattern::literal('/'),
                        Pattern::literal('*'),
                    ]
                )
            ),
            Pattern::repeat(
                Pattern::negate(
                    Pattern::ignore(
                        Pattern::sequence(
                            [
                                Pattern::literal('*'),
                                Pattern::literal('/'),
                            ]
                        )
                    ),
                ),
                0,
                None,
            ),
            Pattern::required(
                Pattern::ignore(
                    Pattern::sequence(
                        [
                            Pattern::literal('*'),
                            Pattern::literal('/'),
                        ]
                    )
                ),
                Action::Error(
                    Arc::new(|span| LexError::new(ErrorKind::Custom("unclosed comment".to_string()), span))
                )
            ),
        ]),
        Arc::new(|chars, span| {
            println!("Comment: {:?}", chars);
            let mut content = String::new();

            for formed in &chars {
                content.push_str(&extract(&formed.form));
            }

            Ok(Token::new(TokenKind::Comment(content.to_string()), span))
        }),
    )
}

fn hex_number() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::literal('0'),
            Pattern::alternative([Pattern::literal('x'), Pattern::literal('X')]),
            Pattern::repeat(
                Pattern::alternative([
                    Pattern::predicate(Arc::new(|c| {
                        is_numeric(*c) || ('a'..='f').contains(c) || ('A'..='F').contains(c)
                    })),
                    Pattern::literal('_'),
                ]),
                1,
                None,
            ),
        ]),
        Arc::new(|chars, span| {
            let mut number = String::new();
            for formed in &chars {
                let chars_str = extract(&formed.form);
                for c in chars_str.chars() {
                    if c != '_' {
                        number.push(c);
                    }
                }
            }

            let parser = crate::axo_rune::parser::<i128>();
            match parser.parse(&number) {
                Ok(num) => Ok(Token::new(TokenKind::Integer(num), span)),
                Err(_) => Err('0'),
            }
        }),
    )
}

fn binary_number() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::literal('0'),
            Pattern::alternative([Pattern::literal('b'), Pattern::literal('B')]),
            Pattern::repeat(
                Pattern::alternative([
                    Pattern::predicate(Arc::new(|c| *c == '0' || *c == '1')),
                    Pattern::literal('_'),
                ]),
                1,
                None,
            ),
        ]),
        Arc::new(|chars, span| {
            let mut number = String::new();
            for formed in &chars {
                let chars_str = extract(&formed.form);
                for c in chars_str.chars() {
                    if c != '_' {
                        number.push(c);
                    }
                }
            }

            let parser = crate::axo_rune::parser::<i128>();
            match parser.parse(&number) {
                Ok(num) => Ok(Token::new(TokenKind::Integer(num), span)),
                Err(_) => Err('0'),
            }
        }),
    )
}

fn octal_number() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::literal('0'),
            Pattern::alternative([Pattern::literal('o'), Pattern::literal('O')]),
            Pattern::repeat(
                Pattern::alternative([
                    Pattern::predicate(Arc::new(|c| ('0'..='7').contains(c))),
                    Pattern::literal('_'),
                ]),
                1,
                None,
            ),
        ]),
        Arc::new(|chars, span| {
            let mut number = String::new();
            for formed in &chars {
                let chars_str = extract(&formed.form);
                for c in chars_str.chars() {
                    if c != '_' {
                        number.push(c);
                    }
                }
            }

            let parser = crate::axo_rune::parser::<i128>();
            match parser.parse(&number) {
                Ok(num) => Ok(Token::new(TokenKind::Integer(num), span)),
                Err(_) => Err('0'),
            }
        }),
    )
}

fn decimal_number() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::predicate(Arc::new(|c| is_numeric(*c))),
            Pattern::repeat(
                Pattern::alternative([
                    Pattern::predicate(Arc::new(|c| is_numeric(*c))),
                    Pattern::literal('_'),
                ]),
                0,
                None,
            ),
            Pattern::optional(Pattern::sequence([
                Pattern::literal('.'),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(Arc::new(|c| is_numeric(*c))),
                        Pattern::literal('_'),
                    ]),
                    0,
                    None,
                ),
            ])),
            Pattern::optional(Pattern::sequence([
                Pattern::predicate(Arc::new(|c| *c == 'e' || *c == 'E')),
                Pattern::optional(Pattern::predicate(Arc::new(|c| *c == '+' || *c == '-'))),
                Pattern::repeat(
                    Pattern::predicate(Arc::new(|c| is_numeric(*c))),
                    1,
                    None,
                ),
            ])),
        ]),
        Arc::new(|chars, span| {
            let mut number = String::new();
            for formed in &chars {
                let chars_str = extract(&formed.form);
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
    )
}

fn number() -> Pattern<char, Token, LexError> {
    Pattern::alternative([
        hex_number(),
        binary_number(),
        octal_number(),
        decimal_number(),
    ])
}

fn identifier() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::predicate(Arc::new(|c| is_alphabetic(*c) || *c == '_')),
            Pattern::repeat(
                Pattern::predicate(Arc::new(|c| is_alphabetic(*c) || is_numeric(*c) || *c == '_')),
                0,
                None,
            ),
        ]),
        Arc::new(|chars, span| {
            let mut ident = String::new();

            for formed in &chars {
                ident.push_str(&extract(&formed.form));
            }

            Ok(Token::new(
                TokenKind::from_str(&ident).unwrap_or(TokenKind::Identifier(ident)),
                span,
            ))
        }),
    )
}

fn quoted_string() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::literal('"'),
            Pattern::repeat(
                Pattern::alternative([
                    Pattern::sequence([
                        Pattern::literal('\\'),
                        Pattern::predicate(Arc::new(|_| true)),
                    ]),
                    Pattern::predicate(Arc::new(|c| *c != '"' && *c != '\\')),
                ]),
                0,
                None,
            ),
            Pattern::literal('"'),
        ]),
        Arc::new(|chars, span| {
            let mut content = String::new();
            let mut i = 1;

            let mut flat_chars = Vec::new();
            for formed in &chars {
                let s = extract(&formed.form);
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
    )
}

fn backtick_string() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::literal('`'),
            Pattern::repeat(
                Pattern::predicate(Arc::new(|c| *c != '`')),
                0,
                None,
            ),
            Pattern::literal('`'),
        ]),
        Arc::new(|chars, span| {
            let mut content = String::new();

            for formed in &chars[1..chars.len() - 1] {
                content.push_str(&extract(&formed.form));
            }

            Ok(Token::new(TokenKind::String(content), span))
        }),
    )
}

fn character() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::literal('\''),
            Pattern::alternative([
                Pattern::sequence([
                    Pattern::literal('\\'),
                    Pattern::predicate(Arc::new(|_| true)),
                ]),
                Pattern::predicate(Arc::new(|c| *c != '\'' && *c != '\\')),
            ]),
            Pattern::literal('\''),
        ]),
        Arc::new(|chars, span| {
            let mut flat_chars = Vec::new();
            for formed in &chars {
                let s = extract(&formed.form);
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
    )
}

fn operator() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::repeat(
            Pattern::predicate(Arc::new(|c: &char| {
                c.is_operator()
            })),
            1,
            None,
        ),
        Arc::new(|chars, span| {
            let mut op = String::new();
            for formed in &chars {
                op.push_str(&extract(&formed.form));
            }
            Ok(Token::new(TokenKind::Operator(op.to_operator()), span))
        }),
    )
}

fn punctuation() -> Pattern<char, Token, LexError> {
    Pattern::transform(
        Pattern::predicate(Arc::new(|c: &char| {
            c.is_punctuation()
        })),
        Arc::new(|chars, span| {
            let ch_str = extract(&chars[0].form);
            if let Some(ch) = ch_str.chars().next() {
                Ok(Token::new(TokenKind::Punctuation(ch.to_punctuation()), span))
            } else {
                Err('0')
            }
        }),
    )
}

pub fn pattern() -> Pattern<char, Token, LexError> {
    Pattern::repeat(
        Pattern::alternative([
            line_comment(),
            multiline_comment(),
            identifier(),
            number(),
            quoted_string(),
            backtick_string(),
            character(),
            operator(),
            punctuation(),
        ]),
        0,
        None,
    )
}

impl Lexer {
    pub fn lex(&mut self) -> (Vec<Token>, Vec<LexError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while self.peek().is_some() {
            let form = self.form(pattern());

            match form.form {
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
                            Form::Error(err) => {
                                errors.push(err);
                            }
                            _ => {}
                        }
                    }
                },
                
                Form::Error(err) => {
                    errors.push(err);
                }

                Form::Empty | Form::Raw(_) => {}
            }
        }

        (tokens, errors)
    }
}
