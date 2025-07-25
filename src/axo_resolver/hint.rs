use {
    matchete::{
        Resembler,
    },
    crate::{
        axo_scanner::Token,
        thread::{
            Arc, Mutex,
        },
        format::{Display, Formatter},
    }
};

#[derive(Clone, Debug)]
pub enum ResolveHint {
    SimilarBrand { candidate: Token, effective: Arc<Mutex<dyn Resembler<String, String, ()>>> },
    Parameter
}

impl Display for ResolveHint {
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