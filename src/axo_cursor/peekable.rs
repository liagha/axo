use crate::{
    axo_cursor::{Position, Span},
};

pub trait Peekable<Item: PartialEq> {
    fn len(&self) -> usize;

    fn peek_ahead(&self, n: usize) -> Option<&Item>;
    fn peek_behind(&self, n: usize) -> Option<&Item>;

    fn restore(&mut self);

    /// Consuming input in a peekable
    fn advance(&mut self) -> Option<Item> {
        let mut position = self.position();
        let mut index = self.index();

        let result = self.next(&mut index, &mut position);

        if result.is_some() {
            self.set_index(index);

            self.set_position(position);
        }

        result
    }

    /// Advancing but on a position and index.
    fn next(&self, index: &mut usize, position: &mut Position) -> Option<Item>;

    fn forward(&self, index: &mut usize, position: &mut Position, amount: usize) {
        for _ in 0..amount {
            self.next(index, position);
        }
    }

    fn get(&self, index: usize) -> Option<&Item> {
        self.input().get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.input_mut().get_mut(index)
    }

    fn input(&self) -> &[Item];
    fn input_mut(&mut self) -> &mut [Item];

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

    fn set_index(&mut self, index: usize) {
        *self.index_mut() = index;
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

    fn set_path(&mut self, path: &'static str) {
        self.position_mut().path = path;
    }

    fn skip(&mut self, count: usize) {
        for _ in 0..count {
            self.advance();
        }
    }
}
