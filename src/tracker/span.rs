use orbyte::Orbyte;
use crate::{
    data::{Boolean, Offset, Str},
    internal::{hash::Hash, operation::Ordering},
    tracker::{Location, Position, Spanned, TrackError},
};

#[derive(Clone, Copy, Eq, Hash, Orbyte, PartialEq)]
pub struct Span<'span> {
    pub location: Location<'span>,
    pub start_line: Offset,
    pub start_column: Offset,
    pub end_line: Offset,
    pub end_column: Offset,
}

impl<'span> Span<'span> {
    #[inline]
    pub fn new(start: Position<'span>, end: Position<'span>) -> Self {
        Self {
            location: start.location,
            start_line: start.line,
            start_column: start.column,
            end_line: end.line,
            end_column: end.column,
        }
    }

    #[inline]
    pub fn void() -> Self {
        Span::point(Position::new(Location::Void))
    }

    #[inline]
    pub fn default(location: Location<'span>) -> Self {
        Self {
            location,
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        }
    }

    #[inline]
    pub fn file(path: Str<'span>) -> Result<Self, TrackError<'span>> {
        let location = Location::Entry(path);

        match location.get_value() {
            Ok(content) => {
                let lines: Vec<Str> = content.lines();
                let total = lines.len().max(1) as Offset;
                let end = lines
                    .last()
                    .map(|line| line.chars().count() + 1)
                    .unwrap_or(1) as Offset;

                Ok(Self {
                    location,
                    start_line: 1,
                    start_column: 1,
                    end_line: total,
                    end_column: end,
                })
            }

            Err(error) => Err(error),
        }
    }

    #[inline]
    pub fn point(pos: Position<'span>) -> Self {
        Self {
            location: pos.location,
            start_line: pos.line,
            start_column: pos.column,
            end_line: pos.line,
            end_column: pos.column,
        }
    }

    #[inline]
    pub fn contains(&self, pos: &Position) -> Boolean {
        if self.location != pos.location {
            return false;
        }

        let start = Position { line: self.start_line, column: self.start_column, location: self.location };
        let end = Position { line: self.end_line, column: self.end_column, location: self.location };

        (start.cmp(pos) != Ordering::Greater) && (end.cmp(pos) != Ordering::Less)
    }

    #[inline]
    pub fn overlaps(&self, other: &Self) -> Boolean {
        if self.location != other.location {
            return false;
        }

        let other_start = Position { line: other.start_line, column: other.start_column, location: other.location };
        let other_end = Position { line: other.end_line, column: other.end_column, location: other.location };

        self.contains(&other_start)
            || self.contains(&other_end)
            || other.contains(&Position { line: self.start_line, column: self.start_column, location: self.location })
            || other.contains(&Position { line: self.end_line, column: self.end_column, location: self.location })
    }

    #[inline]
    #[track_caller]
    pub fn from_slice<T: Spanned<'span>>(items: &[T]) -> Self {
        match items.len() {
            0 => Span::void(),
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
        if self.location != other.location {
            return *self;
        }

        let mut start = Position::new(self.location);
        start.line = self.start_line;
        start.column = self.start_column;

        let mut other_start = Position::new(other.location);
        other_start.line = other.start_line;
        other_start.column = other.start_column;

        let final_start = if start.cmp(&other_start) == Ordering::Less {
            start
        } else {
            other_start
        };

        let mut end = Position::new(self.location);
        end.line = self.end_line;
        end.column = self.end_column;

        let mut other_end = Position::new(other.location);
        other_end.line = other.end_line;
        other_end.column = other.end_column;

        let final_end = if end.cmp(&other_end) == Ordering::Greater {
            end
        } else {
            other_end
        };

        Span::new(final_start, final_end)
    }
}
