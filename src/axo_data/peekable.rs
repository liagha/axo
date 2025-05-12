use crate::{Lexer, Path};
use crate::axo_span::position::Position;

pub trait Peekable<Item> {
    fn peek(&self) -> Option<&Item>;

    fn peek_ahead(&self, n: usize) -> Option<&Item>;

    fn next(&mut self) -> Option<Item>;
    
    fn position(&self) -> Position;
    
    fn set_index(&mut self, index: usize);
    
    fn set_line(&mut self, line: usize);
    
    fn set_column(&mut self, column: usize);
    fn set_position(&mut self, position: Position);
}