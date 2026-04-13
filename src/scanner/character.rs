use crate::{
    data::{Char, Str},
    scanner::Scanner,
    text::{is_alphabetic, is_alphanumeric, is_numeric, is_whitespace},
    tracker::{Position, Span},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Character {
    pub value: Char,
    pub span: Span,
}

impl<'a> FromIterator<Character> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = Character>>(iter: T) -> Self {
        let string: Str = iter.into_iter().map(|c| c.value).collect();
        string
    }
}

impl<'character> Character {
    pub fn new(value: char, span: Span) -> Self {
        Self { value, span }
    }

    pub fn is_digit(&self, radix: u32) -> bool {
        self.value.is_digit(radix)
    }

    pub fn is_numeric(&self) -> bool {
        is_numeric(self.value)
    }

    pub fn is_alphabetic(&self) -> bool {
        is_alphabetic(self.value)
    }

    pub fn is_alphanumeric(&self) -> bool {
        is_alphanumeric(self.value)
    }

    pub fn is_whitespace(&self) -> bool {
        is_whitespace(self.value)
    }
}

impl<'character> From<Character> for char {
    fn from(character: Character) -> Self {
        character.value
    }
}

impl<'character> PartialEq<char> for Character {
    fn eq(&self, other: &char) -> bool {
        self.value == *other
    }
}

impl<'character> PartialEq<Character> for char {
    fn eq(&self, other: &Character) -> bool {
        *self == other.value
    }
}

impl<'character> FromIterator<Character> for String {
    fn from_iter<I: IntoIterator<Item = Character>>(iter: I) -> Self {
        iter.into_iter().map(|character| character.value).collect()
    }
}

impl<'scanner> Scanner<'scanner> {
    pub fn inspect(start: Position, input: Vec<char>) -> Vec<Character> {
        let mut state = start;
        let mut characters = Vec::new();

        for value in input {
            let start = state;
            state.add(value.len_utf8() as u32);
            characters.push(Character {
                value,
                span: Span::new(start, state),
            });
        }

        characters
    }
}
