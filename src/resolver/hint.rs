use crate::{
    format::{self, Display, Formatter},
    scanner::Token,
};

#[derive(Clone, Debug)]
pub enum HintKind<'hint> {
    SimilarBrand {
        candidate: Token<'hint>,
        how: String,
    },
}

impl<'hint> Display for HintKind<'hint> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self {
            HintKind::SimilarBrand { candidate, how } => {
                write!(f, "did you mean `{:?}`? they {}", candidate, how)
            }
        }
    }
}
