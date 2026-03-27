use crate::{
    combinator::{Classifier, Form, Former},
    data::{Offset, Scale, Str},
    initializer::InitializeError,
    parser::{Element, ElementKind, ParseError, Symbol, SymbolKind},
    scanner::{PunctuationKind, Scanner, Token, TokenKind},
    tracker::{Location, Peekable, Position, Span},
};

pub struct Initializer<'a> {
    pub index: Offset,
    pub position: Position<'a>,
    pub input: Vec<Token<'a>>,
    pub output: Vec<Symbol<'a>>,
    pub errors: Vec<InitializeError<'a>>,
}

impl<'a> Peekable<'a, Token<'a>> for Initializer<'a> {
    #[inline]
    fn length(&self) -> Scale {
        self.input.len()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Token<'a>> {
        let current = self.index + n;
        self.get(current)
    }

    fn peek_behind(&self, n: Offset) -> Option<&Token<'a>> {
        self.index
            .checked_sub(n)
            .and_then(|current| self.get(current))
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

impl<'a> Initializer<'a> {
    pub fn new(location: Location<'a>) -> Self {
        Initializer {
            index: 0,
            position: Position::new(location),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn filter<'source>(
        length: Scale,
    ) -> Classifier<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        Classifier::repetition(
            Classifier::alternative([
                Classifier::predicate(is_ignored).with_ignore(),
                Classifier::predicate(|token: &Token| !is_ignored(token)),
            ]),
            0,
            Some(length),
        )
    }

    pub fn directive<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Classifier::alternative([
            Self::verbosity(),
            Self::input(),
            Self::output(),
            Self::discard(),
            Self::bare(),
            Self::implicit_input(),
            Classifier::anything().with_ignore(),
        ])
    }

    pub fn classifier<'source>(
    ) -> Classifier<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Classifier::repetition(Self::directive(), 0, None)
    }

    pub fn initialize(&mut self) -> Vec<(Location<'a>, Span<'a>)> {
        let location = Location::Flag;
        let mut scanner = Scanner::new(location);

        scanner.set_location(location);
        scanner.prepare();
        scanner.scan();

        self.input = scanner.output;

        let length = self.length();
        let classifier = Self::filter(length);

        let inputs = {
            let mut former = Former::new(self);
            former.form(classifier).collect_inputs()
        };

        self.input = inputs;
        self.reset();

        let mut directives = Vec::new();
        let classifier = Self::classifier();

        let forms = {
            let mut former = Former::new(self);
            former.form(classifier).flatten()
        };

        for form in forms {
            match form {
                Form::Output(output) => directives.push(output),
                Form::Failure(failure) => self.errors.push(failure),
                Form::Multiple(multiple) => {
                    for form in multiple {
                        directives.push(form.unwrap_output().clone());
                    }
                }
                _ => {}
            }
        }

        let targets = directives
            .iter()
            .filter_map(|symbol| {
                let name = target_name(symbol)?;
                if name == Str::from("Input") || name.starts_with("Input(") {
                    let value = value_name(symbol)?;
                    Some((Location::Entry(value), symbol.span))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let inputs = directives
            .iter()
            .enumerate()
            .filter_map(|(index, symbol)| {
                if target_name(symbol)? == Str::from("Input") {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if inputs.len() > 1 {
            for (count, index) in inputs.into_iter().enumerate() {
                if let Some(symbol) = directives.get_mut(index) {
                    rename_target(symbol, format!("Input({})", count));
                }
            }
        }

        let indexes = directives
            .iter()
            .enumerate()
            .filter_map(|(index, symbol)| {
                if target_name(symbol)? == Str::from("Output") {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if indexes.len() > 1 {
            for (count, index) in indexes.into_iter().enumerate() {
                if let Some(symbol) = directives.get_mut(index) {
                    rename_target(symbol, format!("{}({})", "Output", count));
                }
            }
        }

        self.output = directives;

        targets
    }
}

fn is_ignored(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::Punctuation(PunctuationKind::Newline)
            | TokenKind::Punctuation(PunctuationKind::Tab)
            | TokenKind::Punctuation(PunctuationKind::Space)
            | TokenKind::Punctuation(PunctuationKind::Indentation(_))
            | TokenKind::Punctuation(PunctuationKind::Semicolon)
            | TokenKind::Comment(_)
    )
}

fn target_name<'a>(symbol: &Symbol<'a>) -> Option<Str<'a>> {
    if let SymbolKind::Binding(binding) = &symbol.kind {
        if let ElementKind::Literal(token) = &binding.target.kind {
            if let TokenKind::Identifier(name) = &token.kind {
                return Some(name.clone());
            }
        }
    }
    None
}

fn value_name<'a>(symbol: &Symbol<'a>) -> Option<Str<'a>> {
    if let SymbolKind::Binding(binding) = &symbol.kind {
        if let Some(value) = &binding.value {
            if let ElementKind::Literal(token) = &value.kind {
                if let TokenKind::Identifier(name) = &token.kind {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

fn rename_target(symbol: &mut Symbol, name: String) {
    if let SymbolKind::Binding(binding) = &mut symbol.kind {
        if let ElementKind::Literal(token) = &binding.target.kind {
            let span = token.span;
            binding.target = Box::new(Element::new(
                ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(name)), span)),
                span,
            ));
        }
    }
}
