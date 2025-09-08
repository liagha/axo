use {
    crate::{
        data::{Offset, Str, Boolean},
        internal::{
            hash::Hash,
            operation::Ordering,
        }
    },
    super::{Location, Position, Spanned},
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span<'span> {
    pub start: Position<'span>,
    pub end: Position<'span>,
}

impl<'span> Span<'span> {
    #[inline]
    pub fn new(start: Position<'span>, end: Position<'span>) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn void() -> Self {
        Span::point(Position::new(Location::Void))
    }
    
    #[inline]
    pub fn default(location: Location<'span>) -> Self {
        Self { start: Position::default(location), end: Position::default(location) } 
    }

    #[inline]
    pub fn file(path: Str<'span>) -> Self {
        let location = Location::File(path);
        let content = location.get_value();

        let lines: Vec<Str> = content.lines();
        let total = lines.len().max(1) as Offset;
        let end = lines.last()
            .map(|line| line.chars().count() + 1)
            .unwrap_or(1) as Offset;

        let start = Position::new(location);
        let end = Position {
            line: total,
            column: end,
            location,
        };

        Self::new(start, end)
    }
    
    #[inline]
    pub fn point(pos: Position<'span>) -> Self {
        Self {
            start: pos.clone(),
            end: pos,
        }
    }

    #[inline]
    pub fn contains(&self, pos: &Position) -> Boolean {
        if self.start.location != pos.location {
            return false;
        }

        (self.start.cmp(pos) != Ordering::Greater) && (self.end.cmp(pos) != Ordering::Less)
    }

    #[inline]
    pub fn overlaps(&self, other: &Self) -> Boolean {
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
    pub fn from_slice<T: Spanned<'span>>(items: &[T]) -> Self {
        match items.len() {
            0 => panic!("can't create a span from an empty Slice."),
            1 => items[0].borrow_span(),
            _ => {
                let start = items.first().unwrap().borrow_span();
                let end = items.last().unwrap().borrow_span();
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