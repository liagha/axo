use crate::checker::CheckError;
use crate::parser::Element;

pub struct Checker<'check, 'source> {
    pub input: &'check mut Vec<Element<'source>>,
    pub errors: Vec<CheckError<'source>>,
}

pub trait Checkable<'element> {
    fn check(&mut self, errors: &mut Vec<CheckError<'element>>);
}

impl<'check, 'source> Checker<'check, 'source> {
    pub fn new(input: &'check mut Vec<Element<'source>>) -> Self {
        Self { input, errors: vec![] }
    }

    pub fn check(&mut self) {
        for element in self.input.iter_mut() {
            element.check(&mut self.errors);
        }
    }
}