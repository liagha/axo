use crate::parser::Element;

pub struct Checker<'checker> {
    pub input: Vec<Element<'checker>>,
}