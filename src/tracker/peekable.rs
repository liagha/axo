use {
    crate::{
        data::{
            Offset, Scale,
            string::Str,
        },
        tracker::{Location, Position},
    },
};

pub trait Peekable<'peekable, Item: PartialEq + 'peekable> {
    fn length(&self) -> Scale;
    fn remaining(&self) -> Scale {
        self.length() - self.index()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Item>;
    fn peek_behind(&self, n: Offset) -> Option<&Item>;

    fn reset(&mut self) {
        self.set_index(0);

        self.set_position(
            Position::new(self.position().location)
        );
    }

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

    fn next(&self, index: &mut Offset, position: &mut Position<'peekable>) -> Option<Item>;

    fn get(&self, index: Offset) -> Option<&Item> {
        self.input().get(index)
    }

    fn get_mut(&mut self, index: Offset) -> Option<&mut Item> {
        self.input_mut().get_mut(index)
    }

    fn insert(&mut self, index: Offset, item: Item) {
        self.input_mut().insert(index, item);
    }

    fn remove(&mut self, index: Offset) -> Option<Item> {
        Some(self.input_mut().remove(index))
    }

    fn input(&self) -> &Vec<Item>;
    fn input_mut(&mut self) -> &mut Vec<Item>;

    fn position(&self) -> Position<'peekable>;
    fn position_mut(&mut self) -> &mut Position<'peekable>;
    fn index(&self) -> Offset;
    fn index_mut(&mut self) -> &mut Offset;

    fn peek(&self) -> Option<&Item> {
        self.peek_ahead(0)
    }

    fn peek_previous(&self) -> Option<&Item> {
        self.peek_behind(1)
    }

    fn set_index(&mut self, index: Offset) {
        *self.index_mut() = index;
    }

    fn set_position(&mut self, position: Position<'peekable>) {
        *self.position_mut() = position;
    }

    fn set_line(&mut self, line: Offset) {
        self.position_mut().line = line;
    }

    fn set_column(&mut self, line: Offset) {
        self.position_mut().column = line;
    }

    fn set_path(&mut self, path: Str<'peekable>) {
        self.position_mut().location = Location::File(path);
    }

    fn set_location(&mut self, location: Location<'peekable>) {
        self.position_mut().location = location;
    }

    fn skip(&mut self, count: Offset) {
        for _ in 0..count {
            self.advance();
        }
    }
}