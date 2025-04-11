use crate::axo_parser::{Expr, ExprKind};

pub enum SyntaxError {

}

pub struct Validator {
    pub errors: Vec<SyntaxError>,
}

impl Validator {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }
    pub fn validate(&mut self, program: Vec<Expr>) {

    }

}