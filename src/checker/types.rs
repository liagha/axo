use {
    crate::{
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

pub trait Type {

}

impl Type for Structure<Box<dyn Type>, Box<dyn Type>> {

}

impl Type for Method<Box<dyn Type>, Box<dyn Type>, Box<dyn Type>, Box<dyn Type>> {

}