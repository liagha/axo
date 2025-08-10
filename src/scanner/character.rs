use {
    super::{
        Scanner,
    },
    crate::{
        data::{
            Char,
            string::Str,
        },
        text::{is_alphabetic, is_numeric, is_whitespace, is_alphanumeric},
        tracker::{Span, Spanned, Position},
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Character<'character> {
    pub value: Char,
    pub span: Span<'character>,
}

impl<'a> FromIterator<Character<'a>> for Str<'a> {
    fn from_iter<T: IntoIterator<Item = Character<'a>>>(iter: T) -> Self {
        let s: String = iter.into_iter().collect();
        Str(s.leak().as_bytes())
    }
}

impl<'character> Character<'character> {
    pub fn new(value: char, span: Span<'character>) -> Self {
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

impl<'character> From<Character<'character>> for char {
    fn from(character: Character) -> Self {
        character.value
    }
}

impl<'character> PartialEq<char> for Character<'character> {
    fn eq(&self, other: &char) -> bool {
        self.value == *other
    }
}

impl<'character> PartialEq<Character<'character>> for char {
    fn eq(&self, other: &Character) -> bool {
        *self == other.value 
    }
}

impl<'character> FromIterator<Character<'character>> for String {
    fn from_iter<I: IntoIterator<Item = Character<'character>>>(iter: I) -> Self {
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