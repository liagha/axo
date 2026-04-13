use crate::data::{Offset, Scale};

pub struct Peeker<State, Input> {
    pub index: Offset,
    pub state: State,
    pub input: Input,
}

pub trait Peekable<'peekable, Item: PartialEq + 'peekable> {
    type State: Copy + Default + Send + Sync;

    fn length(&self) -> Scale;

    fn remaining(&self) -> Scale {
        self.length() - self.index() as Scale
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Item>;
    fn peek_behind(&self, n: Offset) -> Option<&Item>;

    fn origin(&self) -> Self::State;

    fn reset(&mut self) {
        self.set_index(0);
        self.set_state(self.origin());
    }

    fn advance(&mut self) -> Option<Item> {
        let mut index = self.index();
        let mut state = self.state();
        let result = self.next(&mut index, &mut state);

        if result.is_some() {
            self.set_index(index);
            self.set_state(state);
        }

        result
    }

    fn next(&self, index: &mut Offset, state: &mut Self::State) -> Option<Item>;

    fn get(&self, index: Offset) -> Option<&Item> {
        self.input().get(index as usize)
    }

    fn get_mut(&mut self, index: Offset) -> Option<&mut Item> {
        self.input_mut().get_mut(index as usize)
    }

    fn insert(&mut self, index: Offset, item: Item) {
        self.input_mut().insert(index as usize, item);
    }

    fn remove(&mut self, index: Offset) -> Option<Item> {
        Some(self.input_mut().remove(index as usize))
    }

    fn input(&self) -> &Vec<Item>;
    fn input_mut(&mut self) -> &mut Vec<Item>;
    fn state(&self) -> Self::State;
    fn state_mut(&mut self) -> &mut Self::State;
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

    fn set_state(&mut self, state: Self::State) {
        *self.state_mut() = state;
    }

    fn set_input(&mut self, input: Vec<Item>) {
        *self.input_mut() = input;
    }

    fn skip(&mut self, count: Offset) {
        for _ in 0..count {
            self.advance();
        }
    }
}
