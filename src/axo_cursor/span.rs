#![allow(dead_code)]

use {
    super::{
        Spanned,
        Position,
    },
    
    crate::{
        Path,
        
        format,
        file,
        
        compare::{
            Ordering,
        }
    },
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Default for Span {
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
        Span {
            start,
            end,
        }
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
    pub fn from_coords(file: Path, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        let start = Position::at(file.clone(), start_line, start_col);
        let end = Position::at(file, end_line, end_col);
        Self::new(start, end)
    }

    #[inline]
    pub fn contains(&self, pos: &Position) -> bool {
        if self.start.path != pos.path {
            return false;
        }

        (self.start.cmp(pos) != Ordering::Greater) &&
            (self.end.cmp(pos) != Ordering::Less)
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

        self.contains(&other.start) ||
            self.contains(&other.end) ||
            other.contains(&self.start) ||
            other.contains(&self.end)
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

    pub fn mix(&self, other: &Span) -> Span {
        if self.start.path != other.start.path {
            panic!("Cannot mix spans from different files");
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
        format!("{}:{}-{}:{}",
                self.start.line, self.start.column,
                self.end.line, self.end.column)
    }

    pub fn line_spans(&self) -> Vec<Span> {
        if self.start.path != self.end.path {
            return Vec::new();
        }

        let mut result = Vec::new();

        if self.start.line == self.end.line {
            result.push(self.clone());
            return result;
        }

        if let Some(line_content) = self.start.get_line_content() {
            let end_of_line = Position {
                line: self.start.line,
                column: line_content.len() + 1,
                path: self.start.path.clone(),
            };
            result.push(Span::new(self.start.clone(), end_of_line));
        }

        for line_num in (self.start.line + 1)..self.end.line {
            let start_pos = Position {
                line: line_num,
                column: 1,
                path: self.start.path.clone(),
            };
            let mut end_pos = start_pos.clone();

            if let Some(line_content) = start_pos.get_line_content() {
                end_pos.column = line_content.len() + 1;
            }

            let start_pos = start_pos.correct();
            let end_pos = end_pos.correct();

            result.push(Span::new(start_pos, end_pos));
        }

        let start_of_last_line = Position {
            line: self.end.line,
            column: 1,
            path: self.start.path.clone(),
        };
        let start_of_last_line = start_of_last_line.correct();

        result.push(Span::new(start_of_last_line, self.end.clone()));

        result
    }
}


impl Spanned for Span {
    fn span(&self) -> Span {
        self.clone()
    }
}