use {
    crate::{
        parser::{
            Element,
        },
        checker::{
            CheckError,
            types::Type,
        },
    }
};

pub struct Checker<'checker> {
    input: Vec<Element<'checker>>,
    output: Vec<Type<'checker>>,
    errors: Vec<CheckError<'checker>>,
}