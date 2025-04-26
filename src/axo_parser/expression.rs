use {
    crate::{
        axo_parser::{
            Parser,
            Primary, Element,
        },
    },
};

pub trait Expression {
    fn parse_basic(&mut self) -> Element;
    fn parse_complex(&mut self) -> Element;
}

impl Expression for Parser {
    fn parse_basic(&mut self) -> Element {
        self.parse_binary(Parser::parse_primary, 0)
    }

    fn parse_complex(&mut self) -> Element {
        self.parse_binary(Parser::parse_leaf, 0)
    }
}