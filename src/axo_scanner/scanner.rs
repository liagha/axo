use {
    super::{
        character::Character,
        ScanError, Token,
    },
    crate::{
        compiler::{
            Context, Marked,
        },
        axo_form::{
            form::{Form},
            former::Former,
        },
        axo_cursor::{
            Peekable, Position,
        },
    },
};

#[derive(Clone)]
pub struct Scanner {
    pub context: Context,
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

    fn restore(&mut self) {
        self.set_position(Position {
            line: 1,
            column: 1,
            location: self.position.location,
        })
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
    pub fn new(context: Context, input: String, file: &'static str) -> Scanner {
        let start = Position::new(file);
        let chars: Vec<char> = input.chars().collect();
        let characters = Self::inspect(start, chars);

        Scanner {
            context,
            input: characters,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> (Vec<Token>, Vec<ScanError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while self.peek().is_some() {
            let forms = self.form(Self::pattern()).expand();

            for form in forms {
                match form {
                    Form::Output(element) => {
                        tokens.push(element);
                    }

                    Form::Failure(error) => {
                        errors.push(error);
                    }

                    Form::Multiple(_) | Form::Blank | Form::Input(_) => {}
                }
            }
        }

        (tokens, errors)
    }
}

impl Marked for Scanner {
    fn context(&self) -> &Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}