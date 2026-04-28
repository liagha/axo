use crate::{
    combinator::{Form, Former},
    data::{Identity, Offset, Scale, Str},
    internal::{Artifact, RecordKind, Session, SessionError},
    scanner::{Character, ErrorKind, ScanError, Token},
    tracker::{Peekable, Position},
};

pub struct Scanner<'scanner> {
    pub index: Offset,
    pub state: Position,
    pub input: Vec<Character>,
    pub output: Vec<Token<'scanner>>,
    pub errors: Vec<ScanError<'scanner>>,
}

impl<'scanner> Peekable<'scanner, Character> for Scanner<'scanner> {
    type State = Position;

    fn length(&self) -> Scale {
        self.input.len()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Character> {
        self.get(self.index + n)
    }

    fn peek_behind(&self, n: Offset) -> Option<&Character> {
        self.index.checked_sub(n).and_then(|idx| self.get(idx))
    }

    fn origin(&self) -> Self::State {
        Position::new(self.state.identity)
    }

    fn next(&self, index: &mut Offset, state: &mut Self::State) -> Option<Character> {
        let ch = self.get(*index)?;
        *state = Position {
            identity: ch.span.identity,
            offset: ch.span.end,
        };
        *index += 1;
        Some(*ch)
    }

    fn input(&self) -> &Vec<Character> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Character> {
        &mut self.input
    }

    fn state(&self) -> Self::State {
        self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    fn index(&self) -> Offset {
        self.index
    }

    fn index_mut(&mut self) -> &mut Offset {
        &mut self.index
    }
}

impl<'scanner> Scanner<'scanner> {
    pub fn new(state: Position, content: Str<'scanner>) -> Scanner<'scanner> {
        let mut scanner = Scanner {
            index: 0,
            state,
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        };

        let characters = Scanner::inspect(state, content.chars().collect::<Vec<_>>());
        scanner.set_input(characters);
        scanner
    }

    pub fn scan(&mut self) {
        let forms = {
            let mut former = Former::new(self);
            former.form(Self::formation()).flatten()
        };

        for form in forms {
            match form {
                Form::Output(output) => self.output.push(output),
                Form::Failure(failure) => self.errors.push(failure),
                _ => {}
            }
        }
    }

    pub fn execute(session: &mut Session<'scanner>, keys: &[Identity]) {
        for &key in keys {
            Self::process(session, key);
        }
    }

    fn process(session: &mut Session<'scanner>, key: Identity) {
        let (kind, location, content) = {
            let record = session.records.get(&key).unwrap();
            (record.kind.clone(), record.location, record.content.clone())
        };

        if kind != RecordKind::Source {
            return;
        }

        let content = if let Some(content) = content {
            Str::from(content)
        } else {
            match location.get_value() {
                Ok(content) => content,
                Err(error) => {
                    let kind = ErrorKind::Tracking(error.clone());
                    let scan_error = ScanError::new(kind, error.span);
                    session.errors.push(SessionError::Scan(scan_error));
                    return;
                }
            }
        };

        let position = Position::new(key);
        let mut scanner = Scanner::new(position, content);
        scanner.scan();

        scanner.output.shrink_to_fit();

        session.errors.extend(
            scanner
                .errors
                .iter()
                .map(|error| SessionError::Scan(error.clone())),
        );

        let record = session.records.get_mut(&key).unwrap();
        record.store(1, Artifact::Tokens(scanner.output));
    }
}
