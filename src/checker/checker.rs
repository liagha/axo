use crate::parser::{Element, ElementKind};
use crate::scanner::TokenKind;

pub struct Checker<'checker> {
    pub input: Vec<Element<'checker>>,
}

impl<'checker> Checker<'checker> {
    pub fn new(input: Vec<Element<'checker>>) -> Self {
        Self { input }
    }

    pub fn check(&self, element: Element<'checker>) {

    }
}