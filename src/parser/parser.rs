use crate::{
    combinator::{Classifier, Form, Former},
    data::{Offset, Scale},
    parser::{Element, ParseError},
    scanner::{PunctuationKind, Token, TokenKind},
    tracker::{Location, Peekable, Position},
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
        *state = Position { identity: token.span.identity, offset: token.span.end };
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
    pub fn new(_: Location<'a>) -> Self {
        Parser {
            index: 0,
            state: Position::new(0),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn filter(length: Scale) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::repetition(
            Classifier::alternative([
                Classifier::predicate(|token: &Token| {
                    matches!(
                        token.kind,
                        TokenKind::Punctuation(PunctuationKind::Newline)
                            | TokenKind::Punctuation(PunctuationKind::Tab)
                            | TokenKind::Punctuation(PunctuationKind::Space)
                            | TokenKind::Comment(_)
                    )
                })
                .with_ignore(),
                Classifier::predicate(|token: &Token| {
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

        for form in forms {
            match form {
                Form::Output(output) => self.output.push(output),
                Form::Failure(failure) => self.errors.push(failure),
                _ => {}
            }
        }
    }
}
