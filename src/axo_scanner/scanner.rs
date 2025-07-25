use {
    super::{
        character::Character,
        ScanError, Token,
    },
    crate::{
        axo_cursor::{
            Peekable, Position,
            Location,
        },
        axo_internal::{
            compiler::{
                Registry, Marked,
            },
        },
        axo_form::{
            form::Form,
            former::Former,
        },
    },
};

#[derive(Clone)]
pub struct Scanner {
    pub registry: Registry,
    pub index: usize,
    pub position: Position,
    pub input: Vec<Character>,
    pub output: Vec<Token>,
    pub errors: Vec<ScanError>,
}

impl Peekable<Character> for Scanner {
    fn len(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, n: usize) -> Option<&Character> {
        self.get(self.index + n)
    }

    fn peek_behind(&self, n: usize) -> Option<&Character> {
        self.index.checked_sub(n).and_then(|idx| self.get(idx))
    }

    fn next(&self, index: &mut usize, position: &mut Position) -> Option<Character> {
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

    fn input(&self) -> &Vec<Character> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Character> {
        &mut self.input
    }

    fn position(&self) -> Position {
        self.position
    }

    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl Scanner {
    pub fn new(registry: Registry, input: String, location: Location) -> Scanner {
        let position = Position::new(location);
        let chars: Vec<char> = input.chars().collect();
        let characters = Self::inspect(position, chars);

        Scanner {
            registry,
            input: characters,
            index: 0,
            position,
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn scan(&mut self) {
        while self.peek().is_some() {
            let forms = self.form(Self::pattern()).flatten();

            for form in forms {
                match form {
                    Form::Output(output) => {
                        self.output.push(output);
                    }

                    Form::Failure(failure) => {
                        self.errors.push(failure);
                    }

                    Form::Multiple(_) | Form::Blank | Form::Input(_) => {}
                }
            }
        }
    }
}

impl Marked for Scanner {
    fn registry(&self) -> &Registry {
        &self.registry
    }

    fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }
}