use crate::axo_validator::ValidateError;

pub struct Validator {
    pub errors: Vec<ValidateError>,
}