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

impl<M: Display> Hint<M> {
    pub fn new(message: M) -> Self {
        Hint { message, action: Vec::new() }
    }
}