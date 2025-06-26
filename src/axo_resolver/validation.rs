use {
    crate::{
        axo_parser::{
            Element, Item, 
        },

        axo_resolver::{
            Resolver,
        }
    }
};

impl Resolver {
    pub fn validate(&mut self, _element: &Element, _item: &Item) {
    }
}