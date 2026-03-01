mod show;

pub use {
    core::fmt::{Debug, Display, Formatter, Result},
    show::Show,
};

use {
    crate::{
        data::Str,
        scanner::{PunctuationKind, Token, TokenKind},
    },
    broccli::{Color, TextStyle},
};
