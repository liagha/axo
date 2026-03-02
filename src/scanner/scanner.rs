use {
    super::{Character, ScanError, Token},
    crate::{
        data::{Offset, Scale},
        internal::timer::Duration,
        formation::{form::Form, former::Former},
        tracker::{Location, Peekable, Position},
        scanner::error::ErrorKind,
    },
};

pub struct Scanner<'scanner> {
    pub index: Offset,
    pub position: Position<'scanner>,
    pub input: Vec<Character<'scanner>>,
    pub output: Vec<Token<'scanner>>,
    pub errors: Vec<ScanError<'scanner>>,
}

impl<'scanner> Peekable<'scanner, Character<'scanner>> for Scanner<'scanner> {
    fn length(&self) -> Scale {
        self.input.len()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Character<'scanner>> {
        self.get(self.index + n)
    }

    fn peek_behind(&self, n: Offset) -> Option<&Character<'scanner>> {
        self.index.checked_sub(n).and_then(|idx| self.get(idx))
    }

    fn next(
        &self,
        index: &mut Offset,
        position: &mut Position<'scanner>,
    ) -> Option<Character<'scanner>> {
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

    fn index(&self) -> Offset {
        self.index
    }

    fn index_mut(&mut self) -> &mut Offset {
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

    pub fn prepare(&mut self) {
        let location = self.position.location;

        match location.get_value() {
            Ok(content) => {
                let characters =
                    Scanner::inspect(Position::new(location), content.chars().collect::<Vec<_>>());
                self.set_input(characters);
            }

            Err(error) => {
                let kind = ErrorKind::Tracking(error.clone());
                let error = ScanError::new(kind, error.span);

                self.errors.push(error);
            }
        }

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
