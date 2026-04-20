use crate::{
    combinator::{Form, Formation, Former},
    data::{Identity, Offset, Scale},
    internal::{Artifact, RecordKind, Session, SessionError},
    parser::{Element, ErrorKind, ParseError},
    scanner::{PunctuationKind, Token, TokenKind},
    tracker::{Peekable, Position},
};

pub struct Parser<'a> {
    pub index: Offset,
    pub state: Position,
    pub input: Vec<Token<'a>>,
    pub output: Vec<Element<'a>>,
    pub errors: Vec<ParseError<'a>>,
}

impl<'a> Peekable<'a, Token<'a>> for Parser<'a> {
    type State = Position;

    fn length(&self) -> Scale {
        self.input.len()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Token<'a>> {
        self.get(self.index + n)
    }

    fn peek_behind(&self, n: Offset) -> Option<&Token<'a>> {
        self.index.checked_sub(n).and_then(|current| self.get(current))
    }

    fn origin(&self) -> Self::State {
        Position::new(self.state.identity)
    }

    fn next(&self, index: &mut Offset, state: &mut Self::State) -> Option<Token<'a>> {
        let token = self.get(*index)?;
        *state = Position {
            identity: token.span.identity,
            offset: token.span.end,
        };
        *index += 1;
        Some(token.clone())
    }

    fn input(&self) -> &Vec<Token<'a>> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Token<'a>> {
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

impl<'a: 'source, 'source> Parser<'a> {
    #[inline]
    fn priority(error: &ParseError<'a>) -> u8 {
        match &error.kind {
            ErrorKind::UnclosedDelimiter(_) => 4,
            ErrorKind::MissingSeparator(_) => 3,
            ErrorKind::ExpectedBody
            | ErrorKind::ExpectedHead
            | ErrorKind::ExpectedName
            | ErrorKind::ExpectedAnnotation => 2,
            ErrorKind::UnexpectedToken(_) => 1,
        }
    }

    #[inline]
    fn prefer(candidate: &ParseError<'a>, current: &ParseError<'a>) -> bool {
        (candidate.span.end, candidate.span.start, Self::priority(candidate))
            > (current.span.end, current.span.start, Self::priority(current))
    }

    pub fn new() -> Self {
        Parser {
            index: 0,
            state: Position::new(0),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn filter(
        length: Scale,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Formation::repetition(
            Formation::alternative([
                Formation::predicate(|token: &Token| {
                    matches!(
                        token.kind,
                        TokenKind::Punctuation(PunctuationKind::Newline)
                            | TokenKind::Punctuation(PunctuationKind::Tab)
                            | TokenKind::Punctuation(PunctuationKind::Space)
                            | TokenKind::Comment(_)
                    )
                })
                    .with_ignore(),
                Formation::predicate(|token: &Token| {
                    !matches!(
                        token.kind,
                        TokenKind::Punctuation(PunctuationKind::Newline)
                            | TokenKind::Punctuation(PunctuationKind::Tab)
                            | TokenKind::Punctuation(PunctuationKind::Space)
                            | TokenKind::Comment(_)
                    )
                }),
            ]),
            0,
            Some(length),
        )
    }

    pub fn parse(&mut self) {
        let length = self.length();

        let strained = {
            let mut former = Former::new(self);
            former.form(Self::filter(length)).collect_inputs()
        };

        self.set_input(strained);
        self.reset();

        let forms = {
            let mut former = Former::new(self);
            former.form(Self::parser()).flatten()
        };

        let mut error: Option<ParseError<'a>> = None;
        for form in forms {
            match form {
                Form::Output(output) => self.output.push(output),
                Form::Failure(failure) => {
                    if error
                        .as_ref()
                        .map(|current| Self::prefer(&failure, current))
                        .unwrap_or(true)
                    {
                        error = Some(failure);
                    }
                }
                _ => {}
            }
        }

        if let Some(err) = error {
            self.errors.push(err);
        }
    }

    pub fn execute(session: &mut Session<'a>, keys: &[Identity]) {
        for &key in keys {
            Self::process(session, key);
        }
    }

    fn process(session: &mut Session<'a>, key: Identity) {
        let (kind, tokens) = {
            let record = session.records.get(&key).unwrap();
            let tokens = if let Some(Artifact::Tokens(tokens)) = record.fetch(1) {
                Some(tokens.clone())
            } else {
                None
            };
            (record.kind.clone(), tokens)
        };

        if kind != RecordKind::Source {
            return;
        }

        let mut parser = Parser::new();
        if let Some(tokens) = tokens {
            parser.set_input(tokens);
        }
        parser.parse();

        parser.output.shrink_to_fit();

        session.errors.extend(
            parser
                .errors
                .into_iter()
                .map(SessionError::Parse),
        );

        let record = session.records.get_mut(&key).unwrap();
        record.store(2, Artifact::Elements(parser.output));
    }
}
