use {
    crate::{
        axo_cursor::Span,
        is_alphabetic, is_numeric, is_white_space, is_alphanumeric
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Character {
    pub value: char,
    pub span: Span,
}

impl Character {
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
        is_white_space(self.value)
    }
}

impl From<Character> for char {
    fn from(character: Character) -> Self {
        character.value
    }
}

impl PartialEq<char> for Character {
    fn eq(&self, other: &char) -> bool {
        self.value == *other
    }
}

impl PartialEq<Character> for char {
    fn eq(&self, other: &Character) -> bool {
        *self == other.value 
    }
}

impl FromIterator<Character> for String {
    fn from_iter<I: IntoIterator<Item = Character>>(iter: I) -> Self {
        iter.into_iter().map(|character| character.value).collect()
    }
}