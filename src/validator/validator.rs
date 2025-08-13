use crate::parser::Element;
use crate::validator::ValidateError;

pub struct Validator<'validator> {
    pub input: Vec<Element<'validator>>,
    pub errors: Vec<ValidateError<'validator>>,
}

impl<'validator> Validator<'validator> {
    pub fn validate(&mut self, element: &Element<'validator>) {

    }
}