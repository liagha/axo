use crate::analyzer::analysis::Analysis;
use crate::parser::Element;

pub struct Analyzer<'analyzer> {
    pub input: Vec<Element<'analyzer>>,
    pub output: Vec<Analysis<'analyzer>>,
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn new() -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
        } 
    }
    
    pub fn with_input(input: Vec<Element<'analyzer>>) -> Self {
        Self {
            input,
            output: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self) {
        
    }
}