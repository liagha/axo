use {
    matchete::{
        Resembler,
    },
    crate::{
        scanner::Token,
        data::thread::{
            Arc, Mutex,
        },
        format::{Display, Formatter},
    }
};

#[derive(Clone, Debug)]
pub enum ResolveHint<'hint> {
    SimilarBrand { candidate: Token<'hint>, effective: Arc<Mutex<dyn Resembler<String, String, ()>>> },
    Parameter
}

impl<'hint> Display for ResolveHint<'hint> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ResolveHint::SimilarBrand { candidate, effective } => {
                write!(f, "did you mean `{:?}`? they {:?}.", candidate, effective.lock().unwrap())
            }

            ResolveHint::Parameter => {
                write!(f, "")
            }
        }
    }
}