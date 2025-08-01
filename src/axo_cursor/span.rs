use {
    super::{Position, Spanned, Location},
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

impl Span {
    #[inline]
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn default(location: Location) -> Self {
        Self { start: Position::default(location), end: Position::default(location) } 
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
    pub fn overlaps(&self, other: &Self) -> bool {
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
    pub fn from_slice<T: Spanned>(items: &[T]) -> Self {
        match items.len() {
            0 => panic!("can't create a span from an empty Slice."),
            1 => items[0].span(),
            _ => {
                let start = items.first().unwrap().span();
                let end = items.last().unwrap().span();
                start.merge(&end)
            }
        }
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        if self.start.location != other.start.location {
            panic!("cannot mix spans from `{}` with `{}`.", self.start.location, other.start.location);
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