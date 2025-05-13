use crate::{Lexer, Path};
use crate::axo_span::position::Position;
use crate::axo_span::span::Span;

pub trait Peekable<Item : PartialEq> {
    fn peek(&self) -> Option<&Item> {
        self.peek_ahead(0)
    }
    fn peek_previous(&self) -> Option<&Item> {
        self.peek_behind(1)
    }

    fn peek_ahead(&self, n: usize) -> Option<&Item>;
    fn peek_behind(&self, n: usize) -> Option<&Item>;

    fn next(&mut self) -> Option<Item>;
    
    fn match_item(&mut self, item: &Item) -> bool {
        if let Some(peek) = self.peek() {
            if peek == item {
                return false;
            }
            
            self.next();
            
            true
        } else { 
            false
        }
    }

    fn position(&self) -> Position;

    fn set_index(&mut self, index: usize);

    fn set_line(&mut self, line: usize);

    fn set_column(&mut self, column: usize);

    fn set_position(&mut self, position: Position);

    fn save_position(&self) -> Position {
        self.position()
    }

    fn restore_position(&mut self, position: Position) {
        self.set_position(position);
    }

    fn span_from(&self, start: Position) -> Span {
        Span::new(start, self.position())
    }

    fn peek_matches(&self, expected: &Item) -> bool
    where Item: PartialEq {
        if let Some(item) = self.peek() {
            item == expected
        } else {
            false
        }
    }

    fn has_more(&self) -> bool {
        self.peek().is_some()
    }

    fn skip(&mut self, count: usize) -> usize {
        let mut skipped = 0;
        for _ in 0..count {
            if self.next().is_some() {
                skipped += 1;
            } else {
                break;
            }
        }
        skipped
    }
}