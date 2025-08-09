use {
    super::{
        Character, Token, ScanError,
    },
    crate::{
        data::string::Str,
        formation::{
            form::Form,
            former::Former,
        },
        internal::{
            compiler::{
                Registry, Marked,
            },
        },
        tracker::{
            Peekable, Position, Location,
        },
    },
};

pub struct Scanner<'scanner> {
    pub index: usize,
    pub position: Position<'scanner>,
    pub input: Vec<Character<'scanner>>,
    pub output: Vec<Token<'scanner>>,
    pub errors: Vec<ScanError<'scanner>>,
}

impl<'scanner> Peekable<'scanner, Character<'scanner>> for Scanner<'scanner> {
    fn length(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, n: usize) -> Option<&Character<'scanner>> {
        self.get(self.index + n)
    }

    fn peek_behind(&self, n: usize) -> Option<&Character<'scanner>> {
        self.index.checked_sub(n).and_then(|idx| self.get(idx))
    }

    fn next(&self, index: &mut usize, position: &mut Position<'scanner>) -> Option<Character<'scanner>> {
        let ch = self.get(*index)?;

        if *ch == '\n' {
            position.line += 1;
            position.column = 1;
        } else {
            position.column += 1;
        }

        *index += 1;
        Some(*ch)
    }

    fn input(&self) -> &Vec<Character<'scanner>> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Character<'scanner>> {
        &mut self.input
    }

    fn position(&self) -> Position<'scanner> {
        self.position
    }

    fn position_mut(&mut self) -> &mut Position<'scanner> {
        &mut self.position
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl<'scanner> Scanner<'scanner> {
    pub fn new(location: Location<'scanner>) -> Scanner<'scanner> {
        let position = Position::new(location);

        Scanner {
            index: 0,
            position,
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn with_input(self, input: Str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let characters = Self::inspect(self.position, chars);

        Self {
            input: characters,
            ..self
        }
    }

    pub fn set_input(&mut self, input: String) {
        let chars: Vec<char> = input.chars().collect();
        let characters = Self::inspect(self.position, chars);

        self.input = characters;
    }

    pub fn scan(&mut self) {
        let classifier = Self::classifier();
        let mut former = Former::new(self);

        let forms = former.form(classifier.clone()).flatten();

        for form in forms {
            match form {
                Form::Output(output) => {
                    self.output.push(output);
                }

                Form::Failure(failure) => {
                    self.errors.push(failure);
                }

                _ => {}
            }
        }
    }
}

impl<'scanner> Marked<'scanner> for Scanner<'scanner> {
    fn registry(&self) -> &Registry<'scanner> {
        todo!()
    }

    fn registry_mut(&mut self) -> &mut Registry<'scanner> {
        todo!()
    }
}