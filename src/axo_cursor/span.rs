use {
    super::{Position, Spanned},
    crate::{
        hash::Hash,
        file, format,
        compare::Ordering,
        format::{
            Debug, Display,
        },
        axo_form::{
            form::Form,
        },
        axo_scanner::{
            Character, Token,
        },
        axo_parser::{
            Element,
        }
    },
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Default for Span {
    #[inline]
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}

impl Span {
    #[inline]
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    #[inline]
    pub fn point(pos: Position) -> Self {
        Self {
            start: pos.clone(),
            end: pos,
        }
    }

    #[inline]
    pub fn contains(&self, pos: &Position) -> bool {
        if self.start.location != pos.location {
            return false;
        }

        (self.start.cmp(pos) != Ordering::Greater) && (self.end.cmp(pos) != Ordering::Less)
    }

    #[inline]
    pub fn overlaps(&self, other: &Span) -> bool {
        if self.start.location != other.start.location {
            return false;
        }

        self.contains(&other.start)
            || self.contains(&other.end)
            || other.contains(&self.start)
            || other.contains(&self.end)
    }

    #[inline]
    #[track_caller]
    pub fn merge(&self, other: &Span) -> Span {
        if self.start.location != other.start.location {
            panic!("cannot mix spans from `{}` with `{}`", self.start.location, other.start.location);
        }

        let start = if self.start.cmp(&other.start) == Ordering::Less {
            self.start.clone()
        } else {
            other.start.clone()
        };

        let end = if self.end.cmp(&other.end) == Ordering::Greater {
            self.end.clone()
        } else {
            other.end.clone()
        };

        Span::new(start, end)
    }
}