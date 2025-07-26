use {
    crate::{
        axo_cursor::{Span, Position},
        axo_scanner::{
            Scanner,
        },
        is_alphabetic, is_numeric, is_whitespace, is_alphanumeric
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
        is_whitespace(self.value)
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

impl<'scanner> Scanner<'scanner> {
    pub fn inspect(start: Position, input: Vec<char>) -> Vec<Character> {
        let mut position = start;
        let mut characters = Vec::new();

        for char in input {
            let character = match char {
                '\n' => {
                    let start = position;
                    position.add_line(1);
                    position.set_column(1);

                    Character {
                        value: char,
                        span: Span {
                            start,
                            end: position,
                        }
                    }
                }
                char => {
                    let start = position;
                    position.add_column(1);

                    Character {
                        value: char,
                        span: Span {
                            start,
                            end: position,
                        }
                    }
                }
            };

            characters.push(character);
        }

        characters
    }
}