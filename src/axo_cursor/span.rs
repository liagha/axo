#![allow(dead_code)]

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn extend(&mut self, size: (usize, usize)) {
        self.end.line += size.0;
        self.end.column += size.1;
    }

    #[inline]
    pub fn extend_to(&mut self, other: Box<dyn Spanned>) {
        self.end = other.span().end;
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
        if self.start.path != pos.path {
            return false;
        }

        (self.start.cmp(pos) != Ordering::Greater) && (self.end.cmp(pos) != Ordering::Less)
    }

    pub fn contains_span(&self, other: &Span) -> bool {
        if self.start.path != other.start.path || self.start.path != other.end.path {
            return false;
        }

        self.contains(&other.start) && self.contains(&other.end)
    }

    #[inline]
    pub fn overlaps(&self, other: &Span) -> bool {
        if self.start.path != other.start.path {
            return false;
        }

        self.contains(&other.start)
            || self.contains(&other.end)
            || other.contains(&self.start)
            || other.contains(&self.end)
    }

    pub fn merge(&self, other: &Span) -> Option<Span> {
        if self.start.path != other.start.path {
            return None;
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

        Some(Span::new(start, end))
    }

    #[inline]
    pub fn mix(&self, other: &Span) -> Span {
        if self.start.path != other.start.path {
            panic!("cannot mix spans from `{}` with `{}`", self.start.path, other.start.path);
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

    pub fn to_range_string(&self) -> String {
        format!(
            "{}:{}-{}:{}",
            self.start.line, self.start.column, self.end.line, self.end.column
        )
    }
}