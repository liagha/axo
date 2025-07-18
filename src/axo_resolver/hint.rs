use std::sync::{Arc, Mutex};
use matchete::Resembler;
use crate::axo_scanner::Token;
use core::fmt::Display;
use std::fmt::Formatter;

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