use crate::{
    data::{Identity, Offset},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub identity: Identity,
    pub offset: Offset,
}

impl Position {
    #[inline]
    pub fn new(identity: Identity) -> Self {
        Self { identity, offset: 0 }
    }

    #[inline]
    pub fn default(identity: Identity) -> Self {
        Self { identity, offset: 0 }
    }

    #[inline]
    pub fn set_identity(&mut self, identity: Identity) {
        self.identity = identity;
    }

    #[inline]
    pub fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    #[inline]
    pub fn swap_identity(&self, identity: Identity) -> Self {
        Self { identity, ..*self }
    }

    #[inline]
    pub fn swap_offset(&self, offset: Offset) -> Self {
        Self { offset, ..*self }
    }

    #[inline]
    pub fn advance(&self, amount: Offset) -> Self {
        Self {
            offset: self.offset + amount,
            ..*self
        }
    }

    #[inline]
    pub fn add(&mut self, amount: Offset) {
        self.offset += amount;
    }
}
