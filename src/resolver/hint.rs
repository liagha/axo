use crate::{
    format::{
        Display, Formatter,
        Show, Verbosity,
        Result,
    },
    data::Str,
};

#[derive(Clone)]
pub enum HintKind<'hint> {
    SimilarBrand {
        candidate: Str<'hint>,
        how: String,
    },
}

impl<'hint> Display for HintKind<'hint> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            HintKind::SimilarBrand { candidate, how } => {
                write!(f, "did you mean `{}`? they {}", candidate.format(Verbosity::Minimal), how)
            }
        }
    }
}
