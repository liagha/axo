#![allow(dead_code)]

mod fmt;
mod lexer;

use crate::arc::Arc;
use crate::axo_span::Span;
use crate::Peekable;
use core::fmt::Debug;

pub type TransformFunction<Input, Output, Error> =
    Arc<dyn Fn(Vec<Formed<Input, Output, Error>>, Span) -> Result<Output, Input> + Send + Sync>;
pub type PredicateFunction<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
pub type ErrorFunction<Error> = Arc<dyn Fn(Span) -> Error>;

#[derive(Clone)]
pub enum PatternKind<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    Literal(Input),
    Alternative(Vec<Pattern<Input, Output, Error>>),
    Sequence(Vec<Pattern<Input, Output, Error>>),
    Repeat {
        pattern: Box<Pattern<Input, Output, Error>>,
        minimum: usize,
        maximum: Option<usize>,
    },
    Optional(Box<Pattern<Input, Output, Error>>),
    Predicate(PredicateFunction<Input>),
    Negate(Box<Pattern<Input, Output, Error>>),
    Anything,
    Required {
        pattern: Box<Pattern<Input, Output, Error>>,
        action: Action<Input, Output, Error>,
    },
}

#[derive(Clone)]
pub enum Action<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    Transform(TransformFunction<Input, Output, Error>),
    Ignore,
    Error(ErrorFunction<Error>),
    Conditional {
        found: Box<Action<Input, Output, Error>>,
        missing: Box<Action<Input, Output, Error>>,
    },
}

#[derive(Clone, Debug)]
pub struct Pattern<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    kind: PatternKind<Input, Output, Error>,
    action: Option<Action<Input, Output, Error>>,
}

impl<Input, Output, Error> Pattern<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    pub fn literal(value: Input) -> Self {
        Self {
            kind: PatternKind::Literal(value),
            action: None,
        }
    }

    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Alternative(patterns.into()),
            action: None,
        }
    }

    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(patterns.into()),
            action: None,
        }
    }

    pub fn repeat(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        minimum: usize,
        maximum: Option<usize>,
    ) -> Self {
        Self {
            kind: PatternKind::Repeat {
                pattern: pattern.into(),
                minimum,
                maximum,
            },
            action: None,
        }
    }

    pub fn optional(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Optional(pattern.into()),
            action: None,
        }
    }

    pub fn predicate(predicate: PredicateFunction<Input>) -> Self {
        Self {
            kind: PatternKind::Predicate(predicate),
            action: None,
        }
    }

    pub fn negate(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Negate(pattern.into()),
            action: None,
        }
    }

    pub fn anything() -> Self {
        Self {
            kind: PatternKind::Anything,
            action: None,
        }
    }

    pub fn required(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        action: Action<Input, Output, Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Required {
                pattern: pattern.into(),
                action,
            },
            action: None,
        }
    }

    pub fn transform(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        transform: TransformFunction<Input, Output, Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Transform(transform)),
        }
    }

    pub fn ignore(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Ignore),
        }
    }

    pub fn error(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        error_fn: ErrorFunction<Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Error(error_fn)),
        }
    }

    pub fn conditional(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        found: Action<Input, Output, Error>,
        missing: Action<Input, Output, Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Conditional {
                found: Box::new(found),
                missing: Box::new(missing),
            }),
        }
    }

    pub fn with_action(mut self, action: Action<Input, Output, Error>) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_ignore(mut self) -> Self {
        self.action = Some(Action::Ignore);
        self
    }

    pub fn with_error(mut self, error_fn: ErrorFunction<Error>) -> Self {
        self.action = Some(Action::Error(error_fn));
        self
    }

    pub fn with_conditional(
        mut self,
        found: Action<Input, Output, Error>,
        missing: Action<Input, Output, Error>,
    ) -> Self {
        self.action = Some(Action::Conditional {
            found: Box::new(found),
            missing: Box::new(missing),
        });
        self
    }
}

#[derive(Debug)]
pub enum Form<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    Empty,
    Raw(Input),
    Single(Output),
    Multiple(Vec<Formed<Input, Output, Error>>),
    Error(Error),
}

impl<Input, Output, Error> Clone for Form<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Raw(value) => Self::Raw(value.clone()),
            Self::Single(value) => Self::Single(value.clone()),
            Self::Multiple(forms) => {
                Self::Multiple(forms.iter().map(|form| form.clone()).collect())
            }
            Self::Error(e) => Self::Error(e.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Formed<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    pub form: Form<Input, Output, Error>,
    pub span: Span,
}

pub trait Former<Input, Output, Error>: Peekable<Input>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn expand(formed: Formed<Input, Output, Error>) -> Vec<Formed<Input, Output, Error>>;
    fn catch(forms: Vec<Formed<Input, Output, Error>>) -> Option<Formed<Input, Output, Error>>;
    fn action(
        action: &Action<Input, Output, Error>,
        formed_items: Vec<Formed<Input, Output, Error>>,
        span: Span,
    ) -> Formed<Input, Output, Error>;
    fn matches(&mut self, pattern: &Pattern<Input, Output, Error>, offset: usize) -> (bool, usize);
    fn form(&mut self, pattern: Pattern<Input, Output, Error>) -> Formed<Input, Output, Error>;
}

impl<Input, Output, Error> Formed<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    pub fn new(form: Form<Input, Output, Error>, span: Span) -> Self {
        Self { form, span }
    }
}

impl<Matcher, Input, Output, Error> Former<Input, Output, Error> for Matcher
where
    Matcher: Peekable<Input>,
    Input: Clone + PartialEq + Debug,
    Output: Clone + Debug,
    Error: Clone + Debug,
{
    fn expand(formed: Formed<Input, Output, Error>) -> Vec<Formed<Input, Output, Error>> {
        match formed.form {
            Form::Empty => vec![],
            Form::Raw(_) | Form::Single(_) | Form::Error(_) => vec![formed],
            Form::Multiple(items) => items,
        }
    }

    fn catch(forms: Vec<Formed<Input, Output, Error>>) -> Option<Formed<Input, Output, Error>> {
        for formed in forms {
            match formed.form.clone() {
                Form::Multiple(forms) => {
                    if let Some(error) = Self::catch(forms) {
                        return Some(error);
                    } else {
                        continue;
                    }
                }
                Form::Error(_) => {
                    return Some(formed);
                }
                _ => continue,
            }
        }

        None
    }

    fn action(
        action: &Action<Input, Output, Error>,
        items: Vec<Formed<Input, Output, Error>>,
        span: Span,
    ) -> Formed<Input, Output, Error> {
        if let Some(err) = Self::catch(items.clone()) {
            return err;
        }

        let result = match action {
            Action::Transform(transform) => match transform(items, span.clone()) {
                Ok(token) => Formed::new(Form::Single(token), span),
                Err(_) => Formed::new(Form::Empty, span),
            },

            Action::Ignore => Formed::new(Form::Empty, span),

            Action::Error(function) => Formed::new(Form::Error(function(span.clone())), span),

            Action::Conditional { found, missing } => {
                if !items.is_empty() {
                    Self::action(found, items, span)
                } else {
                    Self::action(missing, Vec::new(), span)
                }
            }
        };

        result
    }

    fn matches(&mut self, pattern: &Pattern<Input, Output, Error>, offset: usize) -> (bool, usize) {
        let result = match &pattern.kind {
            PatternKind::Literal(expect) => {
                if let Some(peek) = self.peek_ahead(offset) {
                    let matches = peek == expect;
                    (matches, offset + 1)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Alternative(patterns) => {
                for pattern in patterns {
                    let (matches, new_offset) = self.matches(pattern, offset);

                    match matches {
                        true => {
                            return (true, new_offset);
                        }
                        _ => {}
                    }
                }

                (false, offset)
            }

            PatternKind::Sequence(sequence) => {
                let mut sequence_offset = offset;

                for pattern in sequence {
                    let (matches, pattern_offset) = self.matches(pattern, sequence_offset);

                    match matches {
                        true => {
                            sequence_offset = pattern_offset;
                        }
                        false => {
                            return (false, offset);
                        }
                    }
                }

                (true, sequence_offset)
            }

            PatternKind::Repeat {
                pattern,
                minimum,
                maximum,
            } => {
                let mut current_offset = offset;
                let mut count = 0;

                while let Some(_) = self.peek_ahead(current_offset) {
                    let (matches, new_offset) = self.matches(pattern, current_offset);
                    match matches {
                        true => {
                            if current_offset == new_offset {
                                break;
                            }

                            count += 1;
                            current_offset = new_offset;

                            if let Some(max) = maximum {
                                if count >= *max {
                                    break;
                                }
                            }
                        }
                        false => {
                            break;
                        }
                    }
                }

                let result = count >= *minimum;

                (result, current_offset)
            }

            PatternKind::Optional(pattern) => {
                let (matches, new_offset) = self.matches(pattern, offset);

                let offset = match matches {
                    true => new_offset,
                    false => offset,
                };

                (true, offset)
            }

            PatternKind::Predicate(function) => {
                if let Some(peek) = self.peek_ahead(offset) {
                    let result = function(&peek);

                    (result, if result { offset + 1 } else { offset })
                } else {
                    (false, offset)
                }
            }

            PatternKind::Negate(pattern) => {
                let (matches, _) = self.matches(pattern, offset);

                let result = !matches;

                let final_offset = if result && self.peek_ahead(offset).is_some() {
                    offset + 1
                } else {
                    offset
                };

                (result, final_offset)
            }

            PatternKind::Anything => {
                if self.peek_ahead(offset).is_some() {
                    (true, offset + 1)
                } else {
                    (false, offset)
                }
            }

            PatternKind::Required { pattern, .. } => {
                let (_, new_offset) = self.matches(pattern, offset);

                (true, new_offset)
            }
        };
        result
    }

    fn form(&mut self, pattern: Pattern<Input, Output, Error>) -> Formed<Input, Output, Error> {
        let matches = self.matches(&pattern, 0).0;
        let start = self.position();

        if matches {
            let formed = match &pattern.kind {
                PatternKind::Literal(input) => {
                    self.next();

                    let end = self.position();
                    let formed =
                        Formed::new(Form::Raw(input.clone()), Span::new(start.clone(), end));

                    formed
                }

                PatternKind::Alternative(patterns) => {
                    for subpattern in patterns {
                        if self.matches(subpattern, 0).0 {
                            return self.form(subpattern.clone());
                        }
                    }

                    Formed::new(Form::Empty, Span::point(self.position()))
                }

                PatternKind::Sequence(sequence) => {
                    let mut formed_sequence = Vec::new();

                    for subpattern in sequence {
                        let formed = self.form(subpattern.clone());

                        formed_sequence.push(formed);
                    }

                    let formed = Formed::new(
                        Form::Multiple(formed_sequence),
                        Span::new(start.clone(), self.position()),
                    );

                    formed
                }

                PatternKind::Repeat {
                    pattern: subpattern,
                    maximum,
                    ..
                } => {
                    let mut formed_repeat = Vec::new();

                    while let Some(_) = self.peek() {
                        if self.matches(subpattern, 0).0 {
                            let formed = self.form((**subpattern).clone());

                            formed_repeat.push(formed);

                            if let Some(max) = maximum {
                                if formed_repeat.len() >= *max {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }

                    let formed = Formed::new(
                        Form::Multiple(formed_repeat),
                        Span::new(start.clone(), self.position()),
                    );

                    formed
                }

                PatternKind::Optional(subpattern) => {
                    if self.matches(subpattern, 0).0 {
                        self.form((**subpattern).clone())
                    } else {
                        Formed::new(Form::Empty, Span::point(start.clone()))
                    }
                }

                PatternKind::Predicate(predicate) => {
                    if let Some(input) = self.peek() {
                        if predicate(&input) {
                            let character = self.next().unwrap();
                            let formed = Formed::new(
                                Form::Raw(character),
                                Span::new(start.clone(), self.position()),
                            );

                            formed
                        } else {
                            Formed::new(Form::Empty, Span::point(start.clone()))
                        }
                    } else {
                        Formed::new(Form::Empty, Span::point(start.clone()))
                    }
                }

                PatternKind::Negate(subpattern) => {
                    if !self.matches(subpattern, 0).0 {
                        if let Some(input) = self.next() {
                            let formed = Formed::new(
                                Form::Raw(input),
                                Span::new(start.clone(), self.position()),
                            );

                            formed
                        } else {
                            Formed::new(Form::Empty, Span::point(self.position()))
                        }
                    } else {
                        Formed::new(Form::Empty, Span::point(start.clone()))
                    }
                }

                PatternKind::Anything => {
                    if let Some(input) = self.next() {
                        let formed = Formed::new(
                            Form::Raw(input),
                            Span::new(start.clone(), self.position()),
                        );

                        formed
                    } else {
                        Formed::new(Form::Empty, Span::point(self.position()))
                    }
                }

                PatternKind::Required {
                    pattern: subpattern,
                    action,
                } => {
                    if self.matches(subpattern, 0).0 {
                        self.form(*subpattern.clone())
                    } else {
                        let span = Span::point(self.position());
                        let formed = Self::action(action, Vec::new(), span.clone());

                        formed
                    }
                }
            };

            let end = self.position();
            let span = Span::new(start, end);

            match &pattern.action {
                Some(action) => {
                    let items = Self::expand(formed);

                    let formed = Self::action(action, items, span.clone());

                    formed
                }
                None => formed,
            }
        } else {
            Formed::new(Form::Empty, Span::point(start.clone()))
        }
    }
}
