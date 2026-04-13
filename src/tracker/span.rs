use orbyte::Orbyte;
use crate::{
    data::{Boolean, Identity, Offset},
    tracker::{Position, Spanned},
};

#[derive(Clone, Copy, Eq, Hash, Orbyte, PartialEq)]
pub struct Span {
    pub identity: Identity,
    pub start: Offset,
    pub end: Offset,
}

impl Span {
    #[inline]
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            identity: start.identity,
            start: start.offset,
            end: end.offset,
        }
    }

    #[inline]
    pub fn range(identity: Identity, start: Offset, end: Offset) -> Self {
        Self { identity, start, end }
    }

    #[inline]
    pub fn void() -> Self {
        Self::range(0, 0, 0)
    }

    #[inline]
    pub fn default(identity: Identity) -> Self {
        Self::range(identity, 0, 0)
    }

    #[inline]
    pub fn point(pos: Position) -> Self {
        Self::range(pos.identity, pos.offset, pos.offset)
    }

    #[inline]
    pub fn contains(&self, pos: &Position) -> Boolean {
        self.identity == pos.identity && self.start <= pos.offset && pos.offset <= self.end
    }

    #[inline]
    pub fn overlaps(&self, other: &Self) -> Boolean {
        self.identity == other.identity && self.start <= other.end && other.start <= self.end
    }

    #[inline]
    #[track_caller]
    pub fn from_slice<'a, T: Spanned<'a>>(items: &[T]) -> Self {
        match items.len() {
            0 => Span::void(),
            1 => items[0].span(),
            _ => items.first().unwrap().span().merge(&items.last().unwrap().span()),
        }
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        if self.identity != other.identity {
            return *self;
        }

        Self {
            identity: self.identity,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
