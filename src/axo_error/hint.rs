use {
    crate::{
        format::Display,
    },
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Hint<M: Display> {
    pub message: M,
    pub action: Vec<u8>,
}