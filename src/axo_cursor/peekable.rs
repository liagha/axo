use crate::{
    axo_cursor::{Position, Span},
    Path,
};

pub trait Peekable<Item: PartialEq> {
    fn len(&self) -> usize;

    fn peek_ahead(&self, n: usize) -> Option<&Item>;
    fn peek_behind(&self, n: usize) -> Option<&Item>;

    fn restore(&mut self);
    fn next(&mut self) -> Option<Item>;

    fn position(&self) -> Position;
    fn position_mut(&mut self) -> &mut Position;
    fn index(&self) -> usize;
    fn index_mut(&mut self) -> &mut usize;

    fn peek(&self) -> Option<&Item> {
        self.peek_ahead(0)
    }

    fn peek_previous(&self) -> Option<&Item> {
        self.peek_behind(1)
    }

    fn peek_matches(&self, expected: &Item) -> bool
    where
        Item: PartialEq,
    {
        if let Some(item) = self.peek() {
            item == expected
        } else {
            false
        }
    }

    fn seek(&mut self, position: Position) {
        self.set_position(position);
    }

    fn set_position(&mut self, position: Position) {
        *self.position_mut() = position;
    }

    fn set_line(&mut self, line: usize) {
        self.position_mut().line = line;
    }

    fn set_column(&mut self, line: usize) {
        self.position_mut().column = line;
    }

    fn set_path(&mut self, path: Path) {
        self.position_mut().path = path;
    }

    fn skip(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }
}
