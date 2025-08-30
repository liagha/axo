mod show;

pub use {
    show::Show,
    core::{
        fmt::{
            Debug, Display,
            Formatter, Result,
            Write
        },
    },
};

use {
    broccli::{Color, TextStyle},
    
    crate::{
        data::Str,
        scanner::{
            Token, TokenKind, 
            PunctuationKind, 
        },
    }
};

pub fn format_tokens<'token>(tokens: &[Token<'token>]) -> Str<'token> {
    tokens
        .iter()
        .enumerate()
        .map(|(i, token)| {
            let token_str = match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => format!(
                    "↓ {:#?} | {:#?} ↓\n",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Punctuation(_) => format!(
                    "{:#?} | {:#?}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Operator(_) => format!(
                    "{:#?} | {:#?}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Orange)
                    .to_string(),
                _ => format!("{:#?} | {:#?}", token, token.span),
            };
            if i < tokens.len() - 1
                && !matches!(token.kind, TokenKind::Punctuation(PunctuationKind::Newline))
            {
                format!("{}, ", token_str)
            } else {
                token_str
            }
        })
        .collect()
}
