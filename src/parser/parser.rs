use {
    crate::{
        parser::{Element, ParseError},
        data::{Offset, Scale},
        combinator::{Classifier, Form, Former},
        scanner::{PunctuationKind, Token, TokenKind},
        tracker::{Location, Peekable, Position},
    },
};

pub struct Parser<'a> {
    pub index: Offset,
    pub position: Position<'a>,
    pub input: Vec<Token<'a>>,
    pub output: Vec<Element<'a>>,
    pub errors: Vec<ParseError<'a>>,
}

impl<'a> Peekable<'a, Token<'a>> for Parser<'a> {
    #[inline]
    fn length(&self) -> Scale {
        self.input.len()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Token<'a>> {
        let current = self.index + n;

        self.get(current)
    }

    fn peek_behind(&self, n: Offset) -> Option<&Token<'a>> {
        self.index.checked_sub(n).and_then(|current| self.get(current))
    }

    fn next(&self, index: &mut Offset, position: &mut Position<'a>) -> Option<Token<'a>> {
        if let Some(token) = self.get(*index) {
            *position = token.span.end;

            *index += 1;

            return Some(token.clone());
        }

        None
    }

    fn input(&self) -> &Vec<Token<'a>> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Token<'a>> {
        &mut self.input
    }

    fn position(&self) -> Position<'a> {
        self.position.clone()
    }

    fn position_mut(&mut self) -> &mut Position<'a> {
        &mut self.position
    }

    fn index(&self) -> Offset {
        self.index
    }

    fn index_mut(&mut self) -> &mut Offset {
        &mut self.index
    }
}

impl<'a: 'src, 'src> Parser<'a> {
    pub fn new(location: Location<'a>) -> Self {
        Parser {
            index: 0,
            position: Position::new(location),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn filter(
        length: Scale,
    ) -> Classifier<'a, 'src, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::repetition(
            Classifier::alternative([
                Classifier::predicate(|token: &Token| {
                    matches!(
                        token.kind,
                        TokenKind::Punctuation(PunctuationKind::Newline)
                            | TokenKind::Punctuation(PunctuationKind::Tab)
                            | TokenKind::Punctuation(PunctuationKind::Space)
                            | TokenKind::Punctuation(PunctuationKind::Indentation(_))
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
                            | TokenKind::Punctuation(PunctuationKind::Indentation(_))
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
