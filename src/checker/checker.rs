use crate::analyzer::{AnalyzeError};
use crate::checker::{CheckError};
use crate::parser::Element;

pub struct Checker<'analyzer> {
    pub input: &'analyzer mut Vec<Element<'analyzer>>,
    pub errors: Vec<AnalyzeError<'analyzer>>,
}

pub trait Checkable<'checkable> {
    fn check(&mut self) -> Vec<CheckError<'checkable>>;
}

impl<'resolver> Checker<'resolver> {
    fn check(&mut self) {
        for element in self.input.iter_mut() {
            element.check();
        }
    }
}
