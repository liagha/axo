#![allow(unused_imports)]

use {
    crate::{
        axo_schema::{
            Procedural,
            Group, Sequence,
            Collection, Series,
            Bundle, Block,
            Binary, Unary,
            Index, Invoke, Construct,
            Conditional, Repeat, Iterate,
            Label, Access, Assign,
            Structure, Enumeration,
            Method,
        }
    }
};

pub trait Type {

}

impl Type for Structure<Box<dyn Type>, Box<dyn Type>> {

}

impl Type for Method<Box<dyn Type>, Box<dyn Type>, Box<dyn Type>, Box<dyn Type>> {

}