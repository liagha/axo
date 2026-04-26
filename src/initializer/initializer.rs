use crate::{
    combinator::{Form, Formation, Former},
    data::{Binding, BindingKind, Offset, Scale, Str},
    initializer::{ErrorKind, InitializeError},
    internal::platform::{ARCH, OS},
    parser::{Element, ElementKind, ParseError, Symbol, SymbolKind},
    reporter::Error,
    scanner::{PunctuationKind, Scanner, Token, TokenKind},
    tracker::{Location, Peekable, Position, Span},
};

pub struct Initializer<'a> {
    pub content: Str<'a>,
    pub index: Offset,
    pub state: Position,
    pub input: Vec<Token<'a>>,
    pub output: Vec<Symbol<'a>>,
    pub errors: Vec<InitializeError<'a>>,
}

impl<'a> Peekable<'a, Token<'a>> for Initializer<'a> {
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

impl<'a> Initializer<'a> {
    pub fn new(content: Str<'a>) -> Self {
        Initializer {
            content,
            index: 0,
            state: Position::new(0),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn filter<'source>(
        length: Scale,
    ) -> Formation<'a, 'source, Self, Token<'a>, Element<'a>, ParseError<'a>> {
        let ignore = |token: &Token| {
            matches!(
                token.kind,
                TokenKind::Punctuation(PunctuationKind::Newline)
                    | TokenKind::Punctuation(PunctuationKind::Tab)
                    | TokenKind::Punctuation(PunctuationKind::Space)
                    | TokenKind::Comment(_)
            )
        };

        Formation::repetition(
            Formation::alternative([
                Formation::predicate(ignore).with_ignore(),
                Formation::predicate(move |token: &Token| !ignore(token)),
            ]),
            0,
            Some(length),
        )
    }

    pub fn directive<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::alternative([
            Self::verbosity(),
            Self::input(),
            Self::output(),
            Self::target(),
            Self::discard(),
            Self::bare(),
            Self::cranelift(),
            Self::implicit(),
            Formation::anything().with_panic(|former: &mut Former<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>>, formation| {
                let form = former.forms.get_mut(formation.form).unwrap();
                let input = form.collect_inputs()[0].clone();
                let span = input.span;

                Error::new(ErrorKind::Argument(input), span)
            }),
        ])
    }

    pub fn formation<'source>(
    ) -> Formation<'a, 'source, Self, Token<'a>, Symbol<'a>, InitializeError<'a>> {
        Formation::repetition(Self::directive(), 0, None)
    }

    pub fn initialize(&mut self) -> Vec<(Location<'a>, Span)> {
        let mut scanner = Scanner::new(Position::new(0), self.content);
        scanner.scan();
        self.input = scanner.output;

        let length = self.length();
        let formation = Self::filter(length);

        let input = Former::new(self).form(formation).collect_inputs();
        self.input = input;
        self.reset();

        let mut directives = Vec::new();
        let formation = Self::formation();
        let forms = Former::new(self).form(formation).flatten();

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

        let mut has_target = false;
        let mut targets = Vec::new();

        for symbol in &directives {
            if let SymbolKind::Binding(binding) = &symbol.kind {
                if let ElementKind::Literal(target_token) = &binding.target.kind {
                    if let TokenKind::Identifier(name) = &target_token.kind {
                        if **name == *"Target" {
                            has_target = true;
                        } else if **name == *"Input" || name.starts_with("Input(") {
                            if let Some(value) = &binding.value {
                                if let ElementKind::Literal(value_token) = &value.kind {
                                    match &value_token.kind {
                                        TokenKind::String(val) | TokenKind::Identifier(val) => {
                                            targets.push((**val, symbol.span));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !has_target {
            let string = match (ARCH, OS) {
                ("x86_64", "windows") => "x86_64-pc-windows-msvc",
                ("aarch64", "windows") => "aarch64-pc-windows-msvc",
                ("x86_64", "macos") => "x86_64-apple-darwin",
                ("aarch64", "macos") => "aarch64-apple-darwin",
                ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
                ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
                _ => "unknown",
            };

            let span = Span::void();
            let target = Element::new(
                ElementKind::literal(Token::new(TokenKind::identifier(Str::from("Target")), span)),
                span,
            );
            let value = Element::new(
                ElementKind::literal(Token::new(TokenKind::identifier(Str::from(string)), span)),
                span,
            );
            directives.push(Symbol::new(
                SymbolKind::binding(Binding::new(target, Some(value), None, BindingKind::Static)),
                span,
            ));
        }

        self.output = directives;
        targets
    }
}

#[cfg(test)]
mod tests {
    use super::Initializer;
    use crate::data::Str;

    #[test]
    fn positional_input_becomes_target() {
        let mut initializer = Initializer::new(Str::from("./examples/calculator.axo"));
        let targets = initializer.initialize();

        assert!(
            targets.iter().any(|(location, _)| location.to_string() == "./examples/calculator.axo")
        );
    }

    #[test]
    fn explicit_input_becomes_target() {
        let mut initializer =
            Initializer::new(Str::from("--input ./examples/calculator.axo"));
        let targets = initializer.initialize();

        assert!(
            targets.iter().any(|(location, _)| location.to_string() == "./examples/calculator.axo")
        );
    }
}
