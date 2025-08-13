use {
    crate::{
        scanner::Token,
        schema::{
            Binary, Unary,
            Bundle, Block,
            Collection, Series,
            Conditional, Repeat, Iterate,
            Group, Sequence,
            Index, Invoke, Construct,
            Label, Access, Assign,
            Method,
            Procedural,
            Structure, Enumeration,
        }
    }
};
use crate::data::string::Str;
use crate::scanner::TokenKind;
use crate::tracker::{Location, Position, Span};

pub struct Typed<'ty, Value> {
    pub value: Value,
    pub ty: Box<dyn Type<'ty>>,
    pub phantom: &'ty (),
}

pub trait Type<'ty> {

}

impl<'ty> Type<'ty> for Structure<Token<'ty>, Box<dyn Type<'ty>>> {

}

impl<'ty> Type<'ty> for Method<Token<'ty>, Box<dyn Type<'ty>>, Box<dyn Type<'ty>>, Box<dyn Type<'ty>>> {

}