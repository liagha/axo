use {
    crate::{
        axo_parser::{
            Element, Symbol, 
        },

        axo_resolver::{
            Resolver,
        }
    }
};

impl Resolver {
    pub fn validate(&mut self, _element: &Element, _item: &Symbol) {
    }
}